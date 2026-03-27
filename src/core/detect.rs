pub(crate) fn looks_like_label_plus_command(input: &str) -> bool {
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

pub(crate) fn looks_like_command_line_after_label(line: &str) -> bool {
    let mut parts = line.split_whitespace();
    let Some(program) = parts.next() else {
        return false;
    };

    program
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/'))
        && parts.next().is_some()
}

pub(crate) fn looks_like_prose(input: &str) -> bool {
    if looks_like_yaml_mapping_input(input) {
        return false;
    }

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

pub(crate) fn looks_like_soft_wrapped_prose(input: &str) -> bool {
    let lines: Vec<_> = input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    lines.len() >= 3
        && lines.iter().all(|line| {
            !looks_like_shell_line(line)
                && !is_protected_line(line)
                && !looks_like_aligned_columns_line(line)
                && line.contains(' ')
        })
}

pub(crate) fn looks_like_yaml_mapping_input(input: &str) -> bool {
    let lines: Vec<_> = input
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty())
        .collect();

    if lines.len() < 3 {
        return false;
    }

    let mapping_lines = lines
        .iter()
        .filter(|line| looks_like_yaml_mapping_line(line))
        .count();
    let top_level_lines = lines
        .iter()
        .filter(|line| !line.starts_with(char::is_whitespace) && looks_like_yaml_mapping_line(line))
        .count();
    let nested_lines = lines
        .iter()
        .filter(|line| line.starts_with(char::is_whitespace) && looks_like_yaml_mapping_line(line))
        .count();

    let has_multi_root_shape = top_level_lines >= 2 && nested_lines >= 1;
    let has_single_root_nested_shape = top_level_lines == 1
        && nested_lines >= 2
        && lines.first().is_some_and(|line| line.trim().ends_with(':'));

    (has_multi_root_shape || has_single_root_nested_shape) && mapping_lines * 2 >= lines.len()
}

fn looks_like_yaml_mapping_line(line: &str) -> bool {
    let trimmed = line.trim();

    if trimmed.is_empty()
        || trimmed.starts_with("```")
        || trimmed.starts_with('>')
        || trimmed.starts_with('#')
        || trimmed.starts_with('-')
    {
        return false;
    }

    let Some((key, rest)) = trimmed.split_once(':') else {
        return false;
    };

    if key.is_empty() || key.chars().any(char::is_whitespace) {
        return false;
    }

    rest.is_empty() || rest.starts_with(' ')
}

pub(crate) fn looks_like_reaction_snippet(input: &str) -> bool {
    let mut saw_non_empty = false;

    for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        saw_non_empty = true;

        if !is_reaction_line(line) {
            return false;
        }
    }

    saw_non_empty
}

pub(crate) fn looks_like_command_transcript(input: &str) -> bool {
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

pub(crate) fn looks_like_command(input: &str) -> bool {
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

pub(crate) fn looks_like_shell_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.contains('`') {
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
        || trimmed.starts_with("systemctl ")
        || trimmed.starts_with("journalctl ")
        || trimmed.starts_with("waytrim ")
        || trimmed.starts_with("waytrim-watch ")
        || trimmed.starts_with("waytrimctl ")
        || trimmed.starts_with("rm ")
        || trimmed.starts_with("cp ")
        || trimmed.starts_with("mv ")
        || trimmed.starts_with("cd ")
        || trimmed.starts_with("ls ")
        || trimmed.starts_with("echo ")
}

pub(crate) fn is_command_continuation_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('-') || trimmed.starts_with('\\')
}

pub(crate) fn is_command_block_continuation_line(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty()
        && line.starts_with(char::is_whitespace)
        && (trimmed.starts_with("--") || trimmed.starts_with('\\'))
}

pub(crate) fn strip_prompt(line: &str) -> Option<&str> {
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

pub(crate) fn blockquote_prefix(trimmed: &str) -> Option<&str> {
    Some(trimmed.strip_prefix('>')?.trim_start())
}

pub(crate) fn is_blockquote_continuation_line(line: &str) -> bool {
    line.starts_with(char::is_whitespace) && !line.trim().is_empty()
}

pub(crate) fn is_protected_line(line: &str) -> bool {
    let trimmed = line.trim();

    trimmed.starts_with("```")
        || trimmed.starts_with('>')
        || trimmed.starts_with('#')
        || line.starts_with("    ")
        || line.starts_with('\t')
        || is_bullet_line(trimmed)
        || is_numbered_line(trimmed)
        || is_reaction_line(trimmed)
}

pub(crate) fn looks_like_aligned_columns_line(line: &str) -> bool {
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

pub(crate) fn is_bullet_line(trimmed: &str) -> bool {
    ["- ", "* ", "+ "]
        .iter()
        .any(|marker| trimmed.starts_with(marker))
}

pub(crate) fn is_list_item_line(trimmed: &str) -> bool {
    is_bullet_line(trimmed) || is_numbered_line(trimmed)
}

pub(crate) fn is_list_continuation_line(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty() && line.starts_with(char::is_whitespace) && !is_protected_line(line)
}

pub(crate) fn is_numbered_line(trimmed: &str) -> bool {
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

fn is_reaction_line(trimmed: &str) -> bool {
    looks_like_shortcode_reaction_line(trimmed) || looks_like_emoji_only_line(trimmed)
}

fn looks_like_shortcode_reaction_line(trimmed: &str) -> bool {
    let mut saw_token = false;

    for token in trimmed.split_whitespace() {
        saw_token = true;

        let Some(inner) = token
            .strip_prefix(':')
            .and_then(|value| value.strip_suffix(':'))
        else {
            return false;
        };

        if inner.is_empty() {
            return false;
        }

        if !inner.chars().all(|ch| {
            ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '_' | '-' | '+')
        }) {
            return false;
        }
    }

    saw_token
}

fn looks_like_emoji_only_line(trimmed: &str) -> bool {
    let mut saw_non_whitespace = false;

    for ch in trimmed.chars() {
        if ch.is_whitespace() {
            continue;
        }

        saw_non_whitespace = true;

        if ch.is_ascii() || ch.is_alphanumeric() {
            return false;
        }
    }

    saw_non_whitespace
}

pub(crate) fn should_collapse_blank_line_between(previous: &str, next: &str) -> bool {
    let previous = previous.trim();
    let next = next.trim();

    previous.contains(' ')
        && next.contains(' ')
        && next
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_lowercase())
        && !matches!(previous.chars().last(), Some('.' | '!' | '?' | ':'))
        && !is_protected_line(previous)
        && !is_protected_line(next)
        && !looks_like_shell_line(previous)
        && !looks_like_shell_line(next)
        && !looks_like_aligned_columns_line(previous)
        && !looks_like_aligned_columns_line(next)
}
