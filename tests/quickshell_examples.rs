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
