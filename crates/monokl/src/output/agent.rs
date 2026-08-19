use michi::toon::Value;

use crate::error::MonoklError;
use crate::types::{SymbolKind, Visibility};

/// Carries the name of the CLI (sub)command currently executing, so
/// `RecoveryHint::new()` can name the correct tool for an agent to retry.
pub struct ErrorContext<'a> {
    pub command: &'a str,
}

/// The only sanctioned way to convert a `&str` into a `michi::toon::Value`
/// inside this module. Pre-escapes `\n`/`\r` before handing the string to
/// michi, since michi-toon's own `escape_value` deletes them silently.
pub fn cell(s: &str) -> Value {
    if s.contains('\n') || s.contains('\r') {
        let mut out = String::with_capacity(s.len());
        for ch in s.chars() {
            match ch {
                '\r' => {}
                '\n' => out.push_str("\\n"),
                _ => out.push(ch),
            }
        }
        Value::from(out)
    } else {
        Value::from(s)
    }
}

pub fn opt_cell(s: Option<&str>) -> Value {
    match s {
        None => Value::Null,
        Some(s) => cell(s),
    }
}

#[cfg(test)]
mod cell_tests {
    use super::*;

    #[test]
    fn cell_plain_string_passes_through() {
        assert_eq!(cell("hello"), Value::from("hello"));
    }

    #[test]
    fn cell_replaces_lf_with_literal_backslash_n() {
        let v = cell("line1\nline2");
        assert_eq!(v, Value::from("line1\\nline2"));
    }

    #[test]
    fn cell_drops_cr_and_replaces_lf() {
        let v = cell("line1\r\nline2");
        assert_eq!(v, Value::from("line1\\nline2"));
    }

    #[test]
    fn cell_drops_bare_cr() {
        let v = cell("line1\rline2");
        assert_eq!(v, Value::from("line1line2"));
    }

    #[test]
    fn opt_cell_none_is_null() {
        assert_eq!(opt_cell(None), Value::Null);
    }

    #[test]
    fn opt_cell_some_empty_is_not_null() {
        assert_eq!(opt_cell(Some("")), Value::from(""));
        assert_ne!(opt_cell(Some("")), Value::Null);
    }

    /// AC-001: verifies michi-toon's own `escape_value` deletes `\n`/`\r`
    /// rather than escaping them -- the exact defect `cell()` exists to
    /// route around. If michi's pinned rev ever fixes this, this test fails
    /// loudly rather than the workaround silently going stale.
    #[test]
    fn ac001_michi_escape_value_deletes_newlines() {
        let raw = "// comment\nfn body() {\n    real_code();\n}";
        let escaped = michi::toon::escape_value(raw);
        assert!(!escaped.contains('\n'), "expected newline deletion, got: {escaped:?}");
        assert_eq!(escaped, "// commentfn body() {    real_code();}");
    }

}
