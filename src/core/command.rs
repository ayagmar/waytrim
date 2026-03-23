use super::detect::{looks_like_command, looks_like_shell_line, strip_prompt};
use super::report::ExplainStep;
use super::text::finish_with_newline;

pub(crate) fn repair_command(input: &str) -> (String, Vec<ExplainStep>) {
    let mut lines: Vec<(Option<usize>, String)> = Vec::new();
    let mut saw_prompt = false;
    let mut explain = Vec::new();

    for (index, raw_line) in input.lines().enumerate() {
        let line_number = index + 1;
        let trimmed_end = raw_line.trim_end();

        if trimmed_end.trim().is_empty() {
            lines.push((None, String::new()));
            continue;
        }

        if let Some(stripped) = strip_prompt(trimmed_end) {
            saw_prompt = true;
            explain.push(ExplainStep {
                message: format!("stripped shell prompt from line {line_number}"),
            });
            lines.push((Some(line_number), stripped.to_string()));
            continue;
        }

        if lines.last().is_some_and(|(_, line)| line.ends_with('\\')) {
            lines.push((Some(line_number), trimmed_end.trim_start().to_string()));
            continue;
        }

        if looks_like_shell_line(trimmed_end) {
            lines.push((Some(line_number), trimmed_end.trim().to_string()));
            continue;
        }

        return (
            finish_with_newline(input.trim_end().to_string()),
            Vec::new(),
        );
    }

    if !saw_prompt && !looks_like_command(input) {
        return (
            finish_with_newline(input.trim_end().to_string()),
            Vec::new(),
        );
    }

    let mut joined = Vec::new();
    let mut current = String::new();
    let mut current_start_line = None;
    let mut continued_range = None;

    for (line_number, line) in lines {
        if line.is_empty() {
            if !current.is_empty() {
                if let Some((start, end)) = continued_range.take() {
                    explain.push(ExplainStep {
                        message: format!("joined continued command lines {start}-{end}"),
                    });
                }
                joined.push(current.trim().to_string());
                current.clear();
                current_start_line = None;
            }
            joined.push(String::new());
            continue;
        }

        if current.is_empty() {
            current_start_line = line_number;
            current = line;
            continue;
        }

        if current.ends_with('\\') {
            current.pop();
            while current.ends_with(' ') {
                current.pop();
            }
            current.push(' ');
            current.push_str(line.trim());

            let start = current_start_line.or(line_number).unwrap_or(1);
            let end = line_number.unwrap_or(start);
            continued_range = Some((start, end));
            continue;
        }

        if let Some((start, end)) = continued_range.take() {
            explain.push(ExplainStep {
                message: format!("joined continued command lines {start}-{end}"),
            });
        }
        joined.push(current.trim().to_string());
        current_start_line = line_number;
        current = line;
    }

    if !current.is_empty() {
        if let Some((start, end)) = continued_range.take() {
            explain.push(ExplainStep {
                message: format!("joined continued command lines {start}-{end}"),
            });
        }
        joined.push(current.trim().to_string());
    }

    while joined.last().is_some_and(|line| line.is_empty()) {
        joined.pop();
    }

    (finish_with_newline(joined.join("\n")), explain)
}
