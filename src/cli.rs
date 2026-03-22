use crate::clipboard::ClipboardBackend;
use crate::{Mode, RepairPolicy, render_explain, render_preview, repair_with_policy};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CliArgs {
    pub mode: Option<Mode>,
    pub clipboard: Option<bool>,
    pub preview: Option<bool>,
    pub explain: Option<bool>,
    pub print: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigDefaults {
    pub mode: Mode,
    pub clipboard: bool,
    pub preview: bool,
    pub explain: bool,
    pub print: bool,
    pub policy: RepairPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliConfig {
    pub mode: Mode,
    pub clipboard: bool,
    pub preview: bool,
    pub explain: bool,
    pub print: bool,
    pub policy: RepairPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardFlowStatus {
    Updated,
    Unchanged,
    Preview,
    Explain,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardFlowOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: ClipboardFlowStatus,
}

impl Default for ConfigDefaults {
    fn default() -> Self {
        Self {
            mode: Mode::Prose,
            clipboard: false,
            preview: false,
            explain: false,
            print: false,
            policy: RepairPolicy::default(),
        }
    }
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            mode: Mode::Prose,
            clipboard: false,
            preview: false,
            explain: false,
            print: false,
            policy: RepairPolicy::default(),
        }
    }
}

impl CliArgs {
    pub fn parse<I, S>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut parsed = Self::default();

        for arg in args {
            match arg.as_ref() {
                "prose" => parsed.mode = Some(Mode::Prose),
                "command" => parsed.mode = Some(Mode::Command),
                "auto" => parsed.mode = Some(Mode::Auto),
                "--clipboard" => parsed.clipboard = Some(true),
                "--no-clipboard" => parsed.clipboard = Some(false),
                "--preview" => parsed.preview = Some(true),
                "--no-preview" => parsed.preview = Some(false),
                "--explain" => parsed.explain = Some(true),
                "--no-explain" => parsed.explain = Some(false),
                "--print" => parsed.print = Some(true),
                "--no-print" => parsed.print = Some(false),
                other => return Err(format!("unknown argument: {other}")),
            }
        }

        Ok(parsed)
    }
}

impl CliConfig {
    pub fn parse<I, S>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self::resolve(CliArgs::parse(args)?, ConfigDefaults::default())
    }

    pub fn resolve(args: CliArgs, defaults: ConfigDefaults) -> Result<Self, String> {
        let config = Self {
            mode: args.mode.unwrap_or(defaults.mode),
            clipboard: args.clipboard.unwrap_or(defaults.clipboard),
            preview: args.preview.unwrap_or(defaults.preview),
            explain: args.explain.unwrap_or(defaults.explain),
            print: args.print.unwrap_or(defaults.print),
            policy: defaults.policy,
        };

        config.validate()
    }

    fn validate(self) -> Result<Self, String> {
        if self.preview && self.explain {
            return Err(String::from("cannot combine --preview and --explain"));
        }

        if self.clipboard && self.preview && self.print {
            return Err(String::from(
                "cannot combine --preview and --print with --clipboard",
            ));
        }

        if self.clipboard && self.explain && self.print {
            return Err(String::from(
                "cannot combine --explain and --print with --clipboard",
            ));
        }

        Ok(self)
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

    let result = repair_with_policy(&input, config.mode, &config.policy);

    if config.preview {
        return Ok(ClipboardFlowOutput {
            stdout: render_preview(&input, &result),
            stderr: String::from("clipboard preview only; nothing was written\n"),
            status: ClipboardFlowStatus::Preview,
        });
    }

    if config.explain {
        return Ok(ClipboardFlowOutput {
            stdout: render_explain(config.mode, &result),
            stderr: String::from("clipboard explain only; nothing was written\n"),
            status: ClipboardFlowStatus::Explain,
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
