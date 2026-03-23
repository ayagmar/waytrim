pub mod cli;
pub mod clipboard;
pub mod config;
pub mod core;
pub mod ipc;
pub mod service;

pub use core::{
    AutoPolicy, ExplainStep, Mode, RepairDecision, RepairPolicy, RepairReport, RepairResult,
    render_explain, render_preview, repair, repair_report, repair_report_with_policy,
    repair_with_policy,
};
pub use ipc::{IPC_VERSION, IpcRequest, IpcResponse, default_socket_path};
pub use service::{ServiceConfig, run_service};
