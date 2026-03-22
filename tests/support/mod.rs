#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub struct CommandOutput {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

pub fn run_waytrim(args: &[&str], input: &str) -> String {
    let output = run_waytrim_capture(args, input);
    assert!(output.status.success(), "stderr: {}", output.stderr);
    output.stdout
}

pub fn run_waytrim_capture(args: &[&str], input: &str) -> CommandOutput {
    run_waytrim_capture_env(args, input, &[])
}

pub fn run_waytrim_capture_env(args: &[&str], input: &str, envs: &[(&str, &str)]) -> CommandOutput {
    let mut command = Command::new(env!("CARGO_BIN_EXE_waytrim"));
    command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    for (key, value) in envs {
        command.env(key, value);
    }

    let mut child = command.spawn().expect("spawn waytrim");

    use std::io::Write;
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(input.as_bytes())
        .expect("write stdin");

    let output = child.wait_with_output().expect("wait on child");

    CommandOutput {
        status: output.status,
        stdout: String::from_utf8(output.stdout).expect("utf8 stdout"),
        stderr: String::from_utf8(output.stderr).expect("utf8 stderr"),
    }
}

pub fn fixture_input(stem: &str) -> String {
    fs::read_to_string(fixture_path(stem, "txt")).expect("read fixture input")
}

pub fn fixture_output(stem: &str) -> String {
    fs::read_to_string(fixture_path(stem, "expected.txt")).expect("read fixture output")
}

pub fn fixture_meta(stem: &str) -> FixtureMeta {
    let path = fixture_path(stem, "meta.txt");
    let contents = fs::read_to_string(path).expect("read fixture meta");
    FixtureMeta::parse(&contents)
}

fn fixture_path(stem: &str, extension: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(format!("{stem}.{extension}"))
}

#[derive(Debug, Default)]
pub struct FixtureMeta {
    pub notes: Vec<String>,
    pub preserve: Vec<String>,
    pub avoid: Vec<String>,
}

impl FixtureMeta {
    fn parse(contents: &str) -> Self {
        let mut meta = Self::default();

        for line in contents.lines() {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };

            let values = value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned);

            match key.trim() {
                "notes" => meta.notes.extend(values),
                "preserve" => meta.preserve.extend(values),
                "avoid" => meta.avoid.extend(values),
                _ => {}
            }
        }

        meta
    }
}

pub fn temp_file_path(stem: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("waytrim-{stem}-{}-{unique}", std::process::id()))
}

pub fn temp_dir_path(stem: &str) -> PathBuf {
    temp_file_path(stem)
}

pub fn write_executable_script(stem: &str, contents: &str) -> PathBuf {
    let path = temp_file_path(stem);
    fs::write(&path, contents).expect("write script");
    set_executable(&path);
    path
}

fn set_executable(path: &Path) {
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(path).expect("script metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("set script executable permissions");
    }
}
