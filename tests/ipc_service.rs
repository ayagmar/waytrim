mod support;

use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use support::temp_file_path;
use waytrim::ipc::send_request;
use waytrim::{IpcRequest, IpcResponse, Mode, RepairDecision};

struct Daemon {
    child: Child,
    socket_path: PathBuf,
}

impl Daemon {
    fn start() -> Self {
        let socket_path = temp_file_path("waytrim-ipc.sock");
        let child = Command::new(env!("CARGO_BIN_EXE_waytrimd"))
            .arg("--socket")
            .arg(&socket_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
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

        Self { child, socket_path }
    }

    fn socket_arg(&self) -> String {
        self.socket_path.display().to_string()
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
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
