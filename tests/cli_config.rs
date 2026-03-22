use waytrim::{AutoPolicy, Mode, RepairPolicy};
use waytrim::cli::{CliArgs, CliConfig, ConfigDefaults};

#[test]
fn parses_mode_centered_clipboard_flags() {
    let config = CliConfig::parse(["prose", "--clipboard"]).expect("parse config");

    assert_eq!(config.mode, Mode::Prose);
    assert!(config.clipboard);
    assert!(!config.preview);
    assert!(!config.print);
}

#[test]
fn parses_default_mode_without_explicit_mode() {
    let config = CliConfig::parse(["--clipboard"]).expect("parse config");

    assert_eq!(config.mode, Mode::Prose);
    assert!(config.clipboard);
}

#[test]
fn rejects_ambiguous_clipboard_preview_print_combination() {
    let error = CliConfig::parse(["prose", "--clipboard", "--preview", "--print"])
        .expect_err("expected parse error");

    assert!(error.contains("cannot combine --preview and --print with --clipboard"));
}

#[test]
fn rejects_subcommand_style_clipboard_shape() {
    let error = CliConfig::parse(["clipboard", "prose"]).expect_err("expected parse error");

    assert!(error.contains("unknown argument: clipboard"));
}

#[test]
fn rejects_preview_and_explain_together() {
    let error =
        CliConfig::parse(["prose", "--preview", "--explain"]).expect_err("expected parse error");

    assert!(error.contains("cannot combine --preview and --explain"));
}

#[test]
fn parses_explain_flag() {
    let config = CliConfig::parse(["command", "--explain"]).expect("parse config");

    assert_eq!(config.mode, Mode::Command);
    assert!(config.explain);
}

#[test]
fn cli_args_track_explicit_boolean_overrides() {
    let args = CliArgs::parse(["--preview", "--no-print"]).expect("parse args");

    assert_eq!(args.preview, Some(true));
    assert_eq!(args.print, Some(false));
}

#[test]
fn resolved_cli_config_prefers_explicit_cli_values_over_file_defaults() {
    let args = CliArgs::parse(["command", "--no-preview"]).expect("parse args");
    let defaults = ConfigDefaults {
        mode: Mode::Prose,
        clipboard: false,
        preview: true,
        explain: false,
        print: false,
        policy: RepairPolicy {
            auto_policy: AutoPolicy::ProsePreferred,
            ..RepairPolicy::default()
        },
    };

    let config = CliConfig::resolve(args, defaults).expect("resolve config");

    assert_eq!(config.mode, Mode::Command);
    assert!(!config.preview);
    assert_eq!(config.policy.auto_policy, AutoPolicy::ProsePreferred);
}

#[test]
fn rejects_preview_and_explain_after_merge() {
    let args = CliArgs::parse(["--preview"]).expect("parse args");
    let defaults = ConfigDefaults {
        explain: true,
        ..ConfigDefaults::default()
    };

    let error = CliConfig::resolve(args, defaults).expect_err("expected resolve error");

    assert!(error.contains("cannot combine --preview and --explain"));
}
