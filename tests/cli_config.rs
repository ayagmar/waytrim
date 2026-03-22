use waytrim::Mode;
use waytrim::cli::CliConfig;

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
