mod support;

use std::cell::RefCell;
use std::time::{Duration, Instant};

use support::temp_file_path;
use waytrim::clipboard::{ClipboardBackend, ClipboardError, CommandSpec, SystemClipboard};

struct MemoryClipboard {
    value: RefCell<String>,
}

impl MemoryClipboard {
    fn new(initial: &str) -> Self {
        Self {
            value: RefCell::new(initial.to_string()),
        }
    }
}

impl ClipboardBackend for MemoryClipboard {
    fn read_text(&self) -> Result<String, ClipboardError> {
        Ok(self.value.borrow().clone())
    }

    fn write_text(&self, text: &str) -> Result<(), ClipboardError> {
        *self.value.borrow_mut() = text.to_string();
        Ok(())
    }
}

#[test]
fn fake_backend_reads_and_writes_text_without_shelling_out() {
    let clipboard = MemoryClipboard::new("before");

    assert_eq!(clipboard.read_text().expect("read clipboard"), "before");
    clipboard.write_text("after").expect("write clipboard");
    assert_eq!(clipboard.read_text().expect("read clipboard"), "after");
}

#[test]
fn system_backend_reports_missing_commands_clearly() {
    let clipboard = SystemClipboard::with_commands(
        CommandSpec::new("waytrim-missing-wl-paste"),
        CommandSpec::new("waytrim-missing-wl-copy"),
    );

    let error = clipboard
        .read_text()
        .expect_err("expected missing command error");

    assert!(error.to_string().contains("command not found"));
    assert!(error.to_string().contains("waytrim-missing-wl-paste"));
}

#[test]
fn system_backend_reports_invalid_utf8_reads_clearly() {
    let clipboard = SystemClipboard::with_commands(
        CommandSpec::new("sh")
            .with_arg("-c")
            .with_arg("printf '\\377'"),
        CommandSpec::new("waytrim-missing-wl-copy"),
    );

    let error = clipboard.read_text().expect_err("expected utf8 error");

    assert!(
        error
            .to_string()
            .contains("clipboard did not contain valid UTF-8 text")
    );
}

#[test]
fn system_backend_skips_non_text_clipboard_offers_without_reading_payload() {
    let read_marker_path = temp_file_path("clipboard-read-marker");
    let clipboard = SystemClipboard::with_commands_and_type_list(
        CommandSpec::new("sh").with_arg("-c").with_arg(format!(
            "touch '{}'; sleep 2; printf 'should not be read'",
            read_marker_path.display()
        )),
        CommandSpec::new("waytrim-missing-wl-copy"),
        Some(
            CommandSpec::new("sh")
                .with_arg("-c")
                .with_arg("printf 'image/png\nimage/jpeg\n'"),
        ),
    );

    let start = Instant::now();
    let error = clipboard.read_text().expect_err("expected non-text error");

    assert!(matches!(error, ClipboardError::NonText));
    assert!(
        start.elapsed() < Duration::from_secs(1),
        "non-text probe took too long: {:?}",
        start.elapsed()
    );
    assert!(
        !read_marker_path.exists(),
        "clipboard payload should not be read"
    );
}

#[test]
fn system_backend_prefers_plain_text_type_before_reading_payload() {
    let clipboard = SystemClipboard::with_commands_and_type_list(
        CommandSpec::new("sh")
            .with_arg("-c")
            .with_arg(
                "if [ \"$1\" = \"--type\" ] && [ \"$2\" = \"text/plain;charset=utf-8\" ]; then printf 'hello'; else echo 'wrong type' >&2; exit 1; fi",
            )
            .with_arg("sh"),
        CommandSpec::new("waytrim-missing-wl-copy"),
        Some(
            CommandSpec::new("sh")
                .with_arg("-c")
                .with_arg("printf 'image/png\ntext/plain;charset=utf-8\n'"),
        ),
    );

    assert_eq!(clipboard.read_text().expect("read clipboard"), "hello");
}

#[test]
fn system_backend_writes_text_through_configured_command() {
    let output_path = temp_file_path("clipboard-write-output");
    let clipboard = SystemClipboard::with_commands(
        CommandSpec::new("waytrim-missing-wl-paste"),
        CommandSpec::new("sh")
            .with_arg("-c")
            .with_arg(format!("cat > '{}'", output_path.display())),
    );

    clipboard
        .write_text("copied text\n")
        .expect("write clipboard");

    let written = std::fs::read_to_string(output_path).expect("read written clipboard text");
    assert_eq!(written, "copied text\n");
}

#[test]
fn system_backend_can_write_with_file_backed_stdin() {
    let output_path = temp_file_path("clipboard-write-file-stdin-output");
    let clipboard = SystemClipboard::with_commands(
        CommandSpec::new("waytrim-missing-wl-paste"),
        CommandSpec::new("sh").with_arg("-c").with_arg(format!(
            "if [ -p /dev/stdin ]; then echo 'stdin must not be a pipe' >&2; exit 1; fi; cat > '{}'",
            output_path.display()
        )),
    );

    clipboard
        .write_text("copied through file stdin\n")
        .expect("write clipboard");

    let written = std::fs::read_to_string(output_path).expect("read written clipboard text");
    assert_eq!(written, "copied through file stdin\n");
}

#[test]
fn system_backend_does_not_wait_for_long_lived_clipboard_process() {
    let output_path = temp_file_path("clipboard-write-long-lived-output");
    let clipboard = SystemClipboard::with_commands(
        CommandSpec::new("waytrim-missing-wl-paste"),
        CommandSpec::new("sh")
            .with_arg("-c")
            .with_arg(format!("cat > '{}'; sleep 2", output_path.display())),
    );

    let start = Instant::now();
    clipboard
        .write_text("copied without waiting\n")
        .expect("write clipboard");

    assert!(
        start.elapsed() < Duration::from_secs(1),
        "clipboard write waited too long: {:?}",
        start.elapsed()
    );
}
