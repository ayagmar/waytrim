use std::process::{Command, Stdio};

fn run_waytrim(args: &[&str], input: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_waytrim"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
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
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("utf8 stdout")
}

#[test]
fn cli_smoke_prose_reads_stdin_and_repairs_wrapped_text() {
    let input = "This is a wrapped\nparagraph from a terminal.\n";
    let output = run_waytrim(&["prose"], input);

    assert_eq!(output, "This is a wrapped paragraph from a terminal.\n");
}
