mod support;

use support::{fixture_input, fixture_meta, fixture_output, run_waytrim};

#[test]
fn prose_repairs_wrapped_ai_terminal_paragraphs() {
    let input = fixture_input("prose/ai-terminal/basic-wrap");
    let output = run_waytrim(&["prose"], &input);

    assert_eq!(output, fixture_output("prose/ai-terminal/basic-wrap"));
}

#[test]
fn prose_preserves_structured_negative_cases() {
    let input = fixture_input("prose/negative/bullets");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/negative/bullets");

    assert!(meta.preserve.iter().any(|value| value == "bullets"));
    assert_eq!(output, fixture_output("prose/negative/bullets"));
}

#[test]
fn prose_repairs_wrapped_tui_status_update_fixture() {
    let input = fixture_input("prose/tui/status-update");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/tui/status-update");

    assert!(meta.preserve.iter().any(|value| value == "bullets"));
    assert_eq!(output, fixture_output("prose/tui/status-update"));
}

#[test]
fn prose_repairs_wrapped_blockquote_fixture() {
    let input = fixture_input("prose/docs/blockquote-wrap");
    let output = run_waytrim(&["prose"], &input);

    assert_eq!(output, fixture_output("prose/docs/blockquote-wrap"));
}

#[test]
fn prose_preserves_fenced_code_fixture() {
    let input = fixture_input("prose/negative/code-fence");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/negative/code-fence");

    assert!(meta.preserve.iter().any(|value| value == "code blocks"));
    assert_eq!(output, fixture_output("prose/negative/code-fence"));
}

#[test]
fn prose_repairs_wrapped_pi_answer_fixture() {
    let input = fixture_input("prose/pi/pi-answer-wrap");
    let output = run_waytrim(&["prose"], &input);

    assert_eq!(output, fixture_output("prose/pi/pi-answer-wrap"));
}

#[test]
fn prose_repairs_wrapped_pi_bullets_fixture() {
    let input = fixture_input("prose/pi/pi-bullets");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/pi/pi-bullets");

    assert!(meta.preserve.iter().any(|value| value == "bullets"));
    assert_eq!(output, fixture_output("prose/pi/pi-bullets"));
}

#[test]
fn prose_repairs_wrapped_pi_numbered_steps_fixture() {
    let input = fixture_input("prose/pi/pi-numbered-steps");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/pi/pi-numbered-steps");

    assert!(meta.preserve.iter().any(|value| value == "numbered lists"));
    assert_eq!(output, fixture_output("prose/pi/pi-numbered-steps"));
}

#[test]
fn prose_repairs_wrapped_pi_blockquote_fixture() {
    let input = fixture_input("prose/pi/pi-blockquote");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/pi/pi-blockquote");

    assert!(meta.preserve.iter().any(|value| value == "blockquotes"));
    assert_eq!(output, fixture_output("prose/pi/pi-blockquote"));
}

#[test]
fn prose_preserves_pi_fenced_code_fixture() {
    let input = fixture_input("prose/pi/pi-code-fence");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/pi/pi-code-fence");

    assert!(meta.preserve.iter().any(|value| value == "code blocks"));
    assert_eq!(output, fixture_output("prose/pi/pi-code-fence"));
}

#[test]
fn prose_repairs_wrapped_pi_inline_code_bullets_fixture() {
    let input = fixture_input("prose/pi/pi-inline-code-bullets");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/pi/pi-inline-code-bullets");

    assert!(meta.preserve.iter().any(|value| value == "inline code"));
    assert_eq!(output, fixture_output("prose/pi/pi-inline-code-bullets"));
}

#[test]
fn prose_preserves_mixed_docs_command_block_fixture() {
    let input = fixture_input("prose/docs/mixed-command-block");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/docs/mixed-command-block");

    assert!(meta.preserve.iter().any(|value| value == "command blocks"));
    assert_eq!(output, fixture_output("prose/docs/mixed-command-block"));
}

#[test]
fn prose_preserves_mixed_pi_command_block_fixture() {
    let input = fixture_input("prose/pi/mixed-command-block");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/pi/mixed-command-block");

    assert!(meta.preserve.iter().any(|value| value == "command blocks"));
    assert_eq!(output, fixture_output("prose/pi/mixed-command-block"));
}
