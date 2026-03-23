import QtQuick
import Quickshell
import Quickshell.Io

QtObject {
    id: root

    property string socketPath: defaultSocketPath()
    readonly property bool busy: socket.connected || waitingForResponse || pendingPayload.length > 0
    property string lastStatus: "idle"
    property string lastError: ""
    property string lastOutput: ""
    property bool lastChanged: false
    property string lastRequestedMode: ""
    property string lastEffectiveMode: ""
    property string lastDecision: ""

    signal succeeded()
    signal failed(string message)

    property string pendingPayload: ""
    property bool waitingForResponse: false

    function repair(text, mode, policy) {
        if (busy) {
            return false
        }

        if (socketPath.length === 0) {
            handleFailure("waytrim socketPath is empty")
            return false
        }

        const request = {
            type: "repair",
            version: 1,
            mode: mode,
            text: text,
        }

        if (policy !== undefined && policy !== null) {
            request.policy = policy
        }

        resetLastResponse()
        pendingPayload = JSON.stringify(request) + "\n"
        socket.connected = true
        return true
    }

    function defaultSocketPath() {
        const runtimeDir = Quickshell.env("XDG_RUNTIME_DIR")
        if (!runtimeDir) {
            return ""
        }

        return `${runtimeDir}/waytrim/waytrim.sock`
    }

    function handleResponse(rawResponse) {
        let response
        try {
            response = JSON.parse(rawResponse)
        } catch (error) {
            waitingForResponse = false
            socket.connected = false
            handleFailure(`failed to parse waytrim response: ${error}`)
            return
        }

        if (response.status === "error") {
            waitingForResponse = false
            socket.connected = false
            handleFailure(response.message || "waytrim request failed")
            return
        }

        if (response.status !== "ok" || !response.report) {
            waitingForResponse = false
            socket.connected = false
            handleFailure("unexpected waytrim response")
            return
        }

        const report = response.report
        waitingForResponse = false
        socket.connected = false

        lastStatus = response.status
        lastError = ""
        lastOutput = report.output || ""
        lastChanged = report.changed === true
        lastRequestedMode = report.requested_mode || ""
        lastEffectiveMode = report.effective_mode || ""
        lastDecision = report.decision || ""

        succeeded()
    }

    function handleFailure(message) {
        lastStatus = "error"
        lastError = message
        lastOutput = ""
        lastChanged = false
        lastRequestedMode = ""
        lastEffectiveMode = ""
        lastDecision = ""
        failed(message)
    }

    function resetLastResponse() {
        lastStatus = "running"
        lastError = ""
        lastOutput = ""
        lastChanged = false
        lastRequestedMode = ""
        lastEffectiveMode = ""
        lastDecision = ""
    }

    Socket {
        id: socket

        path: root.socketPath
        parser: SplitParser {
            onRead: rawResponse => root.handleResponse(rawResponse)
        }

        onConnectedChanged: {
            if (connected && root.pendingPayload.length > 0) {
                write(root.pendingPayload)
                flush()
                root.pendingPayload = ""
                root.waitingForResponse = true
                return
            }

            if (!connected && root.waitingForResponse) {
                root.waitingForResponse = false
                root.handleFailure("waytrim disconnected without a response")
            }
        }

        onError: error => {
            root.pendingPayload = ""
            root.waitingForResponse = false
            root.handleFailure(`waytrim socket error: ${error}`)
        }
    }
}
