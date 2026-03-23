import QtQuick
import Quickshell
import Quickshell.Io

QtObject {
    id: root

    property string watchCommand: "waytrim-watch"
    property string systemctlCommand: "systemctl"
    property string statePath: ""
    property string serviceUnitTemplate: "waytrim-watch@.service"
    property bool useSystemd: true
    property int pollIntervalMs: 2000

    property bool enabled: false
    property string mode: "auto"
    readonly property bool busy: statusProcess.running || lifecycleProcess.running || actionProcess.running
    property string lastStatus: "idle"
    property string lastMessage: ""
    property string lastError: ""
    property bool originalAvailable: false
    property string clipboardSource: "unknown"
    property int lastEventId: 0

    property string pendingAction: ""
    property string pendingMode: ""
    property var statusCommand: []
    property var lifecycleCommand: []
    property var actionCommand: []

    signal refreshed()
    signal actionFinished(string status, string message)
    signal actionFailed(string message)

    Component.onCompleted: refresh()

    function toggle() {
        return enabled ? stop() : start(mode)
    }

    function start(nextMode) {
        const requestedMode = normalizeMode(nextMode)
        mode = requestedMode

        if (!useSystemd) {
            Quickshell.execDetached([watchCommand, requestedMode])
            enabled = true
            lastError = ""
            lastStatus = "idle"
            lastMessage = `started watcher in ${requestedMode} mode`
            actionFinished(lastStatus, lastMessage)
            refresh()
            return true
        }

        return runAction("start", [systemctlCommand, "--user", "start", serviceUnitFor(requestedMode)], requestedMode)
    }

    function stop() {
        if (!useSystemd) {
            lastStatus = "error"
            lastError = "stopping the watcher requires a systemd-managed service"
            lastMessage = lastError
            actionFailed(lastError)
            return false
        }

        return runAction("stop", [systemctlCommand, "--user", "stop", serviceUnitFor(mode)], mode)
    }

    function restoreOriginal() {
        return runAction("restore", watchArgs(["--restore-original"]), mode)
    }

    function cleanOnce(nextMode) {
        const requestedMode = normalizeMode(nextMode)
        return runAction("clean_once", watchArgs([requestedMode, "--clean-once"]), requestedMode)
    }

    function refresh() {
        statusCommand = watchArgs(["--status", "--json"])
        statusProcess.running = true

        if (!useSystemd) {
            return true
        }

        lifecycleCommand = [systemctlCommand, "--user", "is-active", "--quiet", serviceUnitFor(mode)]
        lifecycleProcess.running = true
        return true
    }

    function normalizeMode(nextMode) {
        if (!nextMode || nextMode.length === 0) {
            return mode.length > 0 ? mode : "auto"
        }

        return nextMode
    }

    function serviceUnitFor(nextMode) {
        if (!serviceUnitTemplate.includes("@.service")) {
            return serviceUnitTemplate
        }

        return serviceUnitTemplate.replace("@.service", `@${normalizeMode(nextMode)}.service`)
    }

    function watchArgs(extraArgs) {
        const args = [watchCommand]
        args.push(...extraArgs)

        if (statePath.length > 0) {
            args.push("--state-path")
            args.push(statePath)
        }

        return args
    }

    function runAction(name, command, nextMode) {
        if (busy) {
            return false
        }

        pendingAction = name
        pendingMode = normalizeMode(nextMode)
        actionCommand = command
        actionProcess.running = true
        return true
    }

    function applyStatusSnapshot(rawStatus) {
        let snapshot
        try {
            snapshot = JSON.parse(rawStatus)
        } catch (error) {
            lastStatus = "error"
            lastError = `failed to parse waytrim watch status: ${error}`
            lastMessage = lastError
            refreshed()
            return
        }

        lastStatus = snapshot.status || "idle"
        lastMessage = snapshot.message || ""
        lastError = lastStatus === "error" ? lastMessage : ""
        originalAvailable = snapshot.original_available === true
        clipboardSource = snapshot.clipboard_source || "unknown"
        lastEventId = snapshot.event_id || 0

        if (snapshot.mode) {
            mode = snapshot.mode
        }

        refreshed()
    }

    Timer {
        id: pollTimer

        interval: root.pollIntervalMs
        repeat: true
        running: root.pollIntervalMs > 0
        onTriggered: root.refresh()
    }

    Process {
        id: statusProcess

        running: false
        command: root.statusCommand
        stdout: StdioCollector {}
        stderr: StdioCollector {}

        onExited: code => {
            if (code !== 0) {
                root.lastStatus = "error"
                root.lastError = String(stderr.text || stdout.text || "failed to query waytrim watch status").trim()
                root.lastMessage = root.lastError
                root.refreshed()
                return
            }

            root.applyStatusSnapshot(String(stdout.text || "").trim())
        }
    }

    Process {
        id: lifecycleProcess

        running: false
        command: root.lifecycleCommand
        stdout: StdioCollector {}
        stderr: StdioCollector {}

        onExited: code => {
            root.enabled = code === 0
        }
    }

    Process {
        id: actionProcess

        running: false
        command: root.actionCommand
        stdout: StdioCollector {}
        stderr: StdioCollector {}

        onExited: code => {
            const message = String(stderr.text || stdout.text || "").trim()

            if (code !== 0) {
                root.lastStatus = "error"
                root.lastError = message.length > 0 ? message : `waytrim ${root.pendingAction} failed`
                root.lastMessage = root.lastError
                root.actionFailed(root.lastError)
                root.refresh()
                return
            }

            root.lastError = ""
            if (root.pendingAction === "start") {
                root.mode = root.pendingMode
                root.enabled = true
                root.lastStatus = "idle"
                root.lastMessage = `started watcher in ${root.pendingMode} mode`
            } else if (root.pendingAction === "stop") {
                root.enabled = false
                root.lastStatus = "idle"
                root.lastMessage = "stopped watcher"
            } else if (message.length > 0) {
                root.lastMessage = message
            }

            root.actionFinished(root.lastStatus, root.lastMessage)
            root.refresh()
        }
    }
}
