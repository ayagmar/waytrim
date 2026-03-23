use super::policy::Mode;
use super::report::RepairResult;

pub fn render_preview(input: &str, result: &RepairResult) -> String {
    let mut preview = String::from("--- before\n+++ after\n");

    if !result.changed {
        preview.push_str("(no changes)\n");
        return preview;
    }

    let before_lines: Vec<_> = input.lines().collect();
    let after_lines: Vec<_> = result.output.lines().collect();
    let shared = before_lines.len().min(after_lines.len());

    for index in 0..shared {
        if before_lines[index] == after_lines[index] {
            continue;
        }

        preview.push('-');
        preview.push_str(before_lines[index]);
        preview.push('\n');
        preview.push('+');
        preview.push_str(after_lines[index]);
        preview.push('\n');
    }

    for line in &before_lines[shared..] {
        preview.push('-');
        preview.push_str(line);
        preview.push('\n');
    }

    for line in &after_lines[shared..] {
        preview.push('+');
        preview.push_str(line);
        preview.push('\n');
    }

    preview
}

pub fn render_explain(mode: Mode, result: &RepairResult) -> String {
    let mut output = format!(
        "mode: {}\nchanged: {}\nrepairs:\n",
        mode.as_str(),
        if result.changed { "yes" } else { "no" }
    );

    if result.explain.is_empty() {
        output.push_str(if result.changed {
            "- repaired text\n"
        } else {
            "- no repair needed\n"
        });
    } else {
        for step in &result.explain {
            output.push_str("- ");
            output.push_str(&step.message);
            output.push('\n');
        }
    }

    output.push_str("--- output\n");
    output.push_str(&result.output);
    output
}
