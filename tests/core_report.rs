mod support;

use support::fixture_input;
use waytrim::{Mode, RepairDecision, repair_report};

#[test]
fn direct_mode_report_tracks_requested_and_effective_mode() {
    let report = repair_report("This is a wrapped\nparagraph.\n", Mode::Prose);

    assert_eq!(report.requested_mode, Mode::Prose);
    assert_eq!(report.effective_mode, Mode::Prose);
    assert_eq!(report.decision, RepairDecision::RequestedMode);
    assert!(report.changed);
    assert_eq!(report.output, "This is a wrapped paragraph.\n");
}

#[test]
fn auto_report_tracks_command_classification() {
    let input = fixture_input("command/prompts/basic");
    let report = repair_report(&input, Mode::Auto);

    assert_eq!(report.requested_mode, Mode::Auto);
    assert_eq!(report.effective_mode, Mode::Command);
    assert_eq!(report.decision, RepairDecision::AutoCommand);
}

#[test]
fn auto_report_tracks_minimal_cleanup_path() {
    let input = fixture_input("auto/ambiguous/install-section");
    let report = repair_report(&input, Mode::Auto);

    assert_eq!(report.requested_mode, Mode::Auto);
    assert_eq!(report.effective_mode, Mode::Auto);
    assert_eq!(report.decision, RepairDecision::AutoMinimal);
}
