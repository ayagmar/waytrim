mod support;

use std::cell::RefCell;

use support::temp_file_path;
use waytrim::clipboard::{ClipboardBackend, ClipboardError};
use waytrim::{
    AutoClipboardConfig, AutoClipboardStatus, Mode, WatchPaths, restore_last_original,
    run_auto_clipboard_once,
};

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

fn watch_config() -> AutoClipboardConfig {
    AutoClipboardConfig {
        mode: Mode::Auto,
        ..AutoClipboardConfig::default()
    }
}

fn watch_paths(stem: &str) -> WatchPaths {
    WatchPaths {
        state_path: temp_file_path(stem),
    }
}

#[test]
fn auto_watch_repairs_changed_clipboard_and_skips_its_own_followup() {
    let clipboard = MemoryClipboard::new("This is a wrapped\nparagraph from a terminal.\n");
    let paths = watch_paths("watch-state-update");

    let first = run_auto_clipboard_once(&watch_config(), &clipboard, &paths).expect("watch once");
    assert_eq!(first.status, AutoClipboardStatus::Updated);
    assert_eq!(
        clipboard.current(),
        "This is a wrapped paragraph from a terminal.\n"
    );

    let second =
        run_auto_clipboard_once(&watch_config(), &clipboard, &paths).expect("watch once again");
    assert_eq!(second.status, AutoClipboardStatus::Skipped);
    assert_eq!(clipboard.writes().len(), 1);
}

#[test]
fn auto_watch_reports_unchanged_clean_clipboard() {
    let clipboard = MemoryClipboard::new("Already clean text.\n");
    let paths = watch_paths("watch-state-unchanged");

    let output = run_auto_clipboard_once(&watch_config(), &clipboard, &paths).expect("watch once");
    assert_eq!(output.status, AutoClipboardStatus::Unchanged);
    assert!(clipboard.writes().is_empty());
}

#[test]
fn auto_watch_can_restore_last_original_clipboard_text() {
    let clipboard = MemoryClipboard::new("This is a wrapped\nparagraph from a terminal.\n");
    let paths = watch_paths("watch-state-restore");

    let updated = run_auto_clipboard_once(&watch_config(), &clipboard, &paths).expect("watch once");
    assert_eq!(updated.status, AutoClipboardStatus::Updated);
    assert_eq!(
        clipboard.current(),
        "This is a wrapped paragraph from a terminal.\n"
    );

    let restored = restore_last_original(&clipboard, &paths).expect("restore original");
    assert_eq!(restored.status, AutoClipboardStatus::RestoredOriginal);
    assert_eq!(
        clipboard.current(),
        "This is a wrapped\nparagraph from a terminal.\n"
    );

    let skipped = run_auto_clipboard_once(&watch_config(), &clipboard, &paths)
        .expect("skip restore self-update");
    assert_eq!(skipped.status, AutoClipboardStatus::Skipped);
}

#[test]
fn auto_watch_reports_missing_original_when_none_was_saved() {
    let clipboard = MemoryClipboard::new("Already clean text.\n");
    let paths = watch_paths("watch-state-missing-original");

    let output = restore_last_original(&clipboard, &paths).expect("restore without saved original");
    assert_eq!(output.status, AutoClipboardStatus::MissingOriginal);
    assert!(clipboard.writes().is_empty());
}
