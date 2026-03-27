pub(crate) fn normalize_heading_padding(line: &str) -> String {
    let trimmed = line.trim();
    let marker_len = trimmed.chars().take_while(|ch| *ch == '#').count();

    if marker_len == 0 {
        return line.to_string();
    }

    let rest = &trimmed[marker_len..];
    if rest.is_empty() || !rest.starts_with(char::is_whitespace) {
        return line.to_string();
    }

    format!("{} {}", "#".repeat(marker_len), rest.trim())
}

pub(crate) fn strip_uniform_single_leading_space(input: &str) -> String {
    let mut saw_non_empty = false;

    for line in input.lines() {
        if line.trim().is_empty() {
            continue;
        }

        saw_non_empty = true;

        let mut chars = line.chars();
        if !matches!(chars.next(), Some(' ')) {
            return input.to_string();
        }

        if chars.next().is_some_and(char::is_whitespace) {
            return input.to_string();
        }
    }

    if !saw_non_empty {
        return input.to_string();
    }

    let stripped = input
        .lines()
        .map(|line| line.strip_prefix(' ').unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n");

    if input.ends_with('\n') {
        format!("{stripped}\n")
    } else {
        stripped
    }
}

pub(crate) fn normalize_inline_spacing(line: &str) -> String {
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

pub(crate) fn normalize_reaction_snippet(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn minimal_line_preserving_cleanup(input: &str) -> String {
    let input = strip_uniform_single_leading_space(input);
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

pub(crate) fn finish_with_newline(mut output: String) -> String {
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}
