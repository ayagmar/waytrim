use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::clipboard::{ClipboardBackend, ClipboardError};
use crate::core::input_looks_like_reaction_snippet;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WatchEventStatus {
    #[default]
    Idle,
    Updated,
    Unchanged,
    Empty,
    Skipped,
    RestoredOriginal,
    MissingOriginal,
    Error,
}

impl WatchEventStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Updated => "updated",
            Self::Unchanged => "unchanged",
            Self::Empty => "empty",
            Self::Skipped => "skipped",
            Self::RestoredOriginal => "restored_original",
            Self::MissingOriginal => "missing_original",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WatchClipboardSource {
    #[default]
    Unknown,
    CleanedOutput,
    RestoredOriginal,
}

impl WatchClipboardSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::CleanedOutput => "cleaned_output",
            Self::RestoredOriginal => "restored_original",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WatchStatusSnapshot {
    pub mode: Option<Mode>,
    #[serde(default)]
    pub status: WatchEventStatus,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub original_available: bool,
    #[serde(default)]
    pub clipboard_source: WatchClipboardSource,
    pub updated_at_ms: Option<u64>,
    #[serde(default)]
    pub event_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct WatchState {
    #[serde(default)]
    skip_next_input: Option<String>,
    #[serde(default)]
    last_original_input: Option<String>,
    #[serde(default)]
    last_mode: Option<Mode>,
    #[serde(default)]
    last_status: WatchEventStatus,
    #[serde(default)]
    last_message: String,
    #[serde(default)]
    last_clipboard_source: WatchClipboardSource,
    #[serde(default)]
    last_updated_ms: Option<u64>,
    #[serde(default)]
    last_event_id: u64,
}

pub fn run_auto_clipboard_once<B: ClipboardBackend>(
    config: &AutoClipboardConfig,
    clipboard: &B,
    paths: &WatchPaths,
) -> Result<AutoClipboardOutput, String> {
    run_clipboard_once(config, clipboard, paths, true)
}

pub fn run_manual_clipboard_once<B: ClipboardBackend>(
    config: &AutoClipboardConfig,
    clipboard: &B,
    paths: &WatchPaths,
) -> Result<AutoClipboardOutput, String> {
    run_clipboard_once(config, clipboard, paths, false)
}

pub fn restore_last_original<B: ClipboardBackend>(
    clipboard: &B,
    paths: &WatchPaths,
) -> Result<AutoClipboardOutput, String> {
    match restore_last_original_inner(clipboard, paths) {
        Ok(output) => Ok(output),
        Err(message) => {
            let _ = record_watch_error(paths, None, &message);
            Err(message)
        }
    }
}

pub fn read_watch_status(paths: &WatchPaths) -> Result<WatchStatusSnapshot, String> {
    let state = load_state(&paths.state_path)?;
    Ok(state.snapshot())
}

pub fn write_watch_idle_status(paths: &WatchPaths, mode: Mode) -> Result<(), String> {
    let mut state = load_state(&paths.state_path)?;
    record_event(
        &mut state,
        Some(mode),
        WatchEventStatus::Idle,
        &format!("watcher started in {} mode", mode.as_str()),
        WatchClipboardSource::Unknown,
    );
    save_state(&paths.state_path, &state)
}

pub fn record_watch_error(
    paths: &WatchPaths,
    mode: Option<Mode>,
    message: &str,
) -> Result<(), String> {
    let mut state = load_state(&paths.state_path).unwrap_or_default();

    record_event(
        &mut state,
        mode,
        WatchEventStatus::Error,
        message,
        WatchClipboardSource::Unknown,
    );
    save_state(&paths.state_path, &state)
}

fn run_clipboard_once<B: ClipboardBackend>(
    config: &AutoClipboardConfig,
    clipboard: &B,
    paths: &WatchPaths,
    honor_skip_guard: bool,
) -> Result<AutoClipboardOutput, String> {
    match run_clipboard_once_inner(config, clipboard, paths, honor_skip_guard) {
        Ok(output) => Ok(output),
        Err(message) => {
            let _ = record_watch_error(paths, Some(config.mode), &message);
            Err(message)
        }
    }
}

fn run_clipboard_once_inner<B: ClipboardBackend>(
    config: &AutoClipboardConfig,
    clipboard: &B,
    paths: &WatchPaths,
    honor_skip_guard: bool,
) -> Result<AutoClipboardOutput, String> {
    let mut state = load_state(&paths.state_path)?;
    let input = match clipboard.read_text() {
        Ok(input) => input,
        Err(ClipboardError::NonText) => {
            record_event(
                &mut state,
                Some(config.mode),
                WatchEventStatus::Skipped,
                "clipboard did not contain text",
                WatchClipboardSource::Unknown,
            );
            save_state(&paths.state_path, &state)?;
            return Ok(output(
                AutoClipboardStatus::Skipped,
                "clipboard did not contain text",
            ));
        }
        Err(error) => return Err(format!("failed to read clipboard: {error}")),
    };

    if input.is_empty() {
        record_event(
            &mut state,
            Some(config.mode),
            WatchEventStatus::Empty,
            "clipboard is empty",
            WatchClipboardSource::Unknown,
        );
        save_state(&paths.state_path, &state)?;
        return Ok(output(AutoClipboardStatus::Empty, "clipboard is empty"));
    }

    if honor_skip_guard
        && state
            .skip_next_input
            .as_deref()
            .is_some_and(|saved| skip_guard_matches(saved, &input))
    {
        state.skip_next_input = None;
        record_event(
            &mut state,
            Some(config.mode),
            WatchEventStatus::Skipped,
            "skipped clipboard self-update",
            WatchClipboardSource::Unknown,
        );
        save_state(&paths.state_path, &state)?;
        return Ok(output(
            AutoClipboardStatus::Skipped,
            "skipped clipboard self-update",
        ));
    }

    let report = repair_report_with_policy(&input, config.mode, &config.policy);
    let newline_only_change = normalize_skip_guard_text(&input)
        == normalize_skip_guard_text(&report.output)
        && input != report.output;

    if !report.changed || newline_only_change && !input_looks_like_reaction_snippet(&input) {
        record_event(
            &mut state,
            Some(config.mode),
            WatchEventStatus::Unchanged,
            "clipboard unchanged",
            WatchClipboardSource::Unknown,
        );
        save_state(&paths.state_path, &state)?;
        return Ok(output(
            AutoClipboardStatus::Unchanged,
            "clipboard unchanged",
        ));
    }

    if let Err(error) = clipboard.write_text(&report.output) {
        return Err(format!("failed to write clipboard: {error}"));
    }

    state.skip_next_input = Some(report.output);
    state.last_original_input = Some(input);
    record_event(
        &mut state,
        Some(config.mode),
        WatchEventStatus::Updated,
        "clipboard updated",
        WatchClipboardSource::CleanedOutput,
    );
    save_state(&paths.state_path, &state)?;

    Ok(output(AutoClipboardStatus::Updated, "clipboard updated"))
}

fn restore_last_original_inner<B: ClipboardBackend>(
    clipboard: &B,
    paths: &WatchPaths,
) -> Result<AutoClipboardOutput, String> {
    let mut state = load_state(&paths.state_path)?;
    let Some(original) = state.last_original_input.clone() else {
        record_event(
            &mut state,
            None,
            WatchEventStatus::MissingOriginal,
            "no original clipboard text saved",
            WatchClipboardSource::Unknown,
        );
        save_state(&paths.state_path, &state)?;
        return Ok(output(
            AutoClipboardStatus::MissingOriginal,
            "no original clipboard text saved",
        ));
    };

    let previous_skip = state.skip_next_input.clone();
    state.skip_next_input = Some(original.clone());
    save_state(&paths.state_path, &state)?;

    if let Err(error) = clipboard.write_text(&original) {
        state.skip_next_input = previous_skip;
        let _ = save_state(&paths.state_path, &state);
        return Err(format!("failed to write clipboard: {error}"));
    }

    record_event(
        &mut state,
        None,
        WatchEventStatus::RestoredOriginal,
        "restored original clipboard text",
        WatchClipboardSource::RestoredOriginal,
    );
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

fn skip_guard_matches(saved: &str, input: &str) -> bool {
    saved == input || normalize_skip_guard_text(saved) == normalize_skip_guard_text(input)
}

fn normalize_skip_guard_text(value: &str) -> &str {
    value.trim_end_matches(['\r', '\n'])
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

fn record_event(
    state: &mut WatchState,
    mode: Option<Mode>,
    status: WatchEventStatus,
    message: &str,
    clipboard_source: WatchClipboardSource,
) {
    if let Some(mode) = mode {
        state.last_mode = Some(mode);
    }

    state.last_status = status;
    state.last_message = message.to_string();
    state.last_clipboard_source = clipboard_source;
    state.last_updated_ms = Some(current_time_ms());
    state.last_event_id = state.last_event_id.saturating_add(1);
}

fn current_time_ms() -> u64 {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_millis();

    millis.try_into().unwrap_or(u64::MAX)
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

impl WatchState {
    fn snapshot(&self) -> WatchStatusSnapshot {
        WatchStatusSnapshot {
            mode: self.last_mode,
            status: self.last_status,
            message: self.last_message.clone(),
            original_available: self.last_original_input.is_some(),
            clipboard_source: self.last_clipboard_source,
            updated_at_ms: self.last_updated_ms,
            event_id: self.last_event_id,
        }
    }
}
