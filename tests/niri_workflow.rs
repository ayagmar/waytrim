mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use support::{temp_file_path, write_executable_script};

fn helper_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("contrib/niri/waytrim-clipboard-prose")
}

fn watch_session_helper_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("contrib/niri/waytrim-watch-session")
}

#[test]
fn niri_helper_script_is_executable() {
    let mode = fs::metadata(helper_path())
        .expect("helper metadata")
        .permissions()
        .mode();

    assert_ne!(mode & 0o111, 0, "helper is not executable");
}

#[test]
fn niri_watch_session_helper_is_executable() {
    let mode = fs::metadata(watch_session_helper_path())
        .expect("watch session helper metadata")
        .permissions()
        .mode();

    assert_ne!(mode & 0o111, 0, "watch session helper is not executable");
}

#[test]
fn niri_helper_script_forwards_to_mode_centered_clipboard_cli() {
    let args_path = temp_file_path("niri-helper-args");
    let fake_waytrim = write_executable_script(
        "fake-waytrim",
        &format!(
            "#!/bin/sh\nprintf '%s\n' \"$@\" > '{}'\n",
            args_path.display()
        ),
    );

    let status = Command::new(helper_path())
        .arg("--print")
        .env("WAYTRIM_BIN", fake_waytrim)
        .status()
        .expect("run helper script");

    assert!(status.success());
    assert_eq!(
        fs::read_to_string(args_path).expect("read forwarded args"),
        "prose\n--clipboard\n--print\n"
    );
}

#[test]
fn niri_watch_session_helper_imports_environment_and_restarts_enabled_unit() {
    let args_path = temp_file_path("niri-watch-session-args");
    let fake_systemctl = write_executable_script(
        "fake-systemctl",
        &format!(
            "#!/bin/sh\nprintf '%s\n' \"$@\" >> '{}'; if [ \"$2\" = is-enabled ]; then exit 0; fi\n",
            args_path.display()
        ),
    );

    let status = Command::new(watch_session_helper_path())
        .arg("waytrim-watch@auto.service")
        .env("SYSTEMCTL_BIN", fake_systemctl)
        .status()
        .expect("run watch session helper");

    assert!(status.success());
    assert_eq!(
        fs::read_to_string(args_path).expect("read systemctl args"),
        "--user\nimport-environment\nWAYLAND_DISPLAY\nXDG_RUNTIME_DIR\n--user\nis-enabled\n--quiet\nwaytrim-watch@auto.service\n--user\nrestart\nwaytrim-watch@auto.service\n"
    );
}

#[test]
fn niri_watch_session_helper_does_not_restart_disabled_unit() {
    let args_path = temp_file_path("niri-watch-session-disabled-args");
    let fake_systemctl = write_executable_script(
        "fake-systemctl-disabled",
        &format!(
            "#!/bin/sh\nprintf '%s\n' \"$@\" >> '{}'; if [ \"$2\" = is-enabled ]; then exit 1; fi\n",
            args_path.display()
        ),
    );

    let status = Command::new(watch_session_helper_path())
        .env("SYSTEMCTL_BIN", fake_systemctl)
        .status()
        .expect("run watch session helper");

    assert!(status.success());
    assert_eq!(
        fs::read_to_string(args_path).expect("read disabled systemctl args"),
        "--user\nimport-environment\nWAYLAND_DISPLAY\nXDG_RUNTIME_DIR\n--user\nis-enabled\n--quiet\nwaytrim-watch.service\n"
    );
}
