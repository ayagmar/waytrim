use std::fmt;
use std::fs;
use std::process::{self, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
    preferred_text_types: Option<Vec<String>>,
}

impl SystemClipboard {
    pub fn new() -> Self {
        Self::with_commands_and_text_types(
            CommandSpec::new("wl-paste"),
            CommandSpec::new("wl-copy"),
            Some(default_preferred_text_types()),
        )
    }

    pub fn with_commands(read_command: CommandSpec, write_command: CommandSpec) -> Self {
        Self::with_commands_and_text_types(read_command, write_command, None)
    }

    pub fn with_commands_and_text_types(
        read_command: CommandSpec,
        write_command: CommandSpec,
        preferred_text_types: Option<Vec<String>>,
    ) -> Self {
        Self {
            read_command,
            write_command,
            preferred_text_types,
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
        if let Some(preferred_text_types) = &self.preferred_text_types {
            for text_type in preferred_text_types {
                let extra_args = [String::from("--type"), text_type.clone()];
                match run_command(&self.read_command, &extra_args) {
                    Ok(output) => {
                        return String::from_utf8(output.stdout)
                            .map_err(|_| ClipboardError::NonText);
                    }
                    Err(ClipboardError::CommandFailed { detail, .. })
                        if requested_type_is_unavailable(&detail) =>
                    {
                        continue;
                    }
                    Err(error) => return Err(error),
                }
            }

            return Err(ClipboardError::NonText);
        }

        let output = run_command(&self.read_command, &[])?;
        String::from_utf8(output.stdout).map_err(|_| ClipboardError::NonText)
    }

    fn write_text(&self, text: &str) -> Result<(), ClipboardError> {
        let input_path =
            write_clipboard_input(text).map_err(|error| ClipboardError::CommandFailed {
                command: self.write_command.program.clone(),
                detail: error.to_string(),
            })?;
        let input_file =
            fs::File::open(&input_path).map_err(|error| ClipboardError::CommandFailed {
                command: self.write_command.program.clone(),
                detail: error.to_string(),
            })?;

        let mut child = Command::new(&self.write_command.program)
            .args(&self.write_command.args)
            .stdin(Stdio::from(input_file))
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

        let _ = fs::remove_file(&input_path);
        thread::sleep(Duration::from_millis(5));

        if let Some(status) = child
            .try_wait()
            .map_err(|error| ClipboardError::CommandFailed {
                command: self.write_command.program.clone(),
                detail: error.to_string(),
            })?
        {
            let output =
                child
                    .wait_with_output()
                    .map_err(|error| ClipboardError::CommandFailed {
                        command: self.write_command.program.clone(),
                        detail: error.to_string(),
                    })?;

            if !status.success() {
                return Err(ClipboardError::CommandFailed {
                    command: self.write_command.program.clone(),
                    detail: String::from_utf8_lossy(&output.stderr).trim().to_string(),
                });
            }
        }

        Ok(())
    }
}

fn run_command(
    spec: &CommandSpec,
    extra_args: &[String],
) -> Result<std::process::Output, ClipboardError> {
    let output = Command::new(&spec.program)
        .args(&spec.args)
        .args(extra_args)
        .output()
        .map_err(|error| match error.kind() {
            std::io::ErrorKind::NotFound => ClipboardError::CommandNotFound {
                command: spec.program.clone(),
            },
            _ => ClipboardError::CommandFailed {
                command: spec.program.clone(),
                detail: error.to_string(),
            },
        })?;

    if !output.status.success() {
        return Err(ClipboardError::CommandFailed {
            command: spec.program.clone(),
            detail: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }

    Ok(output)
}

fn default_preferred_text_types() -> Vec<String> {
    [
        "text/plain;charset=utf-8",
        "text/plain;charset=utf8",
        "text/plain",
        "UTF8_STRING",
        "STRING",
        "TEXT",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn requested_type_is_unavailable(detail: &str) -> bool {
    detail
        .to_ascii_lowercase()
        .contains("clipboard content is not available as requested type")
}

fn write_clipboard_input(text: &str) -> std::io::Result<std::path::PathBuf> {
    let path = clipboard_input_path();
    fs::write(&path, text)?;
    Ok(path)
}

fn clipboard_input_path() -> std::path::PathBuf {
    temp_path("clipboard-input")
}

fn temp_path(kind: &str) -> std::path::PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();

    std::env::temp_dir().join(format!(
        "waytrim-{kind}-{}-{timestamp}-{unique}",
        process::id()
    ))
}
