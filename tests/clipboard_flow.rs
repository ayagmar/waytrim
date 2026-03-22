mod support;

use std::cell::RefCell;

use support::{fixture_input, fixture_output};
use waytrim::Mode;
use waytrim::cli::{CliConfig, ClipboardFlowStatus, run_clipboard_flow};
use waytrim::clipboard::{ClipboardBackend, ClipboardError};

struct MemoryClipboard {
    value: RefCell<String>,
    writes: RefCell<Vec<String>>,
}

impl MemoryClipboard {
    fn new(initial: &str) -> Self {
        Self {
            value: RefCell::new(initial.to_string()),
            writes: RefCell::new(Vec::new()),
        }
    }

    fn current(&self) -> String {
        self.value.borrow().clone()
    }

    fn writes(&self) -> Vec<String> {
        self.writes.borrow().clone()
    }
}

impl ClipboardBackend for MemoryClipboard {
    fn read_text(&self) -> Result<String, ClipboardError> {
        Ok(self.value.borrow().clone())
    }

    fn write_text(&self, text: &str) -> Result<(), ClipboardError> {
        *self.value.borrow_mut() = text.to_string();
        self.writes.borrow_mut().push(text.to_string());
        Ok(())
    }
}

#[test]
fn clipboard_flow_repairs_and_writes_back_when_text_changes() {
    let clipboard = MemoryClipboard::new("This is a wrapped\nparagraph.\n");
    let config = CliConfig {
        mode: Mode::Prose,
        clipboard: true,
        preview: false,
        explain: false,
        print: false,
    };

    let output = run_clipboard_flow(&config, &clipboard).expect("run clipboard flow");

    assert_eq!(output.status, ClipboardFlowStatus::Updated);
    assert!(output.stdout.is_empty());
    assert!(output.stderr.contains("clipboard updated"));
    assert_eq!(clipboard.current(), "This is a wrapped paragraph.\n");
    assert_eq!(
        clipboard.writes(),
        vec![String::from("This is a wrapped paragraph.\n")]
    );
}

#[test]
fn clipboard_flow_prints_and_writes_back_when_requested() {
    let clipboard = MemoryClipboard::new("This is a wrapped\nparagraph.\n");
    let config = CliConfig {
        mode: Mode::Prose,
        clipboard: true,
        preview: false,
        explain: false,
        print: true,
    };

    let output = run_clipboard_flow(&config, &clipboard).expect("run clipboard flow");

    assert_eq!(output.status, ClipboardFlowStatus::Updated);
    assert_eq!(output.stdout, "This is a wrapped paragraph.\n");
    assert!(output.stderr.contains("clipboard updated"));
    assert_eq!(clipboard.current(), "This is a wrapped paragraph.\n");
}

#[test]
fn clipboard_preview_shows_changes_without_mutating_clipboard() {
    let clipboard = MemoryClipboard::new("This is a wrapped\nparagraph.\n");
    let config = CliConfig {
        mode: Mode::Prose,
        clipboard: true,
        preview: true,
        explain: false,
        print: false,
    };

    let output = run_clipboard_flow(&config, &clipboard).expect("run clipboard flow");

    assert_eq!(output.status, ClipboardFlowStatus::Preview);
    assert!(output.stdout.contains("--- before"));
    assert!(output.stdout.contains("+++ after"));
    assert!(output.stderr.contains("clipboard preview only"));
    assert_eq!(clipboard.current(), "This is a wrapped\nparagraph.\n");
    assert!(clipboard.writes().is_empty());
}

#[test]
fn clipboard_explain_shows_repairs_without_mutating_clipboard() {
    let clipboard = MemoryClipboard::new("This is a wrapped\nparagraph.\n");
    let config = CliConfig {
        mode: Mode::Prose,
        clipboard: true,
        preview: false,
        explain: true,
        print: false,
    };

    let output = run_clipboard_flow(&config, &clipboard).expect("run clipboard flow");

    assert_eq!(output.status, ClipboardFlowStatus::Explain);
    assert!(output.stdout.contains("mode: prose"));
    assert!(output.stdout.contains("- joined wrapped paragraph lines 1-2"));
    assert!(output.stderr.contains("clipboard explain only"));
    assert_eq!(clipboard.current(), "This is a wrapped\nparagraph.\n");
    assert!(clipboard.writes().is_empty());
}

#[test]
fn clipboard_flow_reports_unchanged_text_as_success() {
    let clipboard = MemoryClipboard::new("Already clean text.\n");
    let config = CliConfig {
        mode: Mode::Prose,
        clipboard: true,
        preview: false,
        explain: false,
        print: false,
    };

    let output = run_clipboard_flow(&config, &clipboard).expect("run clipboard flow");

    assert_eq!(output.status, ClipboardFlowStatus::Unchanged);
    assert!(output.stdout.is_empty());
    assert!(output.stderr.contains("clipboard unchanged"));
    assert!(clipboard.writes().is_empty());
}

#[test]
fn clipboard_flow_handles_empty_clipboard_without_crashing() {
    let clipboard = MemoryClipboard::new("");
    let config = CliConfig {
        mode: Mode::Prose,
        clipboard: true,
        preview: false,
        explain: false,
        print: false,
    };

    let output = run_clipboard_flow(&config, &clipboard).expect("run clipboard flow");

    assert_eq!(output.status, ClipboardFlowStatus::Empty);
    assert!(output.stdout.is_empty());
    assert!(output.stderr.contains("clipboard is empty"));
    assert!(clipboard.writes().is_empty());
}

#[test]
fn clipboard_flow_uses_tui_status_fixture_through_prose_mode() {
    let input = fixture_input("prose/tui/status-update");
    let expected = fixture_output("prose/tui/status-update");
    let clipboard = MemoryClipboard::new(&input);
    let config = CliConfig {
        mode: Mode::Prose,
        clipboard: true,
        preview: false,
        explain: false,
        print: false,
    };

    let output = run_clipboard_flow(&config, &clipboard).expect("run clipboard flow");

    assert_eq!(output.status, ClipboardFlowStatus::Updated);
    assert_eq!(clipboard.current(), expected);
}

#[test]
fn clipboard_flow_reports_unchanged_for_fenced_code_fixture() {
    let input = fixture_input("prose/negative/code-fence");
    let clipboard = MemoryClipboard::new(&input);
    let config = CliConfig {
        mode: Mode::Prose,
        clipboard: true,
        preview: false,
        explain: false,
        print: false,
    };

    let output = run_clipboard_flow(&config, &clipboard).expect("run clipboard flow");

    assert_eq!(output.status, ClipboardFlowStatus::Unchanged);
}

#[test]
fn clipboard_flow_repairs_pi_bullets_fixture_through_prose_mode() {
    let input = fixture_input("prose/pi/pi-bullets");
    let expected = fixture_output("prose/pi/pi-bullets");
    let clipboard = MemoryClipboard::new(&input);
    let config = CliConfig {
        mode: Mode::Prose,
        clipboard: true,
        preview: false,
        explain: false,
        print: false,
    };

    let output = run_clipboard_flow(&config, &clipboard).expect("run clipboard flow");

    assert_eq!(output.status, ClipboardFlowStatus::Updated);
    assert_eq!(clipboard.current(), expected);
}

#[test]
fn clipboard_flow_reports_unchanged_for_pi_fenced_code_fixture() {
    let input = fixture_input("prose/pi/pi-code-fence");
    let clipboard = MemoryClipboard::new(&input);
    let config = CliConfig {
        mode: Mode::Prose,
        clipboard: true,
        preview: false,
        explain: false,
        print: false,
    };

    let output = run_clipboard_flow(&config, &clipboard).expect("run clipboard flow");

    assert_eq!(output.status, ClipboardFlowStatus::Unchanged);
}

#[test]
fn clipboard_flow_repairs_mixed_pi_prose_without_changing_command_block() {
    let input = fixture_input("prose/pi/mixed-command-block");
    let expected = fixture_output("prose/pi/mixed-command-block");
    let clipboard = MemoryClipboard::new(&input);
    let config = CliConfig {
        mode: Mode::Prose,
        clipboard: true,
        preview: false,
        explain: false,
        print: false,
    };

    let output = run_clipboard_flow(&config, &clipboard).expect("run clipboard flow");

    assert_eq!(output.status, ClipboardFlowStatus::Updated);
    assert_eq!(clipboard.current(), expected);
}

#[test]
fn clipboard_flow_reports_unchanged_for_alignment_sensitive_fixture() {
    let input = fixture_input("prose/negative/aligned-columns");
    let clipboard = MemoryClipboard::new(&input);
    let config = CliConfig {
        mode: Mode::Prose,
        clipboard: true,
        preview: false,
        explain: false,
        print: false,
    };

    let output = run_clipboard_flow(&config, &clipboard).expect("run clipboard flow");

    assert_eq!(output.status, ClipboardFlowStatus::Unchanged);
    assert!(clipboard.writes().is_empty());
}

#[test]
fn clipboard_preview_reports_no_changes_for_already_clean_fixture() {
    let input = fixture_input("prose/negative/already-clean");
    let clipboard = MemoryClipboard::new(&input);
    let config = CliConfig {
        mode: Mode::Prose,
        clipboard: true,
        preview: true,
        explain: false,
        print: false,
    };

    let output = run_clipboard_flow(&config, &clipboard).expect("run clipboard flow");

    assert_eq!(output.status, ClipboardFlowStatus::Preview);
    assert!(output.stdout.contains("(no changes)"));
    assert!(output.stderr.contains("clipboard preview only"));
    assert!(clipboard.writes().is_empty());
}

#[test]
fn clipboard_flow_command_mode_reports_unchanged_for_already_clean_command() {
    let input = fixture_input("command/negative/already-clean-command");
    let clipboard = MemoryClipboard::new(&input);
    let config = CliConfig {
        mode: Mode::Command,
        clipboard: true,
        preview: false,
        explain: false,
        print: false,
    };

    let output = run_clipboard_flow(&config, &clipboard).expect("run clipboard flow");

    assert_eq!(output.status, ClipboardFlowStatus::Unchanged);
    assert!(clipboard.writes().is_empty());
}

#[test]
fn clipboard_flow_command_mode_preserves_transcript_as_unchanged() {
    let input = fixture_input("command/negative/pi-command-plus-output");
    let clipboard = MemoryClipboard::new(&input);
    let config = CliConfig {
        mode: Mode::Command,
        clipboard: true,
        preview: false,
        explain: false,
        print: false,
    };

    let output = run_clipboard_flow(&config, &clipboard).expect("run clipboard flow");

    assert_eq!(output.status, ClipboardFlowStatus::Unchanged);
    assert!(clipboard.writes().is_empty());
}

#[test]
fn clipboard_flow_reports_unchanged_for_heading_fixture() {
    let input = fixture_input("prose/negative/heading");
    let clipboard = MemoryClipboard::new(&input);
    let config = CliConfig {
        mode: Mode::Prose,
        clipboard: true,
        preview: false,
        explain: false,
        print: false,
    };

    let output = run_clipboard_flow(&config, &clipboard).expect("run clipboard flow");

    assert_eq!(output.status, ClipboardFlowStatus::Unchanged);
    assert!(clipboard.writes().is_empty());
}

#[test]
fn clipboard_flow_reports_unchanged_for_indented_block_fixture() {
    let input = fixture_input("prose/negative/indented-block");
    let clipboard = MemoryClipboard::new(&input);
    let config = CliConfig {
        mode: Mode::Prose,
        clipboard: true,
        preview: false,
        explain: false,
        print: false,
    };

    let output = run_clipboard_flow(&config, &clipboard).expect("run clipboard flow");

    assert_eq!(output.status, ClipboardFlowStatus::Unchanged);
    assert!(clipboard.writes().is_empty());
}
