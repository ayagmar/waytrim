use crate::clipboard::ClipboardBackend;
use crate::{Mode, render_preview, repair};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliConfig {
    pub mode: Mode,
    pub clipboard: bool,
    pub preview: bool,
    pub print: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardFlowStatus {
    Updated,
    Unchanged,
    Preview,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardFlowOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: ClipboardFlowStatus,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            mode: Mode::Prose,
            clipboard: false,
            preview: false,
            print: false,
        }
    }
}

impl CliConfig {
    pub fn parse<I, S>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut config = Self::default();

        for arg in args {
            match arg.as_ref() {
                "prose" => config.mode = Mode::Prose,
                "command" => config.mode = Mode::Command,
                "auto" => config.mode = Mode::Auto,
                "--clipboard" => config.clipboard = true,
                "--preview" => config.preview = true,
                "--print" => config.print = true,
                other => return Err(format!("unknown argument: {other}")),
            }
        }

        if config.clipboard && config.preview && config.print {
            return Err(String::from(
                "cannot combine --preview and --print with --clipboard",
            ));
        }

        Ok(config)
    }
}

pub fn run_clipboard_flow<B: ClipboardBackend>(
    config: &CliConfig,
    clipboard: &B,
) -> Result<ClipboardFlowOutput, String> {
    let input = clipboard
        .read_text()
        .map_err(|error| format!("failed to read clipboard: {error}"))?;

    if input.is_empty() {
        return Ok(ClipboardFlowOutput {
            stdout: String::new(),
            stderr: String::from("clipboard is empty\n"),
            status: ClipboardFlowStatus::Empty,
        });
    }

    let result = repair(&input, config.mode);

    if config.preview {
        return Ok(ClipboardFlowOutput {
            stdout: render_preview(&input, &result),
            stderr: String::from("clipboard preview only; nothing was written\n"),
            status: ClipboardFlowStatus::Preview,
        });
    }

    let stdout = if config.print {
        result.output.clone()
    } else {
        String::new()
    };

    if !result.changed {
        return Ok(ClipboardFlowOutput {
            stdout,
            stderr: String::from("clipboard unchanged\n"),
            status: ClipboardFlowStatus::Unchanged,
        });
    }

    clipboard
        .write_text(&result.output)
        .map_err(|error| format!("failed to write clipboard: {error}"))?;

    Ok(ClipboardFlowOutput {
        stdout,
        stderr: String::from("clipboard updated\n"),
        status: ClipboardFlowStatus::Updated,
    })
}
