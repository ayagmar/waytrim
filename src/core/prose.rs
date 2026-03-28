use super::command::repair_command;
use super::detect::{
    blockquote_prefix, heredoc_delimiter, is_blockquote_continuation_line,
    is_command_block_continuation_line, is_list_continuation_line, is_list_item_line,
    is_protected_line, looks_like_aligned_columns_line, looks_like_reaction_snippet,
    looks_like_shell_line, looks_like_yaml_mapping_input, should_collapse_blank_line_between,
};
use super::policy::RepairPolicy;
use super::report::ExplainStep;
use super::text::{
    finish_with_newline, minimal_line_preserving_cleanup, normalize_heading_padding,
    normalize_inline_spacing, normalize_reaction_snippet, strip_uniform_copied_margin,
};

pub(crate) fn repair_prose(input: &str, policy: &RepairPolicy) -> (String, Vec<ExplainStep>) {
    if looks_like_reaction_snippet(input) {
        return (
            normalize_reaction_snippet(input),
            vec![ExplainStep {
                message: String::from("collapsed reaction snippet into one line"),
            }],
        );
    }

    if looks_like_yaml_mapping_input(input) {
        return (
            minimal_line_preserving_cleanup(input),
            vec![ExplainStep {
                message: String::from("preserved structured yaml-like text"),
            }],
        );
    }

    let input = strip_uniform_copied_margin(input);
    let mut output_lines = Vec::new();
    let mut paragraph = Vec::new();
    let mut paragraph_start_line = None;
    let mut list_item: Option<String> = None;
    let mut active_quote: Option<String> = None;
    let mut in_fenced_code = false;
    let mut in_command_block = false;
    let mut active_heredoc: Option<String> = None;
    let mut in_aligned_block = false;
    let mut explain = Vec::new();

    let mut lines = input.lines().enumerate().peekable();

    while let Some((index, raw_line)) = lines.next() {
        let line_number = index + 1;
        let line = raw_line.trim_end();
        let trimmed = line.trim();

        if let Some(delimiter) = active_heredoc.as_ref() {
            output_lines.push(line.to_string());
            if trimmed == delimiter {
                active_heredoc = None;
            }
            continue;
        }

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
            if paragraph.last().is_some_and(|previous| {
                lines.peek().is_some_and(|(_, next)| {
                    should_collapse_blank_line_between(previous, next.trim_end())
                })
            }) {
                continue;
            }

            flush_paragraph(
                &mut paragraph,
                &mut paragraph_start_line,
                &mut output_lines,
                &mut explain,
            );
            flush_list_item(&mut list_item, &mut output_lines);
            flush_quote(&mut active_quote, &mut output_lines);
            in_command_block = false;
            active_heredoc = None;

            if output_lines.last().is_none_or(|last| !last.is_empty()) {
                output_lines.push(String::new());
            }

            continue;
        }

        if in_aligned_block {
            if looks_like_aligned_columns_line(line) {
                output_lines.push(if policy.protect_aligned_columns {
                    line.to_string()
                } else {
                    normalize_inline_spacing(line)
                });
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
            output_lines.push(if policy.protect_aligned_columns {
                line.to_string()
            } else {
                normalize_inline_spacing(line)
            });
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

            if policy.protect_command_blocks {
                in_command_block = true;
                active_heredoc = heredoc_delimiter(line);
                output_lines.push(line.to_string());
                continue;
            }

            let mut command_block = vec![line.to_string()];

            while let Some((_, next_raw_line)) = lines.peek() {
                let next_line = next_raw_line.trim_end();
                let next_trimmed = next_line.trim();

                if next_trimmed.is_empty() {
                    break;
                }

                if is_command_block_continuation_line(next_line)
                    || looks_like_shell_line(next_trimmed)
                {
                    command_block.push(next_line.to_string());
                    lines.next();
                    continue;
                }

                break;
            }

            let (command_output, command_explain) =
                repair_command(&finish_with_newline(command_block.join("\n")));
            explain.extend(command_explain);
            output_lines.extend(
                command_output
                    .trim_end_matches('\n')
                    .lines()
                    .map(ToOwned::to_owned),
            );
            continue;
        }

        if in_command_block {
            if let Some(delimiter) = heredoc_delimiter(line) {
                active_heredoc = Some(delimiter);
                output_lines.push(line.to_string());
                continue;
            }

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
            output_lines.push(normalize_heading_padding(line));
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
