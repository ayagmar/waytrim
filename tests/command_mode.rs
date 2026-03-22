mod support;

use support::{fixture_input, fixture_meta, fixture_output, run_waytrim};

#[test]
fn command_strips_obvious_prompts() {
    let input = fixture_input("command/prompts/basic");
    let output = run_waytrim(&["command"], &input);

    assert_eq!(output, fixture_output("command/prompts/basic"));
}

#[test]
fn command_leaves_mixed_command_output_snippets_unchanged() {
    let input = fixture_input("command/negative/mixed-output");
    let output = run_waytrim(&["command"], &input);
    let meta = fixture_meta("command/negative/mixed-output");

    assert!(meta.avoid.iter().any(|value| value == "transcript-parsing"));
    assert_eq!(output, fixture_output("command/negative/mixed-output"));
}
