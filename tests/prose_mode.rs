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
