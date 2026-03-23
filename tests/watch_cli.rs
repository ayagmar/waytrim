mod support;

use std::process::Command;

use serde_json::Value;
use support::temp_file_path;
use waytrim::{Mode, WatchPaths, write_watch_idle_status};

#[test]
fn waytrim_watch_status_json_defaults_to_idle_without_state() {
    let state_path = temp_file_path("watch-cli-status-empty");

    let output = Command::new(env!("CARGO_BIN_EXE_waytrim-watch"))
        .args(["--status", "--json", "--state-path"])
        .arg(&state_path)
        .output()
        .expect("run waytrim-watch --status --json");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    let status: Value = serde_json::from_str(&stdout).expect("parse status json");
    assert_eq!(status["status"], "idle");
    assert_eq!(status["original_available"], false);
    assert_eq!(status["clipboard_source"], "unknown");
}

#[test]
fn waytrim_watch_status_json_reports_saved_mode_and_event() {
    let state_path = temp_file_path("watch-cli-status-idle");
    let paths = WatchPaths {
        state_path: state_path.clone(),
    };

    write_watch_idle_status(&paths, Mode::Auto).expect("write watch idle status");

    let output = Command::new(env!("CARGO_BIN_EXE_waytrim-watch"))
        .args(["--status", "--json", "--state-path"])
        .arg(&state_path)
        .output()
        .expect("run waytrim-watch --status --json");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    let status: Value = serde_json::from_str(&stdout).expect("parse status json");
    assert_eq!(status["status"], "idle");
    assert_eq!(status["mode"], "auto");
    assert_eq!(status["message"], "watcher started in auto mode");
    assert!(status["event_id"].as_u64().unwrap_or(0) > 0);
}
