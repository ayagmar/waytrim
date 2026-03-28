use super::command::repair_command;
use super::detect::{
    looks_like_command, looks_like_command_transcript, looks_like_label_plus_command,
    looks_like_prose, looks_like_reaction_snippet, looks_like_soft_wrapped_prose,
    looks_like_yaml_mapping_input,
};
use super::policy::{AutoPolicy, Mode, RepairPolicy};
use super::prose::repair_prose;
use super::report::{ExplainStep, RepairDecision, RepairOutcome};
use super::text::{
    minimal_line_preserving_cleanup, normalize_reaction_snippet, strip_uniform_copied_margin,
};

pub(crate) fn repair_auto(input: &str, policy: &RepairPolicy) -> RepairOutcome {
    let input = strip_uniform_copied_margin(input);

    if looks_like_reaction_snippet(&input) {
        return RepairOutcome::new(
            Mode::Auto,
            RepairDecision::AutoMinimal,
            normalize_reaction_snippet(&input),
            vec![ExplainStep {
                message: String::from("detected reaction snippet; collapsed it into one line"),
            }],
        );
    }

    if looks_like_command(&input) {
        let (output, mut explain) = repair_command(&input);
        explain.insert(
            0,
            ExplainStep {
                message: String::from("classified input as command-like"),
            },
        );
        return RepairOutcome::new(Mode::Command, RepairDecision::AutoCommand, output, explain);
    }

    if looks_like_command_transcript(&input) {
        return RepairOutcome::new(
            Mode::Auto,
            RepairDecision::AutoMinimal,
            minimal_line_preserving_cleanup(&input),
            vec![ExplainStep {
                message: String::from(
                    "detected command transcript; used minimal prose-safe cleanup",
                ),
            }],
        );
    }

    if looks_like_label_plus_command(&input) {
        return RepairOutcome::new(
            Mode::Auto,
            RepairDecision::AutoMinimal,
            minimal_line_preserving_cleanup(&input),
            vec![ExplainStep {
                message: String::from(
                    "detected label-plus-command snippet; used minimal prose-safe cleanup",
                ),
            }],
        );
    }

    if looks_like_yaml_mapping_input(&input) {
        return RepairOutcome::new(
            Mode::Auto,
            RepairDecision::AutoMinimal,
            minimal_line_preserving_cleanup(&input),
            vec![ExplainStep {
                message: String::from(
                    "detected structured yaml-like text; preserved line structure",
                ),
            }],
        );
    }

    if looks_like_prose(&input)
        || matches!(policy.auto_policy, AutoPolicy::ProsePreferred)
            && looks_like_soft_wrapped_prose(&input)
    {
        let (output, mut explain) = repair_prose(&input, policy);
        explain.insert(
            0,
            ExplainStep {
                message: String::from("classified input as prose-like"),
            },
        );
        return RepairOutcome::new(Mode::Prose, RepairDecision::AutoProse, output, explain);
    }

    RepairOutcome::new(
        Mode::Auto,
        RepairDecision::AutoMinimal,
        minimal_line_preserving_cleanup(&input),
        Vec::new(),
    )
}
