mod support;

use support::{fixture_input, run_waytrim};

#[test]
fn preview_shows_before_after_markers_and_changed_lines() {
    let input = "This is a wrapped\nparagraph from a terminal.\n";
    let output = run_waytrim(&["prose", "--preview"], input);

    assert!(output.contains("--- before"));
    assert!(output.contains("+++ after"));
    assert!(output.contains("-This is a wrapped"));
    assert!(output.contains("+This is a wrapped paragraph from a terminal."));
}

#[test]
fn preview_shows_wrapped_blockquote_changes() {
    let input = fixture_input("prose/docs/blockquote-wrap");
    let output = run_waytrim(&["prose", "--preview"], &input);

    assert!(output.contains("--- before"));
    assert!(output.contains("+++ after"));
    assert!(output.contains(
        "+> This copied quote wrapped in a narrow pane and should become one quoted line again."
    ));
}

#[test]
fn preview_shows_pi_answer_wrap_changes() {
    let input = fixture_input("prose/pi/pi-answer-wrap");
    let output = run_waytrim(&["prose", "--preview"], &input);

    assert!(output.contains("--- before"));
    assert!(output.contains("+++ after"));
    assert!(output.contains("+You probably want to send copied TUI output through the same cleanup path used for stdin so hard-wrapped assistant replies become normal paragraphs again."));
}

#[test]
fn preview_repairs_mixed_pi_prose_without_collapsing_command_block() {
    let input = fixture_input("prose/pi/mixed-command-block");
    let output = run_waytrim(&["prose", "--preview"], &input);

    assert!(output.contains("--- before"));
    assert!(output.contains("+++ after"));
    assert!(output.contains(
        "+If you only want the clipboard regression, run this command after copying the reply:"
    ));
    assert!(output.contains(r#"+cargo test \"#));
    assert!(output.contains(r#"+  --test clipboard_flow \"#));
}

#[test]
fn preview_reports_no_changes_for_already_clean_fixture() {
    let input = fixture_input("prose/negative/already-clean");
    let output = run_waytrim(&["prose", "--preview"], &input);

    assert!(output.contains("--- before"));
    assert!(output.contains("+++ after"));
    assert!(output.contains("(no changes)"));
}

#[test]
fn preview_reports_no_changes_for_already_clean_command_fixture() {
    let input = fixture_input("command/negative/already-clean-command");
    let output = run_waytrim(&["command", "--preview"], &input);

    assert!(output.contains("--- before"));
    assert!(output.contains("+++ after"));
    assert!(output.contains("(no changes)"));
}
