mod support;

use support::{fixture_input, fixture_output, run_waytrim};

#[test]
fn auto_chooses_command_when_input_is_obviously_a_command() {
    let input = fixture_input("command/prompts/basic");
    let output = run_waytrim(&["auto"], &input);

    assert_eq!(output, fixture_output("command/prompts/basic"));
}

#[test]
fn auto_falls_back_to_minimal_prose_safe_cleanup_when_ambiguous() {
    let input = fixture_input("auto/ambiguous/minimal-cleanup");
    let output = run_waytrim(&["auto"], &input);

    assert_eq!(output, fixture_output("auto/ambiguous/minimal-cleanup"));
}

#[test]
fn auto_declines_to_merge_label_plus_command_snippets() {
    let input = fixture_input("auto/ambiguous/label-plus-command");
    let output = run_waytrim(&["auto"], &input);

    assert_eq!(output, fixture_output("auto/ambiguous/label-plus-command"));
}

#[test]
fn auto_declines_to_rewrite_pi_command_plus_output_snippet() {
    let input = fixture_input("auto/ambiguous/pi-command-plus-output");
    let output = run_waytrim(&["auto"], &input);

    assert_eq!(
        output,
        fixture_output("auto/ambiguous/pi-command-plus-output")
    );
}
