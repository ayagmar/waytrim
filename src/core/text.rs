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

pub(crate) fn strip_uniform_copied_margin(input: &str) -> String {
    let without_gutter = strip_uniform_leading_gutter(input);
    strip_uniform_leading_whitespace(&without_gutter)
}

pub(crate) fn strip_uniform_leading_whitespace(input: &str) -> String {
    let mut common_prefix: Option<String> = None;

    for line in input.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let prefix: String = line.chars().take_while(|ch| ch.is_whitespace()).collect();

        if prefix.is_empty() {
            return input.to_string();
        }

        common_prefix = Some(match common_prefix {
            Some(existing) => common_whitespace_prefix(&existing, &prefix),
            None => prefix,
        });

        if common_prefix
            .as_ref()
            .is_some_and(|prefix| prefix.is_empty())
        {
            return input.to_string();
        }
    }

    let Some(prefix) = common_prefix.filter(|prefix| !prefix.is_empty()) else {
        return input.to_string();
    };

    let stripped = input
        .lines()
        .map(|line| line.strip_prefix(&prefix).unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n");

    if input.ends_with('\n') {
        format!("{stripped}\n")
    } else {
        stripped
    }
}

fn strip_uniform_leading_gutter(input: &str) -> String {
    let mut saw_non_empty = false;

    for line in input.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if !has_copied_gutter(line) {
            return input.to_string();
        }

        saw_non_empty = true;
    }

    if !saw_non_empty {
        return input.to_string();
    }

    let stripped = input
        .lines()
        .map(strip_leading_gutter)
        .collect::<Vec<_>>()
        .join("\n");

    if input.ends_with('\n') {
        format!("{stripped}\n")
    } else {
        stripped
    }
}

fn has_copied_gutter(line: &str) -> bool {
    let trimmed = line.trim_start_matches(char::is_whitespace);
    let mut chars = trimmed.chars();
    let Some(marker) = chars.next() else {
        return false;
    };

    is_copied_gutter_marker(marker) && chars.next().is_none_or(char::is_whitespace)
}

fn strip_leading_gutter(line: &str) -> String {
    let leading_whitespace_len = line.len() - line.trim_start_matches(char::is_whitespace).len();
    let mut chars = line[leading_whitespace_len..].chars();
    let Some(marker) = chars.next() else {
        return line.to_string();
    };

    if !is_copied_gutter_marker(marker) {
        return line.to_string();
    }

    let marker_len = marker.len_utf8();
    let marker_index = leading_whitespace_len;
    format!(
        "{}{}",
        &line[..marker_index],
        &line[marker_index + marker_len..]
    )
}

fn is_copied_gutter_marker(ch: char) -> bool {
    matches!(ch, '│' | '┃' | '▏' | '▕' | '❘' | '¦')
}

fn common_whitespace_prefix(left: &str, right: &str) -> String {
    left.chars()
        .zip(right.chars())
        .take_while(|(left, right)| left == right)
        .map(|(ch, _)| ch)
        .collect()
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
    let input = strip_uniform_copied_margin(input);
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

#[cfg(test)]
mod tests {
    use super::{strip_uniform_copied_margin, strip_uniform_leading_whitespace};

    #[test]
    fn strips_shared_margin_and_preserves_relative_indentation() {
        let input = "   cat <<'EOF'\n   public class Main {\n       System.out.println(\"hi\");\n   }\n   EOF\n";

        assert_eq!(
            strip_uniform_leading_whitespace(input),
            "cat <<'EOF'\npublic class Main {\n    System.out.println(\"hi\");\n}\nEOF\n"
        );
    }

    #[test]
    fn leaves_mixed_indentation_without_shared_margin_unchanged() {
        let input = "Review this carefully:\n\n    cargo test\n    cargo clippy\n";

        assert_eq!(strip_uniform_leading_whitespace(input), input);
    }

    #[test]
    fn strips_uniform_copied_gutter_prefix() {
        let input = "│ First line\n│\n│ Second line\n";

        assert_eq!(
            strip_uniform_copied_margin(input),
            "First line\n\nSecond line\n"
        );
    }
}
