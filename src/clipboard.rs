use std::fmt;
use std::io::Write;
use std::process::{Command, Stdio};

pub trait ClipboardBackend {
    fn read_text(&self) -> Result<String, ClipboardError>;
    fn write_text(&self, text: &str) -> Result<(), ClipboardError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardError {
    CommandNotFound { command: String },
    CommandFailed { command: String, detail: String },
    NonText,
}

impl fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommandNotFound { command } => {
                write!(f, "clipboard command not found: {command}")
            }
            Self::CommandFailed { command, detail } => {
                write!(f, "clipboard command failed: {command}: {detail}")
            }
            Self::NonText => write!(f, "clipboard did not contain valid UTF-8 text"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    program: String,
    args: Vec<String>,
}

impl CommandSpec {
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
        }
    }

    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemClipboard {
    read_command: CommandSpec,
    write_command: CommandSpec,
}

impl SystemClipboard {
    pub fn new() -> Self {
        Self::with_commands(CommandSpec::new("wl-paste"), CommandSpec::new("wl-copy"))
    }

    pub fn with_commands(read_command: CommandSpec, write_command: CommandSpec) -> Self {
        Self {
            read_command,
            write_command,
        }
    }
}

impl Default for SystemClipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardBackend for SystemClipboard {
    fn read_text(&self) -> Result<String, ClipboardError> {
        let output = Command::new(&self.read_command.program)
            .args(&self.read_command.args)
            .output()
            .map_err(|error| match error.kind() {
                std::io::ErrorKind::NotFound => ClipboardError::CommandNotFound {
                    command: self.read_command.program.clone(),
                },
                _ => ClipboardError::CommandFailed {
                    command: self.read_command.program.clone(),
                    detail: error.to_string(),
                },
            })?;

        if !output.status.success() {
            return Err(ClipboardError::CommandFailed {
                command: self.read_command.program.clone(),
                detail: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            });
        }

        String::from_utf8(output.stdout).map_err(|_| ClipboardError::NonText)
    }

    fn write_text(&self, text: &str) -> Result<(), ClipboardError> {
        let mut child = Command::new(&self.write_command.program)
            .args(&self.write_command.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| match error.kind() {
                std::io::ErrorKind::NotFound => ClipboardError::CommandNotFound {
                    command: self.write_command.program.clone(),
                },
                _ => ClipboardError::CommandFailed {
                    command: self.write_command.program.clone(),
                    detail: error.to_string(),
                },
            })?;

        child
            .stdin
            .as_mut()
            .expect("clipboard command stdin")
            .write_all(text.as_bytes())
            .map_err(|error| ClipboardError::CommandFailed {
                command: self.write_command.program.clone(),
                detail: error.to_string(),
            })?;

        let output = child.wait_with_output().map_err(|error| ClipboardError::CommandFailed {
            command: self.write_command.program.clone(),
            detail: error.to_string(),
        })?;

        if output.status.success() {
            return Ok(());
        }

        Err(ClipboardError::CommandFailed {
            command: self.write_command.program.clone(),
            detail: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        })
    }
}
