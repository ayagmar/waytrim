mod support;

use std::cell::RefCell;

use support::temp_file_path;
use waytrim::clipboard::{ClipboardBackend, ClipboardError};
use waytrim::{
    AutoClipboardConfig, AutoClipboardStatus, Mode, WatchClipboardSource, WatchEventStatus,
    WatchPaths, read_watch_status, restore_last_original, run_auto_clipboard_once,
    run_manual_clipboard_once,
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

struct WriteFailClipboard {
    value: String,
}

impl WriteFailClipboard {
    fn new(initial: &str) -> Self {
        Self {
            value: initial.to_string(),
        }
    }
}

impl ClipboardBackend for WriteFailClipboard {
    fn read_text(&self) -> Result<String, ClipboardError> {
        Ok(self.value.clone())
    }

    fn write_text(&self, _text: &str) -> Result<(), ClipboardError> {
        Err(ClipboardError::CommandFailed {
            command: String::from("wl-copy"),
            detail: String::from("permission denied"),
        })
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

#[test]
fn manual_clean_once_ignores_skip_guard_but_still_marks_its_own_write() {
    let clipboard = MemoryClipboard::new("This is a wrapped\nparagraph from a terminal.\n");
    let paths = watch_paths("watch-state-manual-override");

    let updated = run_auto_clipboard_once(&watch_config(), &clipboard, &paths).expect("watch once");
    assert_eq!(updated.status, AutoClipboardStatus::Updated);

    let restored = restore_last_original(&clipboard, &paths).expect("restore original");
    assert_eq!(restored.status, AutoClipboardStatus::RestoredOriginal);

    let manual =
        run_manual_clipboard_once(&watch_config(), &clipboard, &paths).expect("manual clean once");
    assert_eq!(manual.status, AutoClipboardStatus::Updated);
    assert_eq!(
        clipboard.current(),
        "This is a wrapped paragraph from a terminal.\n"
    );

    let skipped = run_auto_clipboard_once(&watch_config(), &clipboard, &paths)
        .expect("skip manual self-update");
    assert_eq!(skipped.status, AutoClipboardStatus::Skipped);
}

#[test]
fn watch_status_tracks_last_event_and_restore_availability() {
    let clipboard = MemoryClipboard::new("This is a wrapped\nparagraph from a terminal.\n");
    let paths = watch_paths("watch-state-status");

    let initial = read_watch_status(&paths).expect("read initial status");
    assert_eq!(initial.status, WatchEventStatus::Idle);
    assert!(!initial.original_available);
    assert_eq!(initial.clipboard_source, WatchClipboardSource::Unknown);

    run_auto_clipboard_once(&watch_config(), &clipboard, &paths).expect("watch once");
    let updated = read_watch_status(&paths).expect("read updated status");
    assert_eq!(updated.mode, Some(Mode::Auto));
    assert_eq!(updated.status, WatchEventStatus::Updated);
    assert_eq!(updated.message, "clipboard updated");
    assert!(updated.original_available);
    assert_eq!(
        updated.clipboard_source,
        WatchClipboardSource::CleanedOutput
    );
    assert!(updated.updated_at_ms.is_some());
    assert!(updated.event_id > 0);

    restore_last_original(&clipboard, &paths).expect("restore original");
    let restored = read_watch_status(&paths).expect("read restored status");
    assert_eq!(restored.status, WatchEventStatus::RestoredOriginal);
    assert_eq!(
        restored.clipboard_source,
        WatchClipboardSource::RestoredOriginal
    );
    assert!(restored.original_available);
    assert!(restored.event_id > updated.event_id);
}

#[test]
fn watch_status_records_errors_from_failed_clipboard_writes() {
    let clipboard = WriteFailClipboard::new("This is a wrapped\nparagraph from a terminal.\n");
    let paths = watch_paths("watch-state-error");

    let error = run_auto_clipboard_once(&watch_config(), &clipboard, &paths)
        .expect_err("clipboard write should fail");
    assert!(error.contains("failed to write clipboard"));

    let status = read_watch_status(&paths).expect("read error status");
    assert_eq!(status.status, WatchEventStatus::Error);
    assert!(status.message.contains("failed to write clipboard"));
    assert_eq!(status.mode, Some(Mode::Auto));
    assert!(!status.original_available);
}
