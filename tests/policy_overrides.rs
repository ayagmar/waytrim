use waytrim::{AutoPolicy, Mode, RepairPolicy, repair_with_policy};

#[test]
fn prose_can_normalize_aligned_columns_when_alignment_protection_is_disabled() {
    let input = "Option                 Meaning\nwaytrim prose          Repair wrapped prose\nwaytrim auto           Stay conservative by default\n";
    let policy = RepairPolicy {
        protect_aligned_columns: false,
        ..RepairPolicy::default()
    };

    let result = repair_with_policy(input, Mode::Prose, &policy);

    assert_eq!(
        result.output,
        "Option Meaning\nwaytrim prose Repair wrapped prose\nwaytrim auto Stay conservative by default\n"
    );
}

#[test]
fn prose_can_repair_standalone_command_blocks_when_command_block_protection_is_disabled() {
    let input = "Use this command:\n\ncargo test \\\n  --test clipboard_flow\n";
    let policy = RepairPolicy {
        protect_command_blocks: false,
        ..RepairPolicy::default()
    };

    let result = repair_with_policy(input, Mode::Prose, &policy);

    assert_eq!(result.output, "Use this command:\n\ncargo test --test clipboard_flow\n");
}

#[test]
fn auto_can_choose_prose_for_ambiguous_wrapped_text_when_policy_is_prose_preferred() {
    let input = "This copied answer came from a narrow pane\nand lost its paragraph shape\nbut the conservative auto policy should wait\nfor an explicit prose preference\n";
    let policy = RepairPolicy {
        auto_policy: AutoPolicy::ProsePreferred,
        ..RepairPolicy::default()
    };

    let result = repair_with_policy(input, Mode::Auto, &policy);

    assert_eq!(
        result.output,
        "This copied answer came from a narrow pane and lost its paragraph shape but the conservative auto policy should wait for an explicit prose preference\n"
    );
}
