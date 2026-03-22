pub mod cli;
pub mod clipboard;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Prose,
    Command,
    Auto,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Prose => "prose",
            Self::Command => "command",
            Self::Auto => "auto",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplainStep {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairResult {
    pub output: String,
    pub changed: bool,
    pub explain: Vec<ExplainStep>,
}

pub fn repair(input: &str, mode: Mode) -> RepairResult {
    let (output, explain) = match mode {
        Mode::Prose => repair_prose(input),
        Mode::Command => repair_command(input),
        Mode::Auto => repair_auto(input),
    };

    RepairResult {
        changed: output != input,
        output,
        explain,
    }
}

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

fn repair_auto(input: &str) -> (String, Vec<ExplainStep>) {
    if looks_like_command(input) {
        return repair_command(input);
    }

    if looks_like_command_transcript(input) {
        return (minimal_prose_safe_cleanup(input), Vec::new());
    }

    if looks_like_label_plus_command(input) {
        return (minimal_prose_safe_cleanup(input), Vec::new());
    }

    if looks_like_prose(input) {
        return repair_prose(input);
    }

    (minimal_prose_safe_cleanup(input), Vec::new())
}

fn repair_prose(input: &str) -> (String, Vec<ExplainStep>) {
    let mut output_lines = Vec::new();
    let mut paragraph = Vec::new();
    let mut paragraph_start_line = None;
    let mut list_item: Option<String> = None;
    let mut active_quote: Option<String> = None;
    let mut in_fenced_code = false;
    let mut in_command_block = false;
    let mut in_aligned_block = false;
    let mut explain = Vec::new();

    let mut lines = input.lines().enumerate().peekable();

    while let Some((index, raw_line)) = lines.next() {
        let line_number = index + 1;
        let line = raw_line.trim_end();
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            flush_paragraph(
                &mut paragraph,
                &mut paragraph_start_line,
                &mut output_lines,
                &mut explain,
            );
            flush_list_item(&mut list_item, &mut output_lines);
            flush_quote(&mut active_quote, &mut output_lines);
            in_fenced_code = !in_fenced_code;
            output_lines.push(line.to_string());
            continue;
        }

        if in_fenced_code {
            output_lines.push(line.to_string());
            continue;
        }

        if trimmed.is_empty() {
            flush_paragraph(
                &mut paragraph,
                &mut paragraph_start_line,
                &mut output_lines,
                &mut explain,
            );
            flush_list_item(&mut list_item, &mut output_lines);
            flush_quote(&mut active_quote, &mut output_lines);
            in_command_block = false;

            if output_lines.last().is_none_or(|last| !last.is_empty()) {
                output_lines.push(String::new());
            }

            continue;
        }

        if in_aligned_block {
            if looks_like_aligned_columns_line(line) {
                output_lines.push(line.to_string());
                continue;
            }

            in_aligned_block = false;
        }

        if looks_like_aligned_columns_line(line)
            && lines
                .peek()
                .is_some_and(|(_, next)| looks_like_aligned_columns_line(next.trim_end()))
        {
            flush_paragraph(
                &mut paragraph,
                &mut paragraph_start_line,
                &mut output_lines,
                &mut explain,
            );
            flush_list_item(&mut list_item, &mut output_lines);
            flush_quote(&mut active_quote, &mut output_lines);
            in_command_block = false;
            in_aligned_block = true;
            output_lines.push(line.to_string());
            continue;
        }

        if !is_list_item_line(trimmed) && looks_like_shell_line(trimmed) {
            flush_paragraph(
                &mut paragraph,
                &mut paragraph_start_line,
                &mut output_lines,
                &mut explain,
            );
            flush_list_item(&mut list_item, &mut output_lines);
            flush_quote(&mut active_quote, &mut output_lines);
            in_command_block = true;
            output_lines.push(line.to_string());
            continue;
        }

        if in_command_block {
            if is_command_block_continuation_line(line) {
                output_lines.push(line.to_string());
                continue;
            }

            in_command_block = false;
        }

        if is_list_item_line(trimmed) {
            flush_paragraph(
                &mut paragraph,
                &mut paragraph_start_line,
                &mut output_lines,
                &mut explain,
            );
            flush_list_item(&mut list_item, &mut output_lines);
            flush_quote(&mut active_quote, &mut output_lines);
            list_item = Some(normalize_inline_spacing(trimmed));
            continue;
        }

        if let Some(item) = list_item.as_mut() {
            if is_list_continuation_line(line) {
                item.push(' ');
                item.push_str(&normalize_inline_spacing(trimmed));
                continue;
            }

            flush_list_item(&mut list_item, &mut output_lines);
        }

        if let Some(quote) = blockquote_prefix(trimmed) {
            flush_paragraph(
                &mut paragraph,
                &mut paragraph_start_line,
                &mut output_lines,
                &mut explain,
            );
            flush_quote(&mut active_quote, &mut output_lines);
            active_quote = Some(format!("> {}", normalize_inline_spacing(quote)));
            continue;
        }

        if let Some(quote) = active_quote.as_mut() {
            if is_blockquote_continuation_line(line) {
                quote.push(' ');
                quote.push_str(&normalize_inline_spacing(trimmed));
                continue;
            }

            flush_quote(&mut active_quote, &mut output_lines);
        }

        if is_protected_line(line) {
            flush_paragraph(
                &mut paragraph,
                &mut paragraph_start_line,
                &mut output_lines,
                &mut explain,
            );
            output_lines.push(line.to_string());
            continue;
        }

        if paragraph_start_line.is_none() {
            paragraph_start_line = Some(line_number);
        }
        paragraph.push(line.to_string());
    }

    flush_paragraph(
        &mut paragraph,
        &mut paragraph_start_line,
        &mut output_lines,
        &mut explain,
    );
    flush_list_item(&mut list_item, &mut output_lines);
    flush_quote(&mut active_quote, &mut output_lines);

    while output_lines.last().is_some_and(|line| line.is_empty()) {
        output_lines.pop();
    }

    (finish_with_newline(output_lines.join("\n")), explain)
}

fn flush_paragraph(
    paragraph: &mut Vec<String>,
    paragraph_start_line: &mut Option<usize>,
    output_lines: &mut Vec<String>,
    explain: &mut Vec<ExplainStep>,
) {
    if paragraph.is_empty() {
        *paragraph_start_line = None;
        return;
    }

    let joined = paragraph
        .iter()
        .map(|line| normalize_inline_spacing(line.trim()))
        .collect::<Vec<_>>()
        .join(" ");

    if paragraph.len() > 1 {
        let start = paragraph_start_line.unwrap_or(1);
        let end = start + paragraph.len() - 1;
        explain.push(ExplainStep {
            message: format!("joined wrapped paragraph lines {start}-{end}"),
        });
    }

    output_lines.push(joined);
    paragraph.clear();
    *paragraph_start_line = None;
}

fn flush_list_item(list_item: &mut Option<String>, output_lines: &mut Vec<String>) {
    let Some(item) = list_item.take() else {
        return;
    };

    output_lines.push(item);
}

fn flush_quote(active_quote: &mut Option<String>, output_lines: &mut Vec<String>) {
    let Some(quote) = active_quote.take() else {
        return;
    };

    output_lines.push(quote);
}

fn repair_command(input: &str) -> (String, Vec<ExplainStep>) {
    let mut lines = Vec::new();
    let mut saw_prompt = false;

    for raw_line in input.lines() {
        let trimmed_end = raw_line.trim_end();

        if trimmed_end.trim().is_empty() {
            lines.push(String::new());
            continue;
        }

        if let Some(stripped) = strip_prompt(trimmed_end) {
            saw_prompt = true;
            lines.push(stripped.to_string());
            continue;
        }

        if lines
            .last()
            .is_some_and(|line: &String| line.ends_with('\\'))
        {
            lines.push(trimmed_end.trim_start().to_string());
            continue;
        }

        if looks_like_shell_line(trimmed_end) {
            lines.push(trimmed_end.trim().to_string());
            continue;
        }

        return (finish_with_newline(input.trim_end().to_string()), Vec::new());
    }

    if !saw_prompt && !looks_like_command(input) {
        return (finish_with_newline(input.trim_end().to_string()), Vec::new());
    }

    let mut joined = Vec::new();
    let mut current = String::new();

    for line in lines {
        if line.is_empty() {
            if !current.is_empty() {
                joined.push(current.trim().to_string());
                current.clear();
            }
            joined.push(String::new());
            continue;
        }

        if current.is_empty() {
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
            continue;
        }

        joined.push(current.trim().to_string());
        current = line;
    }

    if !current.is_empty() {
        joined.push(current.trim().to_string());
    }

    while joined.last().is_some_and(|line| line.is_empty()) {
        joined.pop();
    }

    (finish_with_newline(joined.join("\n")), Vec::new())
}

fn minimal_prose_safe_cleanup(input: &str) -> String {
    let mut output = Vec::new();
    let mut blank_count = 0;

    for line in input.lines() {
        let trimmed_end = line.trim_end();

        if trimmed_end.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                output.push(String::new());
            }
            continue;
        }

        blank_count = 0;
        output.push(trimmed_end.to_string());
    }

    while output.last().is_some_and(|line| line.is_empty()) {
        output.pop();
    }

    finish_with_newline(output.join("\n"))
}

fn looks_like_label_plus_command(input: &str) -> bool {
    let lines: Vec<_> = input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    if lines.len() != 2 {
        return false;
    }

    lines[0].ends_with(':') && looks_like_command_line_after_label(lines[1])
}

fn looks_like_command_line_after_label(line: &str) -> bool {
    let mut parts = line.split_whitespace();
    let Some(program) = parts.next() else {
        return false;
    };

    program
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/'))
        && parts.next().is_some()
}

fn looks_like_prose(input: &str) -> bool {
    let lines: Vec<_> = input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    if lines.len() < 2 {
        return false;
    }

    let sentence_like = lines
        .iter()
        .filter(|line| {
            line.contains(' ')
                && line
                    .chars()
                    .any(|ch| matches!(ch, '.' | ',' | ':' | ';' | '?' | '!'))
        })
        .count();

    sentence_like >= 1 && lines.iter().all(|line| !looks_like_shell_line(line))
}

fn looks_like_command_transcript(input: &str) -> bool {
    let non_empty: Vec<_> = input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    if non_empty.len() < 2 || strip_prompt(non_empty[0]).is_none() {
        return false;
    }

    non_empty[1..].iter().any(|line| {
        strip_prompt(line).is_none()
            && !is_command_continuation_line(line)
            && !looks_like_shell_line(line)
    })
}

fn looks_like_command(input: &str) -> bool {
    let non_empty: Vec<_> = input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    if non_empty.is_empty() {
        return false;
    }

    if strip_prompt(non_empty[0]).is_some()
        && non_empty[1..].iter().all(|line| {
            strip_prompt(line).is_some()
                || is_command_continuation_line(line)
                || looks_like_shell_line(line)
        })
    {
        return true;
    }

    let shellish = non_empty
        .iter()
        .filter(|line| looks_like_shell_line(line))
        .count();
    shellish >= 1 && shellish == non_empty.len()
}

fn looks_like_shell_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }

    let starts_with_program = trimmed.split_whitespace().next().is_some_and(|word| {
        word.chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/'))
    });

    let shell_tokens = ["|", "&&", "||", "$()", "--", "~/", "./", ">", "<", "\\"];

    starts_with_program && shell_tokens.iter().any(|token| trimmed.contains(token))
        || trimmed.starts_with("sudo ")
        || trimmed.starts_with("git ")
        || trimmed.starts_with("cargo ")
        || trimmed.starts_with("rm ")
        || trimmed.starts_with("cp ")
        || trimmed.starts_with("mv ")
        || trimmed.starts_with("cd ")
        || trimmed.starts_with("ls ")
        || trimmed.starts_with("echo ")
}

fn is_command_continuation_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('-') || trimmed.starts_with("\\")
}

fn is_command_block_continuation_line(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty()
        && line.starts_with(char::is_whitespace)
        && (trimmed.starts_with("--") || trimmed.starts_with("\\"))
}

fn strip_prompt(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();

    for prompt in ["$ ", "# ", "% "] {
        if let Some(stripped) = trimmed.strip_prefix(prompt) {
            return Some(stripped.trim_start());
        }
    }

    for marker in ["$ ", "# ", "% "] {
        let Some((prefix, command)) = trimmed.rsplit_once(marker) else {
            continue;
        };

        if prefix.contains('@') || prefix.contains(':') || prefix.contains('~') {
            return Some(command.trim_start());
        }
    }

    None
}

fn blockquote_prefix(trimmed: &str) -> Option<&str> {
    Some(trimmed.strip_prefix('>')?.trim_start())
}

fn is_blockquote_continuation_line(line: &str) -> bool {
    line.starts_with(char::is_whitespace) && !line.trim().is_empty()
}

fn is_protected_line(line: &str) -> bool {
    let trimmed = line.trim();

    trimmed.starts_with("```")
        || trimmed.starts_with('>')
        || trimmed.starts_with("#")
        || line.starts_with("    ")
        || line.starts_with('\t')
        || is_bullet_line(trimmed)
        || is_numbered_line(trimmed)
}

fn looks_like_aligned_columns_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("```") {
        return false;
    }

    let mut segments = 0;
    let mut in_segment = false;
    let mut spaces = 0;
    let mut saw_wide_gap = false;

    for ch in trimmed.chars() {
        if ch.is_whitespace() {
            spaces += 1;
            if in_segment && spaces >= 2 {
                segments += 1;
                in_segment = false;
                saw_wide_gap = true;
            }
            continue;
        }

        spaces = 0;
        in_segment = true;
    }

    if in_segment {
        segments += 1;
    }

    saw_wide_gap && segments >= 2
}

fn is_bullet_line(trimmed: &str) -> bool {
    ["- ", "* ", "+ "]
        .iter()
        .any(|marker| trimmed.starts_with(marker))
}

fn is_list_item_line(trimmed: &str) -> bool {
    is_bullet_line(trimmed) || is_numbered_line(trimmed)
}

fn is_list_continuation_line(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty() && line.starts_with(char::is_whitespace) && !is_protected_line(line)
}

fn is_numbered_line(trimmed: &str) -> bool {
    let mut chars = trimmed.chars().peekable();
    let mut saw_digit = false;

    while let Some(ch) = chars.peek().copied() {
        if ch.is_ascii_digit() {
            saw_digit = true;
            chars.next();
            continue;
        }
        break;
    }

    if !saw_digit {
        return false;
    }

    matches!(chars.next(), Some('.') | Some(')')) && matches!(chars.next(), Some(' '))
}

fn normalize_inline_spacing(line: &str) -> String {
    let mut result = String::with_capacity(line.len());
    let mut last_was_whitespace = false;

    for ch in line.chars() {
        if ch.is_whitespace() {
            if !last_was_whitespace {
                result.push(' ');
            }
            last_was_whitespace = true;
            continue;
        }

        result.push(ch);
        last_was_whitespace = false;
    }

    result.trim().to_string()
}

fn finish_with_newline(mut output: String) -> String {
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{Mode, minimal_prose_safe_cleanup, repair};

    #[test]
    fn auto_falls_back_to_minimal_cleanup_for_ambiguous_input() {
        let input = "value one  \n\n\nvalue two\n";
        let result = repair(input, Mode::Auto);

        assert_eq!(result.output, minimal_prose_safe_cleanup(input));
    }
}
