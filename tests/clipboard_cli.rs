mod support;

use support::run_waytrim_capture;

#[test]
fn help_shows_mode_centered_clipboard_interface() {
    let output = run_waytrim_capture(&["--help"], "");

    assert!(output.status.success());
    assert!(output.stdout.contains("waytrim prose --clipboard"));
    assert!(!output.stdout.contains("waytrim clipboard prose"));
}

#[test]
fn clipboard_preview_and_print_are_rejected_as_ambiguous() {
    let output = run_waytrim_capture(&["prose", "--clipboard", "--preview", "--print"], "");

    assert!(!output.status.success());
    assert!(
        output
            .stderr
            .contains("cannot combine --preview and --print with --clipboard")
    );
}

#[test]
fn help_shows_explain_interface() {
    let output = run_waytrim_capture(&["--help"], "");

    assert!(output.status.success());
    assert!(output.stdout.contains("waytrim prose --explain"));
    assert!(
        output
            .stdout
            .contains("waytrim prose --clipboard --explain")
    );
}

#[test]
fn clipboard_explain_and_print_are_rejected_as_ambiguous() {
    let output = run_waytrim_capture(&["prose", "--clipboard", "--explain", "--print"], "");

    assert!(!output.status.success());
    assert!(
        output
            .stderr
            .contains("cannot combine --explain and --print with --clipboard")
    );
}
