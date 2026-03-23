use std::process::Command;

fn stdout_for(binary: &str) -> String {
    let output = Command::new(binary)
        .arg("--help")
        .output()
        .expect("run --help");

    assert!(output.status.success());
    String::from_utf8(output.stdout).expect("utf8 stdout")
}

#[test]
fn waytrim_help_mentions_usage_and_modes() {
    let output = stdout_for(env!("CARGO_BIN_EXE_waytrim"));

    assert!(output.contains("Usage:"));
    assert!(output.contains("Modes:"));
    assert!(output.contains("waytrim prose --clipboard"));
}

#[test]
fn waytrim_watch_help_mentions_status_and_restore_commands() {
    let output = stdout_for(env!("CARGO_BIN_EXE_waytrim-watch"));

    assert!(output.contains("Usage:"));
    assert!(output.contains("waytrim-watch --status --json"));
    assert!(output.contains("waytrim-watch --restore-original"));
}

#[test]
fn waytrimctl_help_mentions_json_and_text_output_modes() {
    let output = stdout_for(env!("CARGO_BIN_EXE_waytrimctl"));

    assert!(output.contains("Usage:"));
    assert!(output.contains("repair prints JSON by default"));
    assert!(output.contains("repair --text prints only repaired text"));
}

#[test]
fn waytrimd_help_mentions_socket_usage() {
    let output = stdout_for(env!("CARGO_BIN_EXE_waytrimd"));

    assert!(output.contains("Usage:"));
    assert!(output.contains("waytrimd --socket /path/to/waytrim.sock"));
}
