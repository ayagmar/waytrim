use std::fs;
use std::path::PathBuf;

fn quickshell_example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("contrib/quickshell/waytrim")
        .join(name)
}

#[test]
fn quickshell_client_example_exists_and_targets_socket_ipc() {
    let contents = fs::read_to_string(quickshell_example_path("WaytrimClient.qml"))
        .expect("read WaytrimClient.qml");

    assert!(contents.contains("import Quickshell.Io"));
    assert!(contents.contains("Socket {"));
    assert!(contents.contains("type: \"repair\""));
}

#[test]
fn quickshell_clipboard_action_example_exists_and_uses_quickshell_clipboard() {
    let contents = fs::read_to_string(quickshell_example_path("WaytrimClipboardAction.qml"))
        .expect("read WaytrimClipboardAction.qml");

    assert!(contents.contains("Quickshell.clipboardText"));
    assert!(contents.contains("clipboard updated"));
    assert!(contents.contains("clipboard unchanged"));
}

#[test]
fn quickshell_watch_control_example_exists_and_stays_thin() {
    let contents = fs::read_to_string(quickshell_example_path("WaytrimWatchControl.qml"))
        .expect("read WaytrimWatchControl.qml");

    assert!(contents.contains("Process {"));
    assert!(contents.contains("waytrim-watch"));
    assert!(contents.contains("systemctl"));
    assert!(contents.contains("function toggle()"));
    assert!(contents.contains("function restoreOriginal()"));
    assert!(contents.contains("function cleanOnce(nextMode)"));
    assert!(contents.contains("watchArgs([requestedMode])"));
}

#[test]
fn quickshell_notification_example_exists_and_defaults_to_conservative_popups() {
    let contents = fs::read_to_string(quickshell_example_path("WaytrimNotifications.qml"))
        .expect("read WaytrimNotifications.qml");

    assert!(contents.contains("notificationRequested"));
    assert!(contents.contains("notifyOnUpdated: false"));
    assert!(contents.contains("notifyOnUnchanged: false"));
    assert!(contents.contains("notifyOnRestoredOriginal: true"));
    assert!(contents.contains("notifyOnError: true"));
}
