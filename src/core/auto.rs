use super::command::repair_command;
use super::detect::{
    looks_like_command, looks_like_command_transcript, looks_like_label_plus_command,
    looks_like_prose, looks_like_soft_wrapped_prose,
};
use super::policy::{AutoPolicy, Mode, RepairPolicy};
use super::prose::repair_prose;
use super::report::{ExplainStep, RepairDecision, RepairOutcome};
use super::text::{finish_with_newline, strip_uniform_single_leading_space};

pub(crate) fn repair_auto(input: &str, policy: &RepairPolicy) -> RepairOutcome {
    if looks_like_command(input) {
        let (output, mut explain) = repair_command(input);
        explain.insert(
            0,
            ExplainStep {
                message: String::from("classified input as command-like"),
            },
        );
        return RepairOutcome::new(Mode::Command, RepairDecision::AutoCommand, output, explain);
    }

    if looks_like_command_transcript(input) {
        return RepairOutcome::new(
            Mode::Auto,
            RepairDecision::AutoMinimal,
            minimal_prose_safe_cleanup(input),
            vec![ExplainStep {
                message: String::from(
                    "detected command transcript; used minimal prose-safe cleanup",
                ),
            }],
        );
    }

    if looks_like_label_plus_command(input) {
        return RepairOutcome::new(
            Mode::Auto,
            RepairDecision::AutoMinimal,
            minimal_prose_safe_cleanup(input),
            vec![ExplainStep {
                message: String::from(
                    "detected label-plus-command snippet; used minimal prose-safe cleanup",
                ),
            }],
        );
    }

    if looks_like_prose(input)
        || matches!(policy.auto_policy, AutoPolicy::ProsePreferred)
            && looks_like_soft_wrapped_prose(input)
    {
        let (output, mut explain) = repair_prose(input, policy);
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
        minimal_prose_safe_cleanup(input),
        Vec::new(),
    )
}

pub(crate) fn minimal_prose_safe_cleanup(input: &str) -> String {
    let input = strip_uniform_single_leading_space(input);
    let mut output = Vec::new();
    let mut blank_count = 0;

    for line in input.lines() {
        let trimmed_end = line.trim_end();

        if trimmed_end.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                output.push(String::new());
            }
            continue;
        }

        blank_count = 0;
        output.push(trimmed_end.to_string());
    }

    while output.last().is_some_and(|line| line.is_empty()) {
        output.pop();
    }

    finish_with_newline(output.join("\n"))
}
