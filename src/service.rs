use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::net::{UnixListener, UnixStream};
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
    prepare_socket_path(&config.socket_path)?;

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

fn prepare_socket_path(path: &Path) -> Result<(), String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(format!(
                "failed to inspect socket path {}: {error}",
                path.display()
            ));
        }
    };

    if !metadata.file_type().is_socket() {
        return Err(format!(
            "refusing to remove non-socket path {}",
            path.display()
        ));
    }

    match UnixStream::connect(path) {
        Ok(_) => Err(format!("service already listening on {}", path.display())),
        Err(error) if error.kind() == ErrorKind::ConnectionRefused => remove_socket_file(path),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "failed to verify existing socket {}: {error}",
            path.display()
        )),
    }
}

fn remove_socket_file(path: &Path) -> Result<(), String> {
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

#[cfg(test)]
mod tests {
    use super::prepare_socket_path;
    use std::fs;
    use std::os::unix::net::UnixListener;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn temp_socket_path(stem: &str) -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "waytrim-service-test-{stem}-{}-{unique}.sock",
            std::process::id()
        ))
    }

    #[test]
    fn prepare_socket_path_rejects_non_socket_files() {
        let path = temp_socket_path("file");
        fs::write(&path, "not a socket").expect("write regular file");

        let error = prepare_socket_path(&path).expect_err("expected non-socket error");
        assert!(error.contains("refusing to remove non-socket path"));

        fs::remove_file(path).expect("remove regular file");
    }

    #[test]
    fn prepare_socket_path_rejects_active_listener() {
        let path = temp_socket_path("active");
        let listener = UnixListener::bind(&path).expect("bind active listener");

        let error = prepare_socket_path(&path).expect_err("expected active-listener error");
        assert!(error.contains("service already listening"));

        drop(listener);
        fs::remove_file(path).expect("remove socket file");
    }

    #[test]
    fn prepare_socket_path_removes_stale_socket_files() {
        let path = temp_socket_path("stale");
        let listener = UnixListener::bind(&path).expect("bind socket listener");
        drop(listener);

        assert!(path.exists(), "stale socket file should remain on disk");
        prepare_socket_path(&path).expect("remove stale socket file");
        assert!(!path.exists(), "stale socket file should be removed");
    }
}
