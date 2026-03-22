mod support;

use support::{fixture_input, run_waytrim};

#[test]
fn explain_reports_wrapped_paragraph_repair() {
    let input = "This is a wrapped\nparagraph from a terminal.\n";
    let output = run_waytrim(&["prose", "--explain"], input);

    assert!(output.contains("mode: prose"));
    assert!(output.contains("changed: yes"));
    assert!(output.contains("- joined wrapped paragraph lines 1-2"));
    assert!(output.contains("--- output"));
    assert!(output.contains("This is a wrapped paragraph from a terminal."));
}

#[test]
fn explain_reports_no_changes_for_clean_fixture() {
    let input = fixture_input("prose/negative/already-clean");
    let output = run_waytrim(&["prose", "--explain"], &input);

    assert!(output.contains("mode: prose"));
    assert!(output.contains("changed: no"));
    assert!(output.contains("- no repair needed"));
    assert!(output.contains("--- output"));
}

#[test]
fn explain_reports_prompt_stripping_and_command_joining() {
    let input = "$ cargo test \\\n  --test clipboard_flow\n";
    let output = run_waytrim(&["command", "--explain"], input);

    assert!(output.contains("mode: command"));
    assert!(output.contains("changed: yes"));
    assert!(output.contains("- stripped shell prompt from line 1"));
    assert!(output.contains("- joined continued command lines 1-2"));
    assert!(output.contains("cargo test --test clipboard_flow"));
}

#[test]
fn explain_reports_auto_transcript_fallback() {
    let input = fixture_input("auto/ambiguous/pi-command-plus-output");
    let output = run_waytrim(&["auto", "--explain"], &input);

    assert!(output.contains("mode: auto"));
    assert!(output.contains("- detected command transcript; used minimal prose-safe cleanup"));
}
