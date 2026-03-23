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

pub(crate) fn finish_with_newline(mut output: String) -> String {
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}
