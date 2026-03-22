mod support;

use std::cell::RefCell;

use support::{temp_file_path, write_executable_script};
use waytrim::clipboard::{ClipboardBackend, CommandSpec, ClipboardError, SystemClipboard};

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

    let error = clipboard.read_text().expect_err("expected missing command error");

    assert!(error.to_string().contains("command not found"));
    assert!(error.to_string().contains("waytrim-missing-wl-paste"));
}

#[test]
fn system_backend_reports_invalid_utf8_reads_clearly() {
    let script = write_executable_script(
        "clipboard-invalid-utf8",
        "#!/bin/sh\nprintf '\\377'\n",
    );
    let clipboard = SystemClipboard::with_commands(
        CommandSpec::new(script.to_string_lossy()).with_arg("--"),
        CommandSpec::new("waytrim-missing-wl-copy"),
    );

    let error = clipboard.read_text().expect_err("expected utf8 error");

    assert!(error.to_string().contains("clipboard did not contain valid UTF-8 text"));
}

#[test]
fn system_backend_writes_text_through_configured_command() {
    let output_path = temp_file_path("clipboard-write-output");
    let script = write_executable_script(
        "clipboard-write",
        &format!("#!/bin/sh\ncat > '{}'\n", output_path.display()),
    );
    let clipboard = SystemClipboard::with_commands(
        CommandSpec::new("waytrim-missing-wl-paste"),
        CommandSpec::new(script.to_string_lossy()).with_arg("--"),
    );

    clipboard.write_text("copied text\n").expect("write clipboard");

    let written = std::fs::read_to_string(output_path).expect("read written clipboard text");
    assert_eq!(written, "copied text\n");
}
