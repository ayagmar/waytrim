use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::clipboard::ClipboardBackend;
use crate::{Mode, RepairPolicy, default_runtime_dir, repair_report_with_policy};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoClipboardConfig {
    pub mode: Mode,
    pub policy: RepairPolicy,
}

impl Default for AutoClipboardConfig {
    fn default() -> Self {
        Self {
            mode: Mode::Auto,
            policy: RepairPolicy::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchPaths {
    pub state_path: PathBuf,
}

impl Default for WatchPaths {
    fn default() -> Self {
        Self {
            state_path: default_runtime_dir().join("watch-state.json"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoClipboardStatus {
    Updated,
    Unchanged,
    Empty,
    Skipped,
    RestoredOriginal,
    MissingOriginal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoClipboardOutput {
    pub status: AutoClipboardStatus,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct WatchState {
    #[serde(default)]
    skip_next_input: Option<String>,
    #[serde(default)]
    last_original_input: Option<String>,
}

pub fn run_auto_clipboard_once<B: ClipboardBackend>(
    config: &AutoClipboardConfig,
    clipboard: &B,
    paths: &WatchPaths,
) -> Result<AutoClipboardOutput, String> {
    let input = clipboard
        .read_text()
        .map_err(|error| format!("failed to read clipboard: {error}"))?;

    if input.is_empty() {
        return Ok(output(AutoClipboardStatus::Empty, "clipboard is empty"));
    }

    let mut state = load_state(&paths.state_path)?;
    if state.skip_next_input.as_deref() == Some(input.as_str()) {
        state.skip_next_input = None;
        save_state(&paths.state_path, &state)?;
        return Ok(output(
            AutoClipboardStatus::Skipped,
            "skipped clipboard self-update",
        ));
    }

    let report = repair_report_with_policy(&input, config.mode, &config.policy);
    if !report.changed {
        return Ok(output(
            AutoClipboardStatus::Unchanged,
            "clipboard unchanged",
        ));
    }

    clipboard
        .write_text(&report.output)
        .map_err(|error| format!("failed to write clipboard: {error}"))?;

    state.skip_next_input = Some(report.output);
    state.last_original_input = Some(input);
    save_state(&paths.state_path, &state)?;

    Ok(output(AutoClipboardStatus::Updated, "clipboard updated"))
}

pub fn restore_last_original<B: ClipboardBackend>(
    clipboard: &B,
    paths: &WatchPaths,
) -> Result<AutoClipboardOutput, String> {
    let mut state = load_state(&paths.state_path)?;
    let Some(original) = state.last_original_input.clone() else {
        return Ok(output(
            AutoClipboardStatus::MissingOriginal,
            "no original clipboard text saved",
        ));
    };

    clipboard
        .write_text(&original)
        .map_err(|error| format!("failed to write clipboard: {error}"))?;

    state.skip_next_input = Some(original);
    save_state(&paths.state_path, &state)?;

    Ok(output(
        AutoClipboardStatus::RestoredOriginal,
        "restored original clipboard text",
    ))
}

fn output(status: AutoClipboardStatus, message: &str) -> AutoClipboardOutput {
    AutoClipboardOutput {
        status,
        message: format!("{message}\n"),
    }
}

fn load_state(path: &Path) -> Result<WatchState, String> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(WatchState::default());
        }
        Err(error) => {
            return Err(format!(
                "failed to read watch state {}: {error}",
                path.display()
            ));
        }
    };

    serde_json::from_str(&contents)
        .map_err(|error| format!("failed to parse watch state {}: {error}", path.display()))
}

fn save_state(path: &Path, state: &WatchState) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Err(String::from("watch state path had no parent directory"));
    };

    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "failed to create watch state dir {}: {error}",
            parent.display()
        )
    })?;

    let temp_path = temp_state_path(path);
    let contents = serde_json::to_vec(state)
        .map_err(|error| format!("failed to encode watch state {}: {error}", path.display()))?;

    fs::write(&temp_path, contents).map_err(|error| {
        format!(
            "failed to write watch state temp file {}: {error}",
            temp_path.display()
        )
    })?;

    fs::rename(&temp_path, path).map_err(|error| {
        let _ = fs::remove_file(&temp_path);
        format!("failed to replace watch state {}: {error}", path.display())
    })
}

fn temp_state_path(path: &Path) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();

    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| String::from("watch-state.json"));

    path.with_file_name(format!(
        ".{file_name}.tmp-{}-{timestamp}-{unique}",
        process::id()
    ))
}
