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
