mod support;

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use support::temp_file_path;
use waytrim::ipc::{IPC_VERSION, send_request};
use waytrim::{IpcRequest, IpcResponse, Mode, RepairDecision, RepairPolicy};

struct Daemon {
    child: Child,
    socket_path: PathBuf,
    stopped: bool,
}

impl Daemon {
    fn start() -> Self {
        Self::start_at(temp_file_path("waytrim-ipc.sock"))
    }

    fn start_at(socket_path: PathBuf) -> Self {
        let child = Command::new(env!("CARGO_BIN_EXE_waytrimd"))
            .arg("--socket")
            .arg(&socket_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn waytrimd");

        let start = Instant::now();
        while !socket_path.exists() {
            assert!(
                start.elapsed() < Duration::from_secs(2),
                "socket was not created: {}",
                socket_path.display()
            );
            thread::sleep(Duration::from_millis(20));
        }

        Self {
            child,
            socket_path,
            stopped: false,
        }
    }

    fn socket_arg(&self) -> String {
        self.socket_path.display().to_string()
    }

    fn shutdown(&mut self) {
        if self.stopped {
            return;
        }

        let response = send_request(&self.socket_path, &IpcRequest::shutdown())
            .expect("send shutdown request");
        assert!(matches!(response, IpcResponse::Ack { .. }));

        let status = self.child.wait().expect("wait on daemon exit");
        assert!(status.success(), "daemon exited with {status}");
        self.stopped = true;
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        if self.stopped {
            return;
        }

        let _ = send_request(&self.socket_path, &IpcRequest::shutdown());
        let _ = self.child.wait();
    }
}

#[test]
fn waytrimctl_repair_returns_json_report_from_service() {
    let daemon = Daemon::start();
    let mut child = Command::new(env!("CARGO_BIN_EXE_waytrimctl"))
        .args(["repair", "auto", "--socket", &daemon.socket_arg()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn waytrimctl");

    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(b"This is a wrapped\nparagraph from a terminal.\n")
        .expect("write stdin");

    let output = child.wait_with_output().expect("wait on waytrimctl");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let response: IpcResponse =
        serde_json::from_slice(&output.stdout).expect("parse service response json");

    match response {
        IpcResponse::Ok { report, .. } => {
            assert_eq!(report.requested_mode, Mode::Auto);
            assert_eq!(report.effective_mode, Mode::Prose);
            assert_eq!(report.decision, RepairDecision::AutoProse);
            assert_eq!(
                report.output,
                "This is a wrapped paragraph from a terminal.\n"
            );
            assert!(report.changed);
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn waytrimctl_repair_can_print_repaired_text_only() {
    let daemon = Daemon::start();
    let mut child = Command::new(env!("CARGO_BIN_EXE_waytrimctl"))
        .args([
            "repair",
            "prose",
            "--socket",
            &daemon.socket_arg(),
            "--text",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn waytrimctl");

    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(b"This is a wrapped\nparagraph.\n")
        .expect("write stdin");

    let output = child.wait_with_output().expect("wait on waytrimctl");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("utf8 stdout"),
        "This is a wrapped paragraph.\n"
    );
}

#[test]
fn service_rejects_unsupported_ipc_version() {
    let daemon = Daemon::start();
    let response = send_request(
        &daemon.socket_path,
        &IpcRequest::Repair {
            version: IPC_VERSION + 1,
            mode: Mode::Prose,
            text: String::from("This is a wrapped\nparagraph.\n"),
            policy: None,
        },
    )
    .expect("send version mismatch request");

    match response {
        IpcResponse::Error { message, .. } => {
            assert!(message.contains("unsupported ipc version"));
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn service_returns_parse_error_for_malformed_json() {
    let daemon = Daemon::start();
    let mut stream = UnixStream::connect(&daemon.socket_path).expect("connect to daemon socket");
    stream
        .write_all(b"{ this is not valid json\n")
        .expect("write malformed json");
    stream
        .shutdown(std::net::Shutdown::Write)
        .expect("shutdown write side");

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .expect("read malformed-json response");

    let response: IpcResponse = serde_json::from_str(&response).expect("parse error response");
    match response {
        IpcResponse::Error { message, .. } => {
            assert!(message.contains("failed to parse ipc payload"));
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn service_applies_policy_sent_over_ipc() {
    let daemon = Daemon::start();
    let response = send_request(
        &daemon.socket_path,
        &IpcRequest::repair_with_policy(
            Mode::Prose,
            "Use this command:\n\ncargo test \\\n  --test clipboard_flow\n",
            RepairPolicy {
                protect_command_blocks: false,
                ..RepairPolicy::default()
            },
        ),
    )
    .expect("send policy repair request");

    match response {
        IpcResponse::Ok { report, .. } => {
            assert_eq!(
                report.output,
                "Use this command:\n\ncargo test --test clipboard_flow\n"
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn service_can_shutdown_and_restart_on_same_socket_path() {
    let socket_path = temp_file_path("waytrim-restart.sock");

    let mut first = Daemon::start_at(socket_path.clone());
    first.shutdown();

    let second = Daemon::start_at(socket_path.clone());
    let response = send_request(
        &socket_path,
        &IpcRequest::repair(Mode::Prose, "This is a wrapped\nparagraph.\n"),
    )
    .expect("send repair request after restart");

    match response {
        IpcResponse::Ok { report, .. } => {
            assert_eq!(report.output, "This is a wrapped paragraph.\n");
        }
        other => panic!("unexpected response: {other:?}"),
    }

    drop(second);
}

#[test]
fn second_service_refuses_to_take_over_active_socket() {
    let daemon = Daemon::start();
    let output = Command::new(env!("CARGO_BIN_EXE_waytrimd"))
        .arg("--socket")
        .arg(&daemon.socket_path)
        .output()
        .expect("run second waytrimd");

    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .expect("utf8 stderr")
            .contains("service already listening")
    );
}
