#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};

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
    let mut child = Command::new(env!("CARGO_BIN_EXE_waytrim"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn waytrim");

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
