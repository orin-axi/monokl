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

/// `SymbolKind` has no `as_str()`/`Display` of its own (SPEC-003 AC-001);
/// this returns the same camelCase strings serde's `rename_all = "camelCase"`
/// already produces for it (SPEC-003 AC-002), via a direct, exhaustive
/// match with no default arm.
pub fn symbol_kind_str(k: SymbolKind) -> &'static str {
    match k {
        SymbolKind::Function => "function",
        SymbolKind::Method => "method",
        SymbolKind::Constructor => "constructor",
        SymbolKind::Class => "class",
        SymbolKind::Struct => "struct",
        SymbolKind::Enum => "enum",
        SymbolKind::Interface => "interface",
        SymbolKind::TypeAlias => "typeAlias",
        SymbolKind::Property => "property",
        SymbolKind::Field => "field",
        SymbolKind::Variable => "variable",
        SymbolKind::Module => "module",
        SymbolKind::Impl => "impl",
        SymbolKind::Macro => "macro",
        SymbolKind::Other => "other",
    }
}

/// `Visibility` has no `as_str()`/`Display` of its own (SPEC-003 AC-003);
/// same rationale as `symbol_kind_str`.
pub fn visibility_str(v: Visibility) -> &'static str {
    match v {
        Visibility::Public => "public",
        Visibility::Crate => "crate",
        Visibility::Module => "module",
        Visibility::Private => "private",
    }
}

#[cfg(test)]
mod kind_str_tests {
    use super::*;

    #[test]
    fn symbol_kind_str_matches_serde_camel_case_form() {
        assert_eq!(symbol_kind_str(SymbolKind::TypeAlias), "typeAlias");
        assert_eq!(symbol_kind_str(SymbolKind::Other), "other");
        assert_eq!(symbol_kind_str(SymbolKind::Function), "function");
    }

    #[test]
    fn visibility_str_matches_serde_camel_case_form() {
        assert_eq!(visibility_str(Visibility::Public), "public");
        assert_eq!(visibility_str(Visibility::Module), "module");
    }
}

pub trait ToonProjection<const N: usize> {
    const TYPE_NAME: &'static str;
    const FIELDS: [&'static str; N];
    fn toon_rows(&self) -> Vec<[Value; N]>;
    fn toon_total_count(&self) -> Option<usize> {
        None
    }
    fn toon_hints(&self) -> Vec<michi::Hint> {
        Vec::new()
    }
}
impl ToonProjection<9> for crate::types::SymbolsResult {
    const TYPE_NAME: &'static str = "symbols";
    const FIELDS: [&'static str; 9] =
        ["file", "name", "kind", "line", "signature", "owner", "traitImpl", "visibility", "kindDetail"];

    fn toon_rows(&self) -> Vec<[Value; 9]> {
        let mut rows = Vec::new();
        for (file, entries) in &self.files {
            for e in entries {
                rows.push([
                    cell(file.as_str()),
                    cell(&e.name),
                    cell(symbol_kind_str(e.kind)),
                    Value::from(e.line),
                    opt_cell(e.signature.as_deref()),
                    opt_cell(e.owner.as_deref()),
                    opt_cell(e.trait_impl.as_deref()),
                    opt_cell(e.visibility.map(visibility_str)),
                    opt_cell(e.kind_detail.as_deref()),
                ]);
            }
        }
        rows
    }
}
impl ToonProjection<7> for crate::types::SearchResponse {
    const TYPE_NAME: &'static str = "search";
    const FIELDS: [&'static str; 7] =
        ["file", "lineStart", "lineEnd", "nodeKind", "symbolSignature", "rank", "finalScore"];

    fn toon_rows(&self) -> Vec<[Value; 7]> {
        self.results
            .iter()
            .map(|block| {
                [
                    cell(block.block.file.as_str()),
                    Value::from(block.block.line_start),
                    Value::from(block.block.line_end),
                    cell(symbol_kind_str(block.block.node_kind)),
                    opt_cell(block.block.symbol_signature.as_deref()),
                    Value::from(block.rank),
                    Value::from(block.final_score),
                ]
            })
            .collect()
    }

    fn toon_hints(&self) -> Vec<michi::Hint> {
        vec![michi::Hint::new(
            "use extract with file/lineStart/lineEnd to retrieve a row's actual code content",
        )]
    }
}

/// Constructs a `DomainError` directly from a `michi::toon::ToonError` --
/// deliberately not via `ToDomainError`, which is scoped only to
/// `MonoklError`'s closed `monokl_code()` set. Indicates a bug in this
/// module's own row/field construction, not a caller input error.
pub fn toon_validation_error(e: &michi::toon::ToonError) -> michi::DomainError {
    michi::DomainError::new(michi::ErrorCode::ExternalFailure, format!("toon_render_invalid: {e}")).retryable(false)
}

/// The sole function that converts a `ToonProjection<N>` implementation into
/// a rendered TOON string.
#[allow(clippy::result_large_err)] // AC-004B locks this exact signature verbatim against michi's own DomainError.
pub fn render_projection<const N: usize, T: ToonProjection<N>>(value: &T) -> Result<String, michi::DomainError> {
    let rows: Vec<Vec<Value>> = value.toon_rows().into_iter().map(|row| row.to_vec()).collect();
    let fields: Vec<String> = T::FIELDS.iter().map(|s| (*s).to_string()).collect();

    let opts = michi::toon::ToonOptions::new(T::TYPE_NAME, fields, rows.clone()).total_count(value.toon_total_count());
    if let Err(e) = opts.validate() {
        return Err(toon_validation_error(&e));
    }

    let mut resp = michi::AgentResponse::new(T::TYPE_NAME).items(rows, T::FIELDS.as_slice()).hints(value.toon_hints());
    if let Some(n) = value.toon_total_count() {
        resp = resp.total_count(n);
    }
    Ok(resp.render_toon())
}

#[cfg(test)]
mod projection_tests {
    use super::*;

    #[test]
    fn ac004b_render_projection_symbols_result_end_to_end() {
        use crate::types::{SymbolEntry, SymbolKind, SymbolsResult};
        use camino::Utf8PathBuf;
        use std::collections::BTreeMap;

        let mut files = BTreeMap::new();
        files.insert(
            Utf8PathBuf::from("src/lib.rs"),
            vec![SymbolEntry {
                name: "foo".to_string(),
                kind: SymbolKind::Function,
                line: 10,
                signature: Some("fn foo()".to_string()),
                owner: None,
                trait_impl: None,
                visibility: Some(crate::types::Visibility::Public),
                kind_detail: None,
            }],
        );
        let result =
            SymbolsResult { files, total_symbol_count: 1, truncation_marker: None, diagnostics: Vec::new() };

        let rendered = render_projection(&result).expect("valid projection must render");
        assert!(rendered.starts_with("symbols[1]{file,name,kind,line,signature,owner,traitImpl,visibility,kindDetail}:\n"), "got: {rendered}");
        assert!(rendered.contains("src/lib.rs,foo,function,10,fn foo(),,,public,"), "got: {rendered}");
    }

    #[test]
    fn ac004b_render_projection_search_response_includes_extract_hint() {
        use crate::types::{CodeBlock, RankedBlock, SearchResponse, SymbolKind};
        use camino::Utf8PathBuf;

        let response = SearchResponse {
            results: vec![RankedBlock {
                block: CodeBlock {
                    file: Utf8PathBuf::from("src/lib.rs"),
                    line_start: 1,
                    line_end: 5,
                    node_kind: SymbolKind::Function,
                    code: "fn foo() {}".to_string(),
                    symbol_signature: Some("fn foo()".to_string()),
                    matched_lines: vec![1],
                    matched_keywords: vec!["foo".to_string()],
                },
                bm25_score: 1.0,
                coverage_boost: 0.0,
                node_type_boost: 0.0,
                final_score: 1.0,
                rank: 1,
                parent_context: None,
            }],
            total_blocks_before_truncation: 1,
            truncated: false,
            truncation_marker: None,
            total_bytes: 100,
            total_tokens: 20,
            diagnostics: Vec::new(),
        };

        let rendered = render_projection(&response).expect("valid projection must render");
        assert!(rendered.contains("search[1]{file,lineStart,lineEnd,nodeKind,symbolSignature,rank,finalScore}:\n"), "got: {rendered}");
        assert!(rendered.contains("src/lib.rs,1,5,function,fn foo(),1,1"), "got: {rendered}");
        assert!(!rendered.contains("fn foo() {}"), "code field must never appear in TOON output, got: {rendered}");
        assert!(rendered.contains("help[1]:\n  use extract"), "extract hint must be appended, got: {rendered}");
    }

    /// AC-014: an invalid ToonOptions (deliberately mismatched arity, forced
    /// via a hand-built ToonOptions rather than a real ToonProjection impl,
    /// since ToonProjection<N> makes a real mismatch impossible to construct)
    /// is caught by render_projection's own explicit validate() call and
    /// converted via toon_validation_error, not silently rendered.
    #[test]
    fn ac014_toon_validation_error_shape() {
        let opts = michi::toon::ToonOptions::new(
            "t".to_string(),
            vec!["a".to_string()],
            vec![vec![Value::from("x"), Value::from("y")]],
        );
        let err = opts.validate().unwrap_err();
        let domain_err = toon_validation_error(&err);
        assert_eq!(domain_err.code, michi::ErrorCode::ExternalFailure);
        assert!(!domain_err.retryable);
        assert!(domain_err.message.starts_with("toon_render_invalid: "), "got: {}", domain_err.message);
    }

    /// AC-013: michi::toon::Value::Null and Value::from("") render
    /// identically in a full TOON document -- there is no way for a reader
    /// to distinguish an absent field from a present empty-string field.
    #[test]
    fn ac013_null_and_empty_string_render_identically() {
        let opts_null = michi::toon::ToonOptions::new(
            "t".to_string(),
            vec!["a".to_string()],
            vec![vec![Value::Null]],
        );
        let opts_empty = michi::toon::ToonOptions::new(
            "t".to_string(),
            vec!["a".to_string()],
            vec![vec![Value::from("")]],
        );
        assert_eq!(michi::toon::render_toon(&opts_null), michi::toon::render_toon(&opts_empty));
    }
}
