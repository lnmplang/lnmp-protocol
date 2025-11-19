use lnmp_codec::config::ParserConfig;
use lnmp_codec::{Parser, ParsingMode, TextInputMode};
use lnmp_core::LnmpRecord;
use proptest::prelude::*;

#[derive(Debug, Clone)]
enum FieldValue {
    Int(i32),
    Str(String),
    StrArray(Vec<String>),
}

#[derive(Debug, Clone)]
struct FieldCase {
    fid: u16,
    value: FieldValue,
    drop_quotes: bool,
    unterminated_quote: bool,
    add_trailing_backslash: bool,
    space_before_eq: bool,
    space_after_eq: bool,
    use_newline: bool,
    use_crlf: bool,
    double_semicolon: bool,
    comment_before: bool,
    comment_after: bool,
    zero_pad: bool,
}

impl FieldCase {
    fn canonical_repr(&self) -> String {
        format!("F{}={}", self.fid, self.canonical_value())
    }

    fn canonical_value(&self) -> String {
        match &self.value {
            FieldValue::Int(v) => v.to_string(),
            FieldValue::Str(s) => quote_string(s),
            FieldValue::StrArray(items) => {
                let rendered: Vec<String> =
                    items.iter().map(|item| format_array_item(item)).collect();
                format!("[{}]", rendered.join(","))
            }
        }
    }

    fn lenient_value(&self) -> String {
        match &self.value {
            FieldValue::Int(v) => {
                if self.zero_pad && *v >= 0 {
                    format!("{:03}", v)
                } else {
                    v.to_string()
                }
            }
            FieldValue::Str(raw) => {
                let needs_quotes = raw
                    .chars()
                    .any(|c| c.is_whitespace() || matches!(c, ';' | '"' | '\\' | ',' | '#'));
                let mut val = if self.drop_quotes && needs_quotes {
                    raw.clone()
                } else {
                    quote_string(raw)
                };

                if self.unterminated_quote && val.ends_with('"') {
                    val.pop();
                }

                if self.add_trailing_backslash {
                    val.push('\\');
                }

                val
            }
            FieldValue::StrArray(items) => {
                let mut rendered = Vec::with_capacity(items.len());
                for (idx, item) in items.iter().enumerate() {
                    let mut elem = if self.drop_quotes {
                        item.clone()
                    } else {
                        format_array_item(item)
                    };

                    if self.unterminated_quote && elem.ends_with('"') && idx == items.len() - 1 {
                        elem.pop();
                    }

                    if self.add_trailing_backslash && idx == items.len() - 1 {
                        elem.push('\\');
                    }

                    rendered.push(elem);
                }

                let sep = if self.space_after_eq { ", " } else { "," };
                format!("[{}]", rendered.join(sep))
            }
        }
    }
}

fn quote_string(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len() + 2);
    out.push('"');
    for ch in raw.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

fn format_array_item(raw: &str) -> String {
    if raw
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
    {
        raw.to_string()
    } else {
        quote_string(raw)
    }
}

fn build_canonical_text(fields: &[FieldCase]) -> String {
    fields
        .iter()
        .map(|f| f.canonical_repr())
        .collect::<Vec<_>>()
        .join("\n")
}

fn build_lenient_text(fields: &[FieldCase]) -> String {
    let mut out = String::new();
    for (idx, field) in fields.iter().enumerate() {
        if idx > 0 {
            if field.use_newline {
                if field.use_crlf {
                    out.push_str("\r\n");
                } else {
                    out.push('\n');
                }
            } else {
                out.push(';');
            }
        }

        if field.comment_before {
            out.push_str("# comment\n");
        }

        out.push('F');
        out.push_str(&field.fid.to_string());
        if field.space_before_eq {
            out.push(' ');
        }
        out.push('=');
        if field.space_after_eq {
            out.push(' ');
        }

        out.push_str(&field.lenient_value());

        if field.double_semicolon {
            out.push_str(";;");
        }

        if field.comment_after {
            out.push_str("\n# trailing");
        }
    }
    out
}

fn parse_with_profile(
    text: &str,
    text_mode: TextInputMode,
    mode: ParsingMode,
) -> Option<LnmpRecord> {
    let config = ParserConfig {
        text_input_mode: text_mode,
        mode,
        normalize_values: true,
        ..ParserConfig::default()
    };
    let mut parser = Parser::with_config(text, config).ok()?;
    parser.parse_record().ok()
}

const TEST_CHARS: &[char] = &['a', 'b', 'c', ' ', '"', '\\', ';', ',', '\t'];

fn str_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(TEST_CHARS.to_vec()), 0..8)
        .prop_map(|chars| chars.into_iter().collect())
}

fn field_case_strategy() -> impl Strategy<Value = FieldCase> {
    (
        1u16..200u16,
        any::<bool>(), // use string
        any::<bool>(), // use array
        str_strategy(),
        any::<i16>(),
        prop::collection::vec(str_strategy(), 0..4),
        prop::collection::vec(any::<bool>(), 11),
    )
        .prop_map(
            |(fid, use_str, use_array, str_val, int_val, array_items, flags)| FieldCase {
                fid,
                value: if use_array {
                    FieldValue::StrArray(array_items)
                } else if use_str {
                    FieldValue::Str(str_val)
                } else {
                    FieldValue::Int(int_val as i32)
                },
                drop_quotes: flags.get(0).copied().unwrap_or(false),
                unterminated_quote: flags.get(1).copied().unwrap_or(false),
                add_trailing_backslash: flags.get(2).copied().unwrap_or(false),
                space_before_eq: flags.get(3).copied().unwrap_or(false),
                space_after_eq: flags.get(4).copied().unwrap_or(false),
                use_newline: flags.get(5).copied().unwrap_or(false),
                use_crlf: flags.get(6).copied().unwrap_or(false),
                double_semicolon: flags.get(7).copied().unwrap_or(false),
                comment_before: flags.get(8).copied().unwrap_or(false),
                comment_after: flags.get(9).copied().unwrap_or(false),
                zero_pad: flags.get(10).copied().unwrap_or(false),
            },
        )
}

fn record_strategy() -> impl Strategy<Value = Vec<FieldCase>> {
    prop::collection::vec(field_case_strategy(), 1..6)
}

proptest! {
    #[test]
    fn lenient_sanitizer_matches_canonical(fields in record_strategy()) {
        let canonical_text = build_canonical_text(&fields);
        let lenient_text = build_lenient_text(&fields);

        let canonical = parse_with_profile(&canonical_text, TextInputMode::Strict, ParsingMode::Loose).expect("canonical parse");
        let lenient = match parse_with_profile(&lenient_text, TextInputMode::Lenient, ParsingMode::Loose) {
            Some(record) => record,
            None => return Ok(()), // discard unparseable fuzz cases
        };

        prop_assert_eq!(lenient.sorted_fields(), canonical.sorted_fields());
    }
}
