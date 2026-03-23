use serde::{Deserialize, Serialize};

use super::policy::Mode;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplainStep {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepairResult {
    pub output: String,
    pub changed: bool,
    pub explain: Vec<ExplainStep>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepairDecision {
    RequestedMode,
    AutoCommand,
    AutoProse,
    AutoMinimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepairReport {
    pub requested_mode: Mode,
    pub effective_mode: Mode,
    pub decision: RepairDecision,
    pub output: String,
    pub changed: bool,
    pub explain: Vec<ExplainStep>,
}

impl From<RepairReport> for RepairResult {
    fn from(report: RepairReport) -> Self {
        Self {
            output: report.output,
            changed: report.changed,
            explain: report.explain,
        }
    }
}

pub(crate) struct RepairOutcome {
    pub effective_mode: Mode,
    pub decision: RepairDecision,
    pub output: String,
    pub explain: Vec<ExplainStep>,
}

impl RepairOutcome {
    pub(crate) fn new(
        effective_mode: Mode,
        decision: RepairDecision,
        output: String,
        explain: Vec<ExplainStep>,
    ) -> Self {
        Self {
            effective_mode,
            decision,
            output,
            explain,
        }
    }
}
