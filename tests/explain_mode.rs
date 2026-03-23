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

#[test]
fn explain_reports_policy_driven_command_block_repair() {
    use waytrim::{Mode, RepairPolicy, repair_with_policy};

    let input = "Use this command:\n\ncargo test \\\n  --test clipboard_flow\n";
    let policy = RepairPolicy {
        protect_command_blocks: false,
        ..RepairPolicy::default()
    };

    let output = waytrim::render_explain(
        Mode::Prose,
        &repair_with_policy(input, Mode::Prose, &policy),
    );

    assert!(output.contains("mode: prose"));
    assert!(
        output.contains("joined continued command lines 3-4")
            || output.contains("joined continued command lines 1-2")
    );
}

#[test]
fn explain_reports_auto_refusal_for_prose_then_command_example() {
    let input = fixture_input("auto/ambiguous/prose-then-command-example");
    let output = run_waytrim(&["auto", "--explain"], &input);

    assert!(output.contains("mode: auto"));
    assert!(output.contains("minimal prose-safe cleanup") || output.contains("no repair needed"));
}

#[test]
fn explain_reports_blank_line_noise_repair() {
    let input = fixture_input("prose/ai-terminal/blank-line-noise");
    let output = run_waytrim(&["prose", "--explain"], &input);

    assert!(output.contains("mode: prose"));
    assert!(output.contains("changed: yes"));
    assert!(output.contains("blank line") || output.contains("joined wrapped paragraph"));
}

#[test]
fn explain_reports_command_transcript_refusal() {
    let input = fixture_input("command/negative/transcript-with-status");
    let output = run_waytrim(&["command", "--explain"], &input);

    assert!(output.contains("mode: command"));
    assert!(output.contains("no repair needed") || output.contains("transcript"));
}
