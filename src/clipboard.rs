use std::fmt;
use std::fs;
use std::process::{self, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const WRITE_FAILURE_POLL_ATTEMPTS: usize = 20;
const WRITE_FAILURE_POLL_INTERVAL: Duration = Duration::from_millis(10);

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
    list_types_command: Option<CommandSpec>,
    preferred_text_types: Option<Vec<String>>,
}

impl SystemClipboard {
    pub fn new() -> Self {
        Self::with_commands_and_text_types_and_type_list(
            CommandSpec::new("wl-paste"),
            CommandSpec::new("wl-copy"),
            Some(CommandSpec::new("wl-paste").with_arg("--list-types")),
            Some(default_preferred_text_types()),
        )
    }

    pub fn with_commands(read_command: CommandSpec, write_command: CommandSpec) -> Self {
        Self::with_commands_and_text_types_and_type_list(read_command, write_command, None, None)
    }

    pub fn with_commands_and_text_types(
        read_command: CommandSpec,
        write_command: CommandSpec,
        preferred_text_types: Option<Vec<String>>,
    ) -> Self {
        Self::with_commands_and_text_types_and_type_list(
            read_command,
            write_command,
            None,
            preferred_text_types,
        )
    }

    pub fn with_commands_and_text_types_and_type_list(
        read_command: CommandSpec,
        write_command: CommandSpec,
        list_types_command: Option<CommandSpec>,
        preferred_text_types: Option<Vec<String>>,
    ) -> Self {
        Self {
            read_command,
            write_command,
            list_types_command,
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
            if let Some(list_types_command) = &self.list_types_command {
                if let Ok(offered_types) = list_offered_types(list_types_command) {
                    if !offered_types.is_empty() {
                        if let Some(text_type) =
                            preferred_offered_text_type(&offered_types, preferred_text_types)
                        {
                            let extra_args = [String::from("--type"), text_type];
                            match run_command(&self.read_command, &extra_args) {
                                Ok(output) => {
                                    return String::from_utf8(output.stdout)
                                        .map_err(|_| ClipboardError::NonText);
                                }
                                Err(ClipboardError::CommandFailed { detail, .. })
                                    if requested_type_is_unavailable(&detail) => {}
                                Err(error) => return Err(error),
                            }
                        }

                        if !clipboard_offers_text(&offered_types, preferred_text_types) {
                            return Err(ClipboardError::NonText);
                        }

                        if !clipboard_offers_only_text(&offered_types, preferred_text_types) {
                            return Err(ClipboardError::NonText);
                        }
                    }
                }
            }

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

        for _ in 0..WRITE_FAILURE_POLL_ATTEMPTS {
            let Some(status) = child
                .try_wait()
                .map_err(|error| ClipboardError::CommandFailed {
                    command: self.write_command.program.clone(),
                    detail: error.to_string(),
                })?
            else {
                thread::sleep(WRITE_FAILURE_POLL_INTERVAL);
                continue;
            };

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

            break;
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

fn list_offered_types(list_types_command: &CommandSpec) -> Result<Vec<String>, ClipboardError> {
    let output = run_command(list_types_command, &[])?;
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

fn preferred_offered_text_type(
    offered_types: &[String],
    preferred_text_types: &[String],
) -> Option<String> {
    for preferred in preferred_text_types {
        if let Some(matched) = offered_types
            .iter()
            .find(|value| value.eq_ignore_ascii_case(preferred))
        {
            return Some(matched.clone());
        }
    }

    offered_types
        .iter()
        .find(|value| value.to_ascii_lowercase().starts_with("text/plain;"))
        .cloned()
}

fn clipboard_offers_text(offered_types: &[String], preferred_text_types: &[String]) -> bool {
    offered_types
        .iter()
        .any(|offered| text_offer_kind(offered, preferred_text_types).is_some())
}

fn clipboard_offers_only_text(offered_types: &[String], preferred_text_types: &[String]) -> bool {
    offered_types
        .iter()
        .all(|offered| text_offer_kind(offered, preferred_text_types).is_some())
}

fn text_offer_kind(offered: &str, preferred_text_types: &[String]) -> Option<()> {
    let preferred_set = preferred_text_types
        .iter()
        .map(|value| value.to_ascii_lowercase())
        .collect::<Vec<_>>();

    let lower = offered.to_ascii_lowercase();

    (preferred_set.iter().any(|preferred| preferred == &lower)
        || lower.starts_with("text/")
        || lower.starts_with("text;")
        || lower.starts_with("text/plain;")
        || matches!(lower.as_str(), "utf8_string" | "string" | "text")
        || is_textual_application_offer(&lower))
    .then_some(())
}

fn is_textual_application_offer(lower: &str) -> bool {
    matches!(
        lower,
        "application/json"
            | "application/x-json"
            | "application/yaml"
            | "application/x-yaml"
            | "application/toml"
            | "application/x-toml"
            | "application/xml"
    ) || lower.ends_with("+json")
        || lower.ends_with("+yaml")
        || lower.ends_with("+xml")
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
