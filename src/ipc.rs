use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::Shutdown;
use std::os::unix::fs::MetadataExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{Mode, RepairPolicy, RepairReport};

pub const IPC_VERSION: u32 = 1;
const SOCKET_DIR_NAME: &str = "waytrim";
const SOCKET_FILE_NAME: &str = "waytrim.sock";

pub fn default_socket_path() -> PathBuf {
    if let Some(runtime_dir) = env::var_os("XDG_RUNTIME_DIR") {
        return PathBuf::from(runtime_dir)
            .join(SOCKET_DIR_NAME)
            .join(SOCKET_FILE_NAME);
    }

    env::temp_dir()
        .join(format!("waytrim-{}", fallback_socket_namespace()))
        .join(SOCKET_FILE_NAME)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IpcRequest {
    Repair {
        version: u32,
        mode: Mode,
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        policy: Option<RepairPolicy>,
    },
    Shutdown {
        version: u32,
    },
}

impl IpcRequest {
    pub fn repair(mode: Mode, text: impl Into<String>) -> Self {
        Self::Repair {
            version: IPC_VERSION,
            mode,
            text: text.into(),
            policy: None,
        }
    }

    pub fn repair_with_policy(mode: Mode, text: impl Into<String>, policy: RepairPolicy) -> Self {
        Self::Repair {
            version: IPC_VERSION,
            mode,
            text: text.into(),
            policy: Some(policy),
        }
    }

    pub fn shutdown() -> Self {
        Self::Shutdown {
            version: IPC_VERSION,
        }
    }

    pub fn version(&self) -> u32 {
        match self {
            Self::Repair { version, .. } | Self::Shutdown { version } => *version,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum IpcResponse {
    Ok { version: u32, report: RepairReport },
    Ack { version: u32, message: String },
    Error { version: u32, message: String },
}

impl IpcResponse {
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            version: IPC_VERSION,
            message: message.into(),
        }
    }
}

pub fn send_request(socket_path: &Path, request: &IpcRequest) -> Result<IpcResponse, String> {
    let mut stream = UnixStream::connect(socket_path)
        .map_err(|error| format!("failed to connect to {}: {error}", socket_path.display()))?;

    write_json(&mut stream, request)?;
    stream
        .shutdown(Shutdown::Write)
        .map_err(|error| format!("failed to finish request write: {error}"))?;

    read_json(&mut stream)
}

pub(crate) fn ensure_socket_parent(path: &Path) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };

    fs::create_dir_all(parent)
        .map_err(|error| format!("failed to create socket dir {}: {error}", parent.display()))
}

fn fallback_socket_namespace() -> String {
    if let Some(home) = env::var_os("HOME")
        && let Ok(metadata) = fs::metadata(home)
    {
        return metadata.uid().to_string();
    }

    if let Some(user) = env::var_os("USER") {
        let sanitized: String = user
            .to_string_lossy()
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                    ch
                } else {
                    '_'
                }
            })
            .collect();

        if !sanitized.is_empty() {
            return sanitized;
        }
    }

    String::from("unknown")
}

pub(crate) fn read_request(stream: &mut UnixStream) -> Result<IpcRequest, String> {
    read_json(stream)
}

pub(crate) fn write_response(
    stream: &mut UnixStream,
    response: &IpcResponse,
) -> Result<(), String> {
    write_json(stream, response)
}

fn read_json<T, R>(reader: &mut R) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
    R: Read,
{
    let mut contents = String::new();
    reader
        .read_to_string(&mut contents)
        .map_err(|error| format!("failed to read ipc payload: {error}"))?;

    serde_json::from_str(&contents).map_err(|error| format!("failed to parse ipc payload: {error}"))
}

fn write_json<T, W>(writer: &mut W, value: &T) -> Result<(), String>
where
    T: Serialize,
    W: Write,
{
    serde_json::to_writer(&mut *writer, value)
        .map_err(|error| format!("failed to serialize ipc payload: {error}"))?;
    writer
        .write_all(b"\n")
        .map_err(|error| format!("failed to write ipc payload: {error}"))
}
