use std::env;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use crate::cli::ConfigDefaults;
use crate::{AutoPolicy, Mode};

#[derive(Debug, Default, Deserialize)]
pub struct FileConfig {
    #[serde(default)]
    pub defaults: DefaultsSection,
    #[serde(default)]
    pub protect: ProtectSection,
    #[serde(default, rename = "auto")]
    pub auto_section: AutoSection,
}

#[derive(Debug, Default, Deserialize)]
pub struct DefaultsSection {
    pub mode: Option<String>,
    pub clipboard: Option<bool>,
    pub preview: Option<bool>,
    pub explain: Option<bool>,
    pub print: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ProtectSection {
    pub aligned_columns: Option<bool>,
    pub command_blocks: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
pub struct AutoSection {
    pub policy: Option<String>,
}

pub fn load_user_defaults() -> (ConfigDefaults, Option<String>) {
    let defaults = ConfigDefaults::default();
    let Some(path) = user_config_path() else {
        return (defaults, None);
    };

    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return (defaults, None),
        Err(error) => {
            return (
                defaults,
                Some(format!(
                    "warning: failed to load config {}: {error}",
                    path.display()
                )),
            );
        }
    };

    match parse_file_config(&contents) {
        Ok(file) => match apply_file_config(defaults, file) {
            Ok(applied) => (applied, None),
            Err(error) => (
                ConfigDefaults::default(),
                Some(format!(
                    "warning: failed to load config {}: {error}",
                    path.display()
                )),
            ),
        },
        Err(error) => (
            ConfigDefaults::default(),
            Some(format!(
                "warning: failed to load config {}: {error}",
                path.display()
            )),
        ),
    }
}

fn parse_file_config(contents: &str) -> Result<FileConfig, toml::de::Error> {
    toml::from_str(contents)
}

fn apply_file_config(
    mut defaults: ConfigDefaults,
    file: FileConfig,
) -> Result<ConfigDefaults, String> {
    if let Some(mode) = file.defaults.mode {
        defaults.mode = parse_mode(&mode)?;
    }

    if let Some(clipboard) = file.defaults.clipboard {
        defaults.clipboard = clipboard;
    }

    if let Some(preview) = file.defaults.preview {
        defaults.preview = preview;
    }

    if let Some(explain) = file.defaults.explain {
        defaults.explain = explain;
    }

    if let Some(print) = file.defaults.print {
        defaults.print = print;
    }

    if let Some(aligned_columns) = file.protect.aligned_columns {
        defaults.policy.protect_aligned_columns = aligned_columns;
    }

    if let Some(command_blocks) = file.protect.command_blocks {
        defaults.policy.protect_command_blocks = command_blocks;
    }

    if let Some(policy) = file.auto_section.policy {
        defaults.policy.auto_policy = parse_auto_policy(&policy)?;
    }

    Ok(defaults)
}

fn parse_mode(value: &str) -> Result<Mode, String> {
    match value {
        "prose" => Ok(Mode::Prose),
        "command" => Ok(Mode::Command),
        "auto" => Ok(Mode::Auto),
        other => Err(format!("invalid mode: {other}")),
    }
}

fn parse_auto_policy(value: &str) -> Result<AutoPolicy, String> {
    match value {
        "conservative" => Ok(AutoPolicy::Conservative),
        "prose_preferred" => Ok(AutoPolicy::ProsePreferred),
        other => Err(format!("invalid auto policy: {other}")),
    }
}

fn user_config_path() -> Option<PathBuf> {
    if let Some(config_home) = env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(config_home).join("waytrim/config.toml"));
    }

    let home = env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".config/waytrim/config.toml"))
}
