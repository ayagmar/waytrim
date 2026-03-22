mod support;

use std::fs;

use support::{fixture_input, run_waytrim_capture_env, temp_dir_path};

#[test]
fn missing_user_config_uses_builtin_defaults() {
    let config_home = temp_dir_path("missing-config-home");
    fs::create_dir_all(&config_home).expect("create config home");

    let output = run_waytrim_capture_env(
        &["prose"],
        "This is a wrapped\nparagraph.\n",
        &[("XDG_CONFIG_HOME", config_home.to_str().expect("utf8 path"))],
    );

    assert!(output.status.success());
    assert_eq!(output.stdout, "This is a wrapped paragraph.\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn invalid_user_config_warns_and_falls_back_to_defaults() {
    let config_home = temp_dir_path("invalid-config-home");
    let config_dir = config_home.join("waytrim");
    fs::create_dir_all(&config_dir).expect("create config dir");
    fs::write(
        config_dir.join("config.toml"),
        "[defaults\nmode = 'prose'\n",
    )
    .expect("write invalid toml");

    let output = run_waytrim_capture_env(
        &["prose"],
        "This is a wrapped\nparagraph.\n",
        &[("XDG_CONFIG_HOME", config_home.to_str().expect("utf8 path"))],
    );

    assert!(output.status.success());
    assert_eq!(output.stdout, "This is a wrapped paragraph.\n");
    assert!(output.stderr.contains("warning: failed to load config"));
}

#[test]
fn valid_user_config_sets_default_mode_and_preview() {
    let config_home = temp_dir_path("valid-config-home");
    let config_dir = config_home.join("waytrim");
    fs::create_dir_all(&config_dir).expect("create config dir");
    fs::write(
        config_dir.join("config.toml"),
        "[defaults]\nmode = 'auto'\npreview = true\n",
    )
    .expect("write config");

    let output = run_waytrim_capture_env(
        &[],
        "This is a wrapped\nparagraph from a terminal.\n",
        &[("XDG_CONFIG_HOME", config_home.to_str().expect("utf8 path"))],
    );

    assert!(output.status.success());
    assert!(output.stdout.contains("--- before"));
    assert!(output.stdout.contains("+++ after"));
}

#[test]
fn cli_flag_overrides_configured_preview_default() {
    let config_home = temp_dir_path("override-preview-home");
    let config_dir = config_home.join("waytrim");
    fs::create_dir_all(&config_dir).expect("create config dir");
    fs::write(
        config_dir.join("config.toml"),
        "[defaults]\npreview = true\n",
    )
    .expect("write config");

    let output = run_waytrim_capture_env(
        &["--no-preview"],
        "This is a wrapped\nparagraph.\n",
        &[("XDG_CONFIG_HOME", config_home.to_str().expect("utf8 path"))],
    );

    assert!(output.status.success());
    assert_eq!(output.stdout, "This is a wrapped paragraph.\n");
}

#[test]
fn config_can_enable_explain_by_default() {
    let config_home = temp_dir_path("default-explain-home");
    let config_dir = config_home.join("waytrim");
    fs::create_dir_all(&config_dir).expect("create config dir");
    fs::write(
        config_dir.join("config.toml"),
        "[defaults]\nexplain = true\n",
    )
    .expect("write config");

    let output = run_waytrim_capture_env(
        &["prose"],
        "This is a wrapped\nparagraph.\n",
        &[("XDG_CONFIG_HOME", config_home.to_str().expect("utf8 path"))],
    );

    assert!(output.status.success());
    assert!(output.stdout.contains("mode: prose"));
    assert!(output.stdout.contains("changed: yes"));
}

#[test]
fn config_can_switch_auto_to_prose_preferred() {
    let config_home = temp_dir_path("prose-preferred-home");
    let config_dir = config_home.join("waytrim");
    fs::create_dir_all(&config_dir).expect("create config dir");
    fs::write(
        config_dir.join("config.toml"),
        "[defaults]\nmode = 'auto'\n\n[auto]\npolicy = 'prose_preferred'\n",
    )
    .expect("write config");

    let input = fixture_input("auto/ambiguous/prose-preferred-wrap");
    let output = run_waytrim_capture_env(
        &[],
        &input,
        &[("XDG_CONFIG_HOME", config_home.to_str().expect("utf8 path"))],
    );

    assert!(output.status.success());
    assert_eq!(
        output.stdout,
        "This copied answer came from a narrow pane and lost its paragraph shape but the conservative auto policy should wait for an explicit prose preference\n"
    );
}
