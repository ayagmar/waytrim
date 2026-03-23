mod support;

use std::fs;
use std::process::Command;

use support::{temp_file_path, write_executable_script};

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
    let helper = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("contrib/niri/waytrim-clipboard-prose");

    let status = Command::new("sh")
        .arg(helper)
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
