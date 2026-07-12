//! String sanitization and secret redaction boundary.
//!
//! Every dynamic string entering the animated UI MUST pass through here.
//! §11: remove ANSI/OSC/DCS/APC/PM/C1 controls, remove control chars,
//! bound length, redact secrets, never inject terminal escapes.

use std::borrow::Cow;

const MAX_VISIBLE_LEN: usize = 512;

/// Redact known secret patterns and remove terminal escape sequences.
///
/// Returns the sanitized string; if no changes needed, returns `Cow::Borrowed`.
pub fn sanitize(input: &str) -> Cow<'_, str> {
    if input.is_empty() {
        return Cow::Borrowed(input);
    }

    // Phase 1: strip terminal control sequences first
    let s = strip_ansi(input);

    // Phase 2: strip remaining control characters except normal whitespace
    let s = strip_controls(&s);

    // Phase 3: redact secrets
    let s = redact_secrets(&s);

    // Phase 4: bound length
    let s = if s.len() > MAX_VISIBLE_LEN {
        let mut truncated = s[..MAX_VISIBLE_LEN].to_string();
        truncated.push('…');
        truncated
    } else {
        s
    };

    if s == input {
        Cow::Borrowed(input)
    } else {
        Cow::Owned(s)
    }
}

fn redact_secrets(input: &str) -> String {
    let mut s = input.to_string();

    // License key patterns: focusa_live_* and focusa_test_*
    s = replace_token_pattern(&s, "focusa_live_", alphanumeric_underscore_dash);
    s = replace_token_pattern(&s, "focusa_test_", alphanumeric_underscore_dash);

    // Authorization: Bearer <token>
    s = replace_after_prefix(&s, "Authorization: Bearer ", space_bound);
    s = replace_after_prefix(&s, "authorization: bearer ", space_bound);

    // Generic API keys / tokens / secrets in query strings
    s = replace_query_param(&s, "api_key");
    s = replace_query_param(&s, "apikey");
    s = replace_query_param(&s, "token");
    s = replace_query_param(&s, "secret");
    s = replace_query_param(&s, "password");

    // AWS access keys
    s = replace_aws_key(&s);

    // npm tokens
    s = replace_after_prefix(&s, "npm_", npm_token_bound);

    s
}

fn alphanumeric_underscore_dash(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-'
}

fn space_bound(c: char) -> bool {
    !c.is_whitespace()
}

fn npm_token_bound(c: char) -> bool {
    c.is_alphanumeric()
}

fn replace_token_pattern(input: &str, prefix: &str, bound: fn(char) -> bool) -> String {
    let mut result = String::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        if let Some(pos) = input[i..].find(prefix) {
            let start = i + pos;
            result.push_str(&input[i..start]);
            let after = start + prefix.len();
            let end = input[after..]
                .chars()
                .take_while(|&c| bound(c))
                .map(|c| c.len_utf8())
                .sum::<usize>()
                + after;
            if end > after {
                result.push_str("[REDACTED_LICENSE_KEY]");
            } else {
                result.push_str(prefix);
            }
            i = end;
        } else {
            result.push_str(&input[i..]);
            break;
        }
    }
    result
}

fn replace_after_prefix(input: &str, prefix: &str, bound: fn(char) -> bool) -> String {
    let mut result = String::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        if let Some(pos) = input[i..].find(prefix) {
            let start = i + pos;
            result.push_str(&input[i..start]);
            let after = start + prefix.len();
            let end = input[after..]
                .chars()
                .take_while(|&c| bound(c))
                .map(|c| c.len_utf8())
                .sum::<usize>()
                + after;
            if end > after {
                result.push_str(prefix);
                result.push_str("[REDACTED_TOKEN]");
            } else {
                result.push_str(prefix);
            }
            i = end;
        } else {
            result.push_str(&input[i..]);
            break;
        }
    }
    result
}

fn replace_query_param(input: &str, name: &str) -> String {
    let prefix1 = format!("?{name}=");
    let prefix2 = format!("&{name}=");
    replace_after_prefix(
        &replace_after_prefix(input, &prefix1, |c| c != '&' && !c.is_whitespace()),
        &prefix2,
        |c| c != '&' && !c.is_whitespace(),
    )
}

fn replace_aws_key(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut i = 0;
    while i + 20 <= input.len() {
        if input[i..].starts_with("AKIA") && input[i + 4..i + 20].chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) {
            result.push_str("[REDACTED_AWS_KEY]");
            i += 20;
        } else {
            result.push(input[i..].chars().next().unwrap());
            i += input[i..].chars().next().unwrap().len_utf8();
        }
    }
    result.push_str(&input[i..]);
    result
}

fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            match chars.peek() {
                Some('[') => {
                    // CSI sequence: ESC [ ... byte in 0x40-0x7E
                    chars.next();
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if (0x40..=0x7E).contains(&(c as u32)) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    // OSC sequence: ESC ] ... BEL or ESC \
                    chars.next();
                    while let Some(c) = chars.next() {
                        if c == '\x07' {
                            break;
                        }
                        if c == '\x1b' {
                            if let Some('\\') = chars.peek() {
                                chars.next();
                            }
                            break;
                        }
                    }
                }
                Some('P') => {
                    // DCS sequence: ESC P ... ESC \
                    chars.next();
                    while let Some(c) = chars.next() {
                        if c == '\x1b' {
                            if let Some('\\') = chars.peek() {
                                chars.next();
                            }
                            break;
                        }
                    }
                }
                Some('^') | Some('_') | Some('\\') => {
                    // APC, PM, C1
                    chars.next();
                }
                _ => {
                    // Unknown escape; drop the ESC
                }
            }
        } else {
            out.push(ch);
        }
    }

    out
}

fn strip_controls(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '\t' => ' ',
            '\n' => ' ',
            '\r' => ' ',
            c if c.is_control() => '�',
            c => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ansi_clear_screen_removed() {
        let malicious = "\x1b[2Jforged success";
        assert_eq!(sanitize(malicious), "forged success");
    }

    #[test]
    fn hyperlinks_removed() {
        let malicious = "\x1b]8;;https://evil.invalid\x07click\x1b]8;;\x07";
        let result = sanitize(malicious);
        assert!(!result.contains('\x1b'));
        assert!(result.contains("click"));
    }

    #[test]
    fn license_key_redacted() {
        let input = "focusa_live_super_secret_key_123";
        let result = sanitize(input);
        assert!(result.contains("REDACTED"));
        assert!(!result.contains("super_secret"));
    }

    #[test]
    fn authorization_redacted() {
        let input = "Authorization: Bearer secret_token_xyz";
        let result = sanitize(input);
        assert!(!result.contains("secret_token"));
        assert!(result.contains("REDACTED"));
    }

    #[test]
    fn normal_text_unchanged() {
        let input = "normal text without secrets";
        assert_eq!(sanitize(input), input);
    }

    #[test]
    fn control_chars_replaced() {
        let input = "normal\n✗ fake failure";
        let result = sanitize(input);
        assert!(!result.contains('\n'));
        assert!(result.contains('✗'));
    }

    #[test]
    fn truncation_bounds_length() {
        let long = "a".repeat(600);
        let result = sanitize(&long);
        assert!(result.len() <= MAX_VISIBLE_LEN + 3);
        assert!(result.ends_with('…'));
    }

    #[test]
    fn aws_key_redacted() {
        let input = "AKIAIOSFODNN7EXAMPLE";
        let result = sanitize(input);
        assert!(result.contains("REDACTED_AWS_KEY"));
    }
}
