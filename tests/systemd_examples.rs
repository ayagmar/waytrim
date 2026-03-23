use std::fs;
use std::path::PathBuf;

fn systemd_example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("contrib/systemd/user")
        .join(name)
}

#[test]
fn templated_watch_service_example_exists_for_mode_switching() {
    let contents = fs::read_to_string(systemd_example_path("waytrim-watch@.service"))
        .expect("read waytrim-watch@.service");

    assert!(contents.contains("Description=waytrim automatic clipboard cleaner (%i)"));
    assert!(contents.contains("ExecStart=%h/.local/bin/waytrim-watch %i"));
}
