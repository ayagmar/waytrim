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
