import QtQuick
import Quickshell

Item {
    id: root

    visible: false
    width: 0
    height: 0

    property QtObject control: null
    property bool enabled: true
    property bool useNotifySend: false
    property string notifySendCommand: "notify-send"
    property bool notifyOnUpdated: false
    property bool notifyOnUnchanged: false
    property bool notifyOnRestoredOriginal: true
    property bool notifyOnError: true
    property int seenEventId: 0

    signal notificationRequested(string title, string message, string urgency)

    onControlChanged: {
        seenEventId = control ? control.lastEventId : 0
    }

    function maybeNotify() {
        if (!enabled || !control || control.lastEventId <= seenEventId) {
            return
        }

        seenEventId = control.lastEventId

        let title = "waytrim"
        let urgency = "normal"
        let shouldNotify = false

        switch (control.lastStatus) {
        case "updated":
            shouldNotify = notifyOnUpdated
            title = "waytrim updated the clipboard"
            break
        case "unchanged":
            shouldNotify = notifyOnUnchanged
            title = "waytrim left the clipboard unchanged"
            break
        case "restored_original":
            shouldNotify = notifyOnRestoredOriginal
            title = "waytrim restored the original clipboard text"
            break
        case "error":
            shouldNotify = notifyOnError
            urgency = "critical"
            title = "waytrim watcher error"
            break
        default:
            shouldNotify = false
            break
        }

        if (!shouldNotify) {
            return
        }

        const message = control.lastMessage && control.lastMessage.length > 0
            ? control.lastMessage
            : control.lastStatus

        notificationRequested(title, message, urgency)

        if (useNotifySend) {
            Quickshell.execDetached([
                notifySendCommand,
                "--app-name=waytrim",
                "--urgency",
                urgency,
                title,
                message,
            ])
        }
    }

    Connections {
        target: root.control

        function onLastEventIdChanged() {
            root.maybeNotify()
        }
    }
}
