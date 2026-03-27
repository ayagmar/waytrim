mod auto;
mod command;
mod detect;
mod policy;
mod prose;
mod render;
mod report;
mod text;

pub use policy::{AutoPolicy, Mode, RepairPolicy};
pub use render::{render_explain, render_preview};
pub use report::{ExplainStep, RepairDecision, RepairReport, RepairResult};

use auto::repair_auto;
use command::repair_command;
use detect::looks_like_reaction_snippet;
use prose::repair_prose;

pub fn repair(input: &str, mode: Mode) -> RepairResult {
    repair_with_policy(input, mode, &RepairPolicy::default())
}

pub fn repair_with_policy(input: &str, mode: Mode, policy: &RepairPolicy) -> RepairResult {
    repair_report_with_policy(input, mode, policy).into()
}

pub fn repair_report(input: &str, mode: Mode) -> RepairReport {
    repair_report_with_policy(input, mode, &RepairPolicy::default())
}

pub fn repair_report_with_policy(input: &str, mode: Mode, policy: &RepairPolicy) -> RepairReport {
    let outcome = match mode {
        Mode::Prose => {
            let (output, explain) = repair_prose(input, policy);
            report::RepairOutcome::new(Mode::Prose, RepairDecision::RequestedMode, output, explain)
        }
        Mode::Command => {
            let (output, explain) = repair_command(input);
            report::RepairOutcome::new(
                Mode::Command,
                RepairDecision::RequestedMode,
                output,
                explain,
            )
        }
        Mode::Auto => repair_auto(input, policy),
    };

    RepairReport {
        requested_mode: mode,
        effective_mode: outcome.effective_mode,
        decision: outcome.decision,
        changed: outcome.output != input,
        output: outcome.output,
        explain: outcome.explain,
    }
}

pub(crate) fn input_looks_like_reaction_snippet(input: &str) -> bool {
    looks_like_reaction_snippet(input)
}

#[cfg(test)]
mod tests {
    use super::{RepairDecision, repair, repair_report};
    use crate::core::policy::Mode;
    use crate::core::text::minimal_line_preserving_cleanup;

    #[test]
    fn auto_falls_back_to_minimal_cleanup_for_ambiguous_input() {
        let input = "value one  \n\n\nvalue two\n";
        let result = repair(input, Mode::Auto);

        assert_eq!(result.output, minimal_line_preserving_cleanup(input));
    }

    #[test]
    fn repair_report_tracks_auto_classification_path() {
        let report = repair_report("$ cargo test \\\n  --test clipboard_flow\n", Mode::Auto);

        assert_eq!(report.requested_mode, Mode::Auto);
        assert_eq!(report.effective_mode, Mode::Command);
        assert_eq!(report.decision, RepairDecision::AutoCommand);
    }
}
