use std::borrow::Cow;

use crate::mode::SanitizationLevel;

/// Configuration options for sanitization.
#[derive(Debug, Clone)]
pub struct SanitizationConfig {
    /// Overall repair level for heuristics
    pub level: SanitizationLevel,
    /// Automatically wrap string-like segments with quotes when needed
    pub auto_quote_strings: bool,
    /// Escape stray quotes inside text sections
    pub auto_escape_quotes: bool,
    /// Normalize boolean text representations to 1/0 outside quotes
    pub normalize_booleans: bool,
    /// Normalize simple numeric forms (e.g., remove leading zeros)
    pub normalize_numbers: bool,
}

impl Default for SanitizationConfig {
    fn default() -> Self {
        Self {
            level: SanitizationLevel::Normal,
            auto_quote_strings: true,
            auto_escape_quotes: true,
            normalize_booleans: true,
            normalize_numbers: false,
        }
    }
}

/// Leniently sanitizes LNMP-like text. When no changes are required the input is returned
/// by reference to avoid allocations.
pub fn sanitize_lnmp_text<'a>(input: &'a str, config: &SanitizationConfig) -> Cow<'a, str> {
    let mut changed = false;

    // Pass 1: whitespace/structural cleanup
    let pass1 = structural_cleanup(input, config, &mut changed);

    // Pass 2: quote/escape repair + optional auto-quoting
    let pass2 = if config.level == SanitizationLevel::Minimal {
        pass1
    } else {
        let quote_fixed = quote_and_escape_repair(&pass1, config, &mut changed);
        if config.auto_quote_strings {
            Cow::Owned(auto_quote_unquoted_values(
                quote_fixed.as_ref(),
                &mut changed,
            ))
        } else {
            quote_fixed
        }
    };

    // Pass 3: semantic normalization (Aggressive only)
    let pass3 = if config.level == SanitizationLevel::Aggressive
        && (config.normalize_booleans || config.normalize_numbers)
    {
        Cow::Owned(normalize_tokens(&pass2, config, &mut changed))
    } else {
        pass2
    };

    if changed {
        Cow::Owned(pass3.into_owned())
    } else {
        Cow::Borrowed(input)
    }
}

fn structural_cleanup<'a>(
    input: &'a str,
    config: &SanitizationConfig,
    changed: &mut bool,
) -> Cow<'a, str> {
    // Minimal mode: only newline normalization and trailing space trim.
    if config.level == SanitizationLevel::Minimal {
        let mut output = String::with_capacity(input.len());
        for line in input.lines() {
            let trimmed = line.trim_end_matches([' ', '\t']);
            if trimmed.len() != line.len() {
                *changed = true;
            }
            output.push_str(trimmed);
            output.push('\n');
        }
        if !input.ends_with('\n') && !input.is_empty() {
            output.pop();
        }

        if *changed {
            return Cow::Owned(output);
        }
        return Cow::Borrowed(input);
    }

    let mut output = String::with_capacity(input.len());
    let mut in_quotes = false;
    let mut escape_next = false;
    let mut last_emitted: Option<char> = None;

    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if escape_next {
            output.push(ch);
            last_emitted = Some(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => {
                output.push('\\');
                match chars.peek() {
                    Some('"' | '\\' | 'n' | 'r' | 't') => {
                        escape_next = true;
                    }
                    Some(_) if in_quotes && config.auto_escape_quotes => {
                        escape_next = true;
                        *changed = true;
                    }
                    None => {
                        output.push('\\');
                        *changed = true;
                    }
                    _ => {}
                }
                last_emitted = Some('\\');
            }
            '"' => {
                in_quotes = !in_quotes;
                output.push('"');
                last_emitted = Some('"');
            }
            ';' if !in_quotes => {
                output.push(';');
                last_emitted = Some(';');
                while matches!(chars.peek(), Some(c) if c.is_whitespace()) {
                    chars.next();
                    *changed = true;
                }
            }
            ',' if !in_quotes => {
                output.push(',');
                last_emitted = Some(',');
                while matches!(chars.peek(), Some(c) if c.is_whitespace()) {
                    chars.next();
                    *changed = true;
                }
            }
            '\n' => {
                while output.ends_with(' ') || output.ends_with('\t') {
                    output.pop();
                    *changed = true;
                }
                output.push('\n');
                last_emitted = Some('\n');
            }
            '\r' => {
                *changed = true;
                output.push('\n');
                last_emitted = Some('\n');
            }
            ' ' | '\t' if !in_quotes => {
                let next_non_space = {
                    let mut clone = chars.clone();
                    clone.find(|c| *c != ' ' && *c != '\t')
                };

                let prev_is_boundary = matches!(
                    last_emitted,
                    None | Some('\n' | ';' | ',' | '=' | '[' | '{')
                );
                let next_is_boundary = matches!(
                    next_non_space,
                    None | Some('\n' | ';' | ',' | '=' | ']' | '}')
                );

                if prev_is_boundary || next_is_boundary {
                    *changed = true;
                    continue;
                }

                if last_emitted == Some(' ') {
                    *changed = true;
                    continue;
                }

                output.push(' ');
                last_emitted = Some(' ');
            }
            other => {
                output.push(other);
                last_emitted = Some(other);
            }
        }
    }

    if in_quotes && config.auto_escape_quotes {
        output.push('"');
        *changed = true;
    }

    if *changed {
        Cow::Owned(output)
    } else {
        Cow::Borrowed(input)
    }
}

fn quote_and_escape_repair<'a>(
    input: &'a str,
    config: &SanitizationConfig,
    changed: &mut bool,
) -> Cow<'a, str> {
    let mut output = String::with_capacity(input.len());
    let mut in_quotes = false;
    let mut escape_next = false;

    for ch in input.chars() {
        if escape_next {
            output.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => {
                output.push('\\');
                escape_next = true;
            }
            '"' => {
                in_quotes = !in_quotes;
                output.push('"');
            }
            _ => {
                output.push(ch);
            }
        }
    }

    if in_quotes && config.auto_escape_quotes {
        output.push('"');
        *changed = true;
    }

    if *changed {
        Cow::Owned(output)
    } else {
        Cow::Borrowed(input)
    }
}

fn auto_quote_unquoted_values(input: &str, changed: &mut bool) -> String {
    let mut output = String::with_capacity(input.len());
    let mut iter = input.char_indices().peekable();
    while let Some((idx, ch)) = iter.next() {
        if ch == '=' {
            output.push('=');

            let value_start = idx + ch.len_utf8();
            let mut value_end = value_start;
            let mut in_quotes = false;
            let mut escape_next = false;

            while let Some(&(next_idx, next_ch)) = iter.peek() {
                if escape_next {
                    escape_next = false;
                    iter.next();
                    value_end = next_idx + next_ch.len_utf8();
                    continue;
                }
                match next_ch {
                    '\\' => {
                        escape_next = true;
                        iter.next();
                        value_end = next_idx + next_ch.len_utf8();
                    }
                    '"' => {
                        in_quotes = !in_quotes;
                        iter.next();
                        value_end = next_idx + next_ch.len_utf8();
                    }
                    ';' | '\n' if !in_quotes => break,
                    _ => {
                        iter.next();
                        value_end = next_idx + next_ch.len_utf8();
                    }
                }
            }

            let value = &input[value_start..value_end];
            let trimmed = value.trim();
            let starts_structural = trimmed.starts_with('[') || trimmed.starts_with('{');
            let needs_quotes = !trimmed.is_empty()
                && !trimmed.starts_with('"')
                && !starts_structural
                && (trimmed.contains('"') || trimmed.chars().any(char::is_whitespace));

            if needs_quotes {
                let mut escaped = String::with_capacity(value.len() + 4);
                for ch in value.chars() {
                    match ch {
                        '"' => {
                            escaped.push_str("\\\"");
                        }
                        '\\' => {
                            escaped.push_str("\\\\");
                        }
                        _ => escaped.push(ch),
                    }
                }
                output.push('"');
                output.push_str(escaped.trim());
                output.push('"');
                *changed = true;
            } else {
                output.push_str(value);
            }

            if let Some(&(_, delim)) = iter.peek() {
                if delim == ';' || delim == '\n' {
                    output.push(delim);
                    iter.next();
                }
            }
        } else {
            output.push(ch);
        }
    }

    output
}

fn normalize_tokens(input: &str, config: &SanitizationConfig, changed: &mut bool) -> String {
    let mut out = String::with_capacity(input.len());
    let mut token = String::new();
    let mut in_quotes = false;
    let mut escape_next = false;

    for ch in input.chars() {
        if escape_next {
            out.push(ch);
            escape_next = false;
            continue;
        }

        if ch == '\\' && in_quotes {
            out.push('\\');
            escape_next = true;
            continue;
        }

        if ch == '"' {
            flush_token(&mut token, &mut out, config, changed);
            in_quotes = !in_quotes;
            out.push('"');
            continue;
        }

        if in_quotes {
            out.push(ch);
            continue;
        }

        if ch.is_ascii_alphanumeric() || ch == '-' {
            token.push(ch);
        } else {
            flush_token(&mut token, &mut out, config, changed);
            out.push(ch);
        }
    }

    flush_token(&mut token, &mut out, config, changed);
    out
}

fn flush_token(
    token: &mut String,
    out: &mut String,
    config: &SanitizationConfig,
    changed: &mut bool,
) {
    if token.is_empty() {
        return;
    }

    let mut replacement: Option<String> = None;

    if config.normalize_booleans {
        match token.to_ascii_lowercase().as_str() {
            "true" | "yes" => replacement = Some("1".to_string()),
            "false" | "no" => replacement = Some("0".to_string()),
            _ => {}
        }
    }

    if replacement.is_none()
        && config.normalize_numbers
        && token.len() > 1
        && token.chars().all(|c| c.is_ascii_digit())
        && token.starts_with('0')
    {
        let trimmed = token.trim_start_matches('0');
        let normalized = if trimmed.is_empty() { "0" } else { trimmed };
        replacement = Some(normalized.to_string());
    }

    if let Some(ref value) = replacement {
        *changed |= value != token;
        out.push_str(value);
    } else {
        out.push_str(token);
    }

    token.clear();
}
