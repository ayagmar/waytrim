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
fn auto_strips_shared_copied_margin_without_flattening_internal_indentation() {
    let input = "   cat <<'EOF'\n   public class Main {\n       System.out.println(\"hi\");\n   }\n   EOF\n";
    let output = run_waytrim(&["auto"], input);

    assert_eq!(
        output,
        "cat <<'EOF'\npublic class Main {\n    System.out.println(\"hi\");\n}\nEOF\n"
    );
}

#[test]
fn auto_collapses_single_reaction_line_without_trailing_newline() {
    let input = ":rofl:\n";
    let output = run_waytrim(&["auto"], input);

    assert_eq!(output, ":rofl:");
}

#[test]
fn auto_collapses_multiline_reaction_snippet_into_one_line() {
    let input = ":rofl:\n:rofl:\n";
    let output = run_waytrim(&["auto"], input);

    assert_eq!(output, ":rofl: :rofl:");
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

#[test]
fn auto_declines_to_rewrite_mixed_docs_command_block_fixture() {
    let input = fixture_input("auto/ambiguous/docs-mixed-command-block");
    let output = run_waytrim(&["auto"], &input);

    assert_eq!(
        output,
        fixture_output("auto/ambiguous/docs-mixed-command-block")
    );
}

#[test]
fn auto_keeps_real_tui_copied_install_command_block_conservative() {
    let input = fixture_input("prose/docs/watcher-install-command-block");
    let output = run_waytrim(&["auto"], &input);
    let meta = support::fixture_meta("prose/docs/watcher-install-command-block");

    assert!(
        meta.avoid
            .iter()
            .any(|value| value == "prose-command merge")
    );
    assert_eq!(
        output,
        fixture_output("prose/docs/watcher-install-command-block")
    );
}

#[test]
fn auto_keeps_real_tui_copied_systemctl_command_block_conservative() {
    let input = fixture_input("prose/docs/watcher-systemctl-command-block");
    let output = run_waytrim(&["auto"], &input);
    let meta = support::fixture_meta("prose/docs/watcher-systemctl-command-block");

    assert!(
        meta.avoid
            .iter()
            .any(|value| value == "prose-command merge")
    );
    assert_eq!(
        output,
        fixture_output("prose/docs/watcher-systemctl-command-block")
    );
}

#[test]
fn auto_repairs_prose_with_uniform_vertical_gutter_fixture() {
    let input = fixture_input("prose/tui/vertical-gutter-wrap");
    let output = run_waytrim(&["auto"], &input);

    assert_eq!(output, fixture_output("prose/tui/vertical-gutter-wrap"));
}

#[test]
fn auto_declines_to_rewrite_mixed_pi_command_block_fixture() {
    let input = fixture_input("auto/ambiguous/pi-mixed-command-block");
    let output = run_waytrim(&["auto"], &input);

    assert_eq!(
        output,
        fixture_output("auto/ambiguous/pi-mixed-command-block")
    );
}

#[test]
fn auto_declines_to_rewrite_alignment_sensitive_columns_fixture() {
    let input = fixture_input("auto/ambiguous/aligned-columns");
    let output = run_waytrim(&["auto"], &input);

    assert_eq!(output, fixture_output("auto/ambiguous/aligned-columns"));
}

#[test]
fn auto_leaves_already_clean_command_fixture_unchanged() {
    let input = fixture_input("command/negative/already-clean-command");
    let output = run_waytrim(&["auto"], &input);

    assert_eq!(
        output,
        fixture_output("command/negative/already-clean-command")
    );
}

#[test]
fn auto_leaves_heading_fixture_unchanged() {
    let input = fixture_input("prose/negative/heading");
    let output = run_waytrim(&["auto"], &input);

    assert_eq!(output, fixture_output("prose/negative/heading"));
}

#[test]
fn auto_leaves_indented_block_fixture_unchanged() {
    let input = fixture_input("prose/negative/indented-block");
    let output = run_waytrim(&["auto"], &input);

    assert_eq!(output, fixture_output("prose/negative/indented-block"));
}

#[test]
fn auto_keeps_prose_preferred_fixture_unchanged_when_conservative() {
    let input = fixture_input("auto/ambiguous/prose-preferred-wrap");
    let output = run_waytrim(&["auto"], &input);

    assert_eq!(
        output,
        fixture_output("auto/ambiguous/prose-preferred-wrap")
    );
}

#[test]
fn auto_declines_to_merge_prose_then_command_example_fixture() {
    let input = fixture_input("auto/ambiguous/prose-then-command-example");
    let output = run_waytrim(&["auto"], &input);
    let meta = support::fixture_meta("auto/ambiguous/prose-then-command-example");

    assert!(
        meta.avoid
            .iter()
            .any(|value| value == "forced classification")
    );
    assert_eq!(
        output,
        fixture_output("auto/ambiguous/prose-then-command-example")
    );
}

#[test]
fn auto_declines_to_merge_install_section_fixture() {
    let input = fixture_input("auto/ambiguous/install-section");
    let output = run_waytrim(&["auto"], &input);
    let meta = support::fixture_meta("auto/ambiguous/install-section");

    assert!(
        meta.avoid
            .iter()
            .any(|value| value == "forced classification")
    );
    assert_eq!(output, fixture_output("auto/ambiguous/install-section"));
}

#[test]
fn auto_declines_to_rewrite_indented_command_example_fixture() {
    let input = fixture_input("auto/ambiguous/indented-command-example");
    let output = run_waytrim(&["auto"], &input);
    let meta = support::fixture_meta("auto/ambiguous/indented-command-example");

    assert!(
        meta.preserve
            .iter()
            .any(|value| value == "command examples")
    );
    assert_eq!(
        output,
        fixture_output("auto/ambiguous/indented-command-example")
    );
}

#[test]
fn auto_preserves_openapi_yaml_structure() {
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
    let output = run_waytrim(&["auto"], input);

    assert_eq!(output, input);
}

#[test]
fn auto_preserves_single_root_openapi_yaml_structure() {
    let input = "\
openapi:
  info:
    title: Example API
    version: 1.0.0
";
    let output = run_waytrim(&["auto"], input);

    assert_eq!(output, input);
}

#[test]
fn auto_preserves_two_line_yaml_mapping_structure() {
    let input = "\
name: Example API
version: 1.0.0
";
    let output = run_waytrim(&["auto"], input);

    assert_eq!(output, input);
}

#[test]
fn auto_preserves_yaml_sequence_structure() {
    let input = "\
services:
  - name: api
    port: 8080
  - name: web
    port: 3000
";
    let output = run_waytrim(&["auto"], input);

    assert_eq!(output, input);
}

#[test]
fn auto_preserves_yaml_scalar_sequence_under_mapping() {
    let input = "\
tags:
  - alpha
  - beta
";
    let output = run_waytrim(&["auto"], input);

    assert_eq!(output, input);
}
