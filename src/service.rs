use std::fs;
use std::io::ErrorKind;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};

use crate::ipc::{
    IPC_VERSION, IpcRequest, IpcResponse, default_socket_path, ensure_socket_parent, read_request,
    write_response,
};
use crate::{repair_report, repair_report_with_policy};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceConfig {
    pub socket_path: PathBuf,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            socket_path: default_socket_path(),
        }
    }
}

pub fn run_service(config: &ServiceConfig) -> Result<(), String> {
    ensure_socket_parent(&config.socket_path)?;
    remove_stale_socket(&config.socket_path)?;

    let _cleanup = SocketCleanup::new(config.socket_path.clone());
    let listener = UnixListener::bind(&config.socket_path).map_err(|error| {
        format!(
            "failed to bind socket {}: {error}",
            config.socket_path.display()
        )
    })?;

    loop {
        let (mut stream, _) = listener
            .accept()
            .map_err(|error| format!("failed to accept ipc connection: {error}"))?;

        let (response, should_stop) = match read_request(&mut stream) {
            Ok(request) => handle_request(request),
            Err(error) => (IpcResponse::error(error), false),
        };

        write_response(&mut stream, &response)?;

        if should_stop {
            break;
        }
    }

    Ok(())
}

fn handle_request(request: IpcRequest) -> (IpcResponse, bool) {
    if request.version() != IPC_VERSION {
        return (
            IpcResponse::error(format!("unsupported ipc version: {}", request.version())),
            false,
        );
    }

    match request {
        IpcRequest::Repair {
            mode, text, policy, ..
        } => {
            let report = if let Some(policy) = policy {
                repair_report_with_policy(&text, mode, &policy)
            } else {
                repair_report(&text, mode)
            };

            (
                IpcResponse::Ok {
                    version: IPC_VERSION,
                    report,
                },
                false,
            )
        }
        IpcRequest::Shutdown { .. } => (
            IpcResponse::Ack {
                version: IPC_VERSION,
                message: String::from("service shutting down"),
            },
            true,
        ),
    }
}

fn remove_stale_socket(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "failed to remove stale socket {}: {error}",
            path.display()
        )),
    }
}

struct SocketCleanup {
    path: PathBuf,
}

impl SocketCleanup {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for SocketCleanup {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
