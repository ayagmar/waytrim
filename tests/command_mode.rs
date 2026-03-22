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

#[test]
fn command_strips_common_host_prompts() {
    let input = fixture_input("command/prompts/host-shell");
    let output = run_waytrim(&["command"], &input);

    assert_eq!(output, fixture_output("command/prompts/host-shell"));
}

#[test]
fn command_repairs_pi_multiline_command_fixture() {
    let input = fixture_input("command/pi/pi-command");
    let output = run_waytrim(&["command"], &input);
    let meta = fixture_meta("command/pi/pi-command");

    assert!(meta.preserve.iter().any(|value| value == "shell command"));
    assert_eq!(output, fixture_output("command/pi/pi-command"));
}

#[test]
fn command_preserves_pi_command_plus_output_fixture() {
    let input = fixture_input("command/negative/pi-command-plus-output");
    let output = run_waytrim(&["command"], &input);
    let meta = fixture_meta("command/negative/pi-command-plus-output");

    assert!(
        meta.preserve
            .iter()
            .any(|value| value == "transcript shape")
    );
    assert_eq!(
        output,
        fixture_output("command/negative/pi-command-plus-output")
    );
}
