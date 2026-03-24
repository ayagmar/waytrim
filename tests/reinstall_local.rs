use std::fs;
use std::path::PathBuf;

#[test]
fn reinstall_local_restarts_fixed_and_templated_watch_units() {
    let script = fs::read_to_string(script_path()).expect("read reinstall-local");

    assert!(script.contains("for unit in waytrim-watch.service waytrim-watch@auto.service; do"));
    assert!(script.contains("systemctl --user restart \"$unit\""));
}

fn script_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/reinstall-local")
}
