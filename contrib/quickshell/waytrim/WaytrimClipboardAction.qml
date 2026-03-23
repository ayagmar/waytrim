import QtQuick
import Quickshell

Item {
    id: root

    visible: false
    width: 0
    height: 0

    property string socketPath: client.defaultSocketPath()
    property string mode: "prose"
    readonly property bool busy: client.busy
    property string status: "idle"
    property string lastMessage: ""

    signal finished(string status, string message)

    function trigger() {
        if (busy) {
            return false
        }

        const clipboardText = Quickshell.clipboardText
        if (!clipboardText || clipboardText.length === 0) {
            status = "empty"
            lastMessage = "clipboard is empty"
            finished(status, lastMessage)
            return false
        }

        status = "running"
        lastMessage = ""
        return client.repair(clipboardText, mode)
    }

    WaytrimClient {
        id: client

        socketPath: root.socketPath

        onSucceeded: {
            if (lastChanged) {
                Quickshell.clipboardText = lastOutput
                root.status = "updated"
                root.lastMessage = "clipboard updated"
            } else {
                root.status = "unchanged"
                root.lastMessage = "clipboard unchanged"
            }

            root.finished(root.status, root.lastMessage)
        }

        onFailed: function(message) {
            root.status = "error"
            root.lastMessage = message
            root.finished(root.status, root.lastMessage)
        }
    }
}
