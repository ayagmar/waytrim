use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Prose,
    Command,
    Auto,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Prose => "prose",
            Self::Command => "command",
            Self::Auto => "auto",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoPolicy {
    Conservative,
    ProsePreferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepairPolicy {
    pub protect_aligned_columns: bool,
    pub protect_command_blocks: bool,
    pub auto_policy: AutoPolicy,
}

impl Default for RepairPolicy {
    fn default() -> Self {
        Self {
            protect_aligned_columns: true,
            protect_command_blocks: true,
            auto_policy: AutoPolicy::Conservative,
        }
    }
}
