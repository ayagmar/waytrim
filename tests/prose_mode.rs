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
fn prose_repairs_real_tui_copied_watcher_wrap_fixture() {
    let input = fixture_input("prose/tui/watcher-wrap");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/tui/watcher-wrap");

    assert!(meta.preserve.iter().any(|value| value == "paragraphs"));
    assert_eq!(output, fixture_output("prose/tui/watcher-wrap"));
}

#[test]
fn prose_repairs_real_tui_copied_watcher_bullets_fixture() {
    let input = fixture_input("prose/tui/watcher-bullets");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/tui/watcher-bullets");

    assert!(meta.preserve.iter().any(|value| value == "bullets"));
    assert_eq!(output, fixture_output("prose/tui/watcher-bullets"));
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
fn prose_preserves_real_tui_copied_install_command_block_fixture() {
    let input = fixture_input("prose/docs/watcher-install-command-block");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/docs/watcher-install-command-block");

    assert!(
        meta.preserve
            .iter()
            .any(|value| value == "command examples")
    );
    assert_eq!(
        output,
        fixture_output("prose/docs/watcher-install-command-block")
    );
}

#[test]
fn prose_preserves_real_tui_copied_systemctl_command_block_fixture() {
    let input = fixture_input("prose/docs/watcher-systemctl-command-block");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/docs/watcher-systemctl-command-block");

    assert!(
        meta.preserve
            .iter()
            .any(|value| value == "command examples")
    );
    assert_eq!(
        output,
        fixture_output("prose/docs/watcher-systemctl-command-block")
    );
}

#[test]
fn prose_preserves_mixed_pi_command_block_fixture() {
    let input = fixture_input("prose/pi/mixed-command-block");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/pi/mixed-command-block");

    assert!(meta.preserve.iter().any(|value| value == "command blocks"));
    assert_eq!(output, fixture_output("prose/pi/mixed-command-block"));
}

#[test]
fn prose_preserves_alignment_sensitive_columns_fixture() {
    let input = fixture_input("prose/negative/aligned-columns");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/negative/aligned-columns");

    assert!(meta.preserve.iter().any(|value| value == "aligned columns"));
    assert_eq!(output, fixture_output("prose/negative/aligned-columns"));
}

#[test]
fn prose_leaves_already_clean_paragraph_fixture_unchanged() {
    let input = fixture_input("prose/negative/already-clean");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/negative/already-clean");

    assert!(meta.avoid.iter().any(|value| value == "rewrite"));
    assert_eq!(output, fixture_output("prose/negative/already-clean"));
}

#[test]
fn prose_preserves_heading_fixture() {
    let input = fixture_input("prose/negative/heading");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/negative/heading");

    assert!(meta.preserve.iter().any(|value| value == "headings"));
    assert_eq!(output, fixture_output("prose/negative/heading"));
}

#[test]
fn prose_preserves_indented_block_fixture() {
    let input = fixture_input("prose/negative/indented-block");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/negative/indented-block");

    assert!(
        meta.preserve
            .iter()
            .any(|value| value == "indented sections")
    );
    assert_eq!(output, fixture_output("prose/negative/indented-block"));
}

#[test]
fn prose_preserves_options_table_fixture() {
    let input = fixture_input("prose/docs/options-table");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/docs/options-table");

    assert!(meta.preserve.iter().any(|value| value == "aligned columns"));
    assert_eq!(output, fixture_output("prose/docs/options-table"));
}

#[test]
fn prose_collapses_useless_blank_lines_inside_wrapped_paragraph_fixture() {
    let input = fixture_input("prose/ai-terminal/blank-line-noise");
    let output = run_waytrim(&["prose"], &input);

    assert_eq!(output, fixture_output("prose/ai-terminal/blank-line-noise"));
}

#[test]
fn prose_collapses_shortcode_only_reaction_lines_into_one_line() {
    let input = ":rofl:\n:rofl:\n";
    let output = run_waytrim(&["prose"], input);

    assert_eq!(output, ":rofl: :rofl:");
}

#[test]
fn prose_collapses_emoji_only_reaction_lines_into_one_line() {
    let input = "🤣\n🤣\n";
    let output = run_waytrim(&["prose"], input);

    assert_eq!(output, "🤣 🤣");
}

#[test]
fn prose_preserves_real_section_break_fixture() {
    let input = fixture_input("prose/negative/section-break");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/negative/section-break");

    assert!(meta.preserve.iter().any(|value| value == "section breaks"));
    assert_eq!(output, fixture_output("prose/negative/section-break"));
}

#[test]
fn prose_trims_excessive_heading_padding_fixture() {
    let input = fixture_input("prose/ai-terminal/heading-padding");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/ai-terminal/heading-padding");

    assert!(meta.preserve.iter().any(|value| value == "headings"));
    assert_eq!(output, fixture_output("prose/ai-terminal/heading-padding"));
}

#[test]
fn prose_repairs_ai_terminal_spacing_noise_fixture() {
    let input = fixture_input("prose/ai-terminal/spacing-noise-wrap");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/ai-terminal/spacing-noise-wrap");

    assert!(meta.avoid.iter().any(|value| value == "rewrite"));
    assert_eq!(
        output,
        fixture_output("prose/ai-terminal/spacing-noise-wrap")
    );
}

#[test]
fn prose_repairs_ai_terminal_inline_code_followup_fixture() {
    let input = fixture_input("prose/ai-terminal/inline-code-followup");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/ai-terminal/inline-code-followup");

    assert!(meta.preserve.iter().any(|value| value == "inline code"));
    assert_eq!(
        output,
        fixture_output("prose/ai-terminal/inline-code-followup")
    );
}

#[test]
fn prose_repairs_copy_induced_spacing_noise_inside_paragraph_fixture() {
    let input = fixture_input("prose/ai-terminal/spacing-noise-paragraph");
    let output = run_waytrim(&["prose"], &input);
    let meta = fixture_meta("prose/ai-terminal/spacing-noise-paragraph");

    assert!(meta.avoid.iter().any(|value| value == "rewrite"));
    assert_eq!(
        output,
        fixture_output("prose/ai-terminal/spacing-noise-paragraph")
    );
}

#[test]
fn prose_preserves_openapi_yaml_structure() {
    let input = "\
openapi: 3.0.0
info:
  title: Example API
  version: 1.0.0
paths:
  /pets:
    get:
      summary: List pets
";
    let output = run_waytrim(&["prose"], input);

    assert_eq!(output, input);
}
