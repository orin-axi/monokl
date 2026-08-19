pub mod lexer;
pub mod parser;
pub mod plan;

pub use parser::{Modifier, ParsedTerm, parse};
pub use plan::QueryPlan;

#[cfg(test)]
mod tests {
    // AC-021: mod.rs's exact re-export surface -- Modifier, ParsedTerm, parse,
    // and QueryPlan are reachable directly at query::, without going through
    // query::parser:: / query::plan::. Lexer/Token are deliberately NOT
    // re-exported here -- only reachable via query::lexer::.
    #[test]
    fn reexported_items_reachable_at_query_root() {
        use crate::query::{Modifier, ParsedTerm, QueryPlan, parse};
        let terms: Vec<ParsedTerm> = parse("+foo").unwrap();
        assert_eq!(terms.len(), 1);
        assert_eq!(terms[0].modifier, Modifier::Required);
        let qp = QueryPlan::from_terms(terms);
        assert_eq!(qp.required, vec![0]);
    }

    #[test]
    fn lexer_and_token_only_reachable_via_query_lexer_path() {
        use crate::query::lexer::{Lexer, Token};
        let mut l = Lexer::new("foo");
        assert_eq!(l.next_token(), Token::Word("foo".to_string()));
    }

    // AC-021: mod.rs declares exactly 3 `pub mod` lines (lexer, parser, plan,
    // in that order), then exactly 2 `pub use` lines (parser's grouped
    // re-export first, then plan's), with Lexer/Token never re-exported at
    // the query:: level. Pinned via structural source inspection rather than
    // reachability alone -- reachability tests (above) can't detect an EXTRA
    // re-export line (e.g. `pub use lexer::{Lexer, Token};` added alongside
    // the existing ones) since that would still leave the already-tested
    // paths reachable.
    //
    // Whole-head exact comparison, not a keyword filter: an approach that
    // collects lines by checking for a `mod `/`use ` keyword (even after
    // stripping a leading visibility modifier) is blind to declarations
    // that use neither keyword literally, e.g. `pub type Token =
    // lexer::Token;` -- a fully public `query::Token` path that contains
    // neither substring. A keyword filter's visibility-modifier stripping
    // is also easy to get subtly wrong -- e.g. missing the legal
    // `pub (crate) use ...` form (a space before `(crate)`) if the
    // stripping only recognizes `pub(crate)` with no space. Comparing the
    // ENTIRE pre-test-module portion of the file, line for line, against
    // the exact expected declaration set sidesteps both failure modes: any
    // extra, reordered, or visibility-narrowed line -- regardless of
    // whether it happens to contain `mod `/`use ` or how its visibility
    // modifier is spelled -- fails the `assert_eq!` below, since it cannot
    // possibly appear in a 5-line exact-match vec that doesn't contain it.
    #[test]
    fn mod_rs_exact_reexport_surface_structural() {
        let src = include_str!("mod.rs");
        // Scoped to the pre-test-module portion of the file only -- the test
        // module itself contains its own `use crate::query::{...}` /
        // `mod tests {` lines, which would otherwise corrupt the exact-line
        // comparison below.
        let head = &src[..src.find("#[cfg(test)]").expect("no test module marker found in mod.rs")];

        let head_lines: Vec<&str> = head
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with("//"))
            .collect();
        assert_eq!(
            head_lines,
            vec![
                "pub mod lexer;",
                "pub mod parser;",
                "pub mod plan;",
                "pub use parser::{Modifier, ParsedTerm, parse};",
                "pub use plan::QueryPlan;",
            ],
            "mod.rs's pre-test portion must contain exactly these 5 declarations, in this exact order, and nothing else (AC-021)"
        );
    }

    // AC-021: the test above (`mod_rs_exact_reexport_surface_structural`)
    // only scans the portion of the file BEFORE the `#[cfg(test)]` marker --
    // it has no visibility into anything appended after `mod tests { ... }`'s
    // closing brace. A top-level declaration placed there (e.g. `pub use
    // lexer::Token;` at end-of-file) is ordinary, live, public Rust that
    // creates a `query::Token` path in violation of AC-021, yet is invisible
    // to that test. This test closes that gap by scanning the ENTIRE file
    // for top-level (non-indented, non-comment) lines, rather than only the
    // pre-test-module head, and asserting the complete set is exactly the
    // same 5 declarations plus the `#[cfg(test)]` / `mod tests {` / closing
    // `}` markers that bound this very test module.
    //
    // Indentation-based filtering, not brace-counting: a brace-matching
    // approach that counts `{`/`}` characters to find and excise the `mod
    // tests { ... }` block does NOT work here, because `include_str!` pulls
    // in this very file, including this matcher's own `'{' => ...` /
    // `'}' => ...` source lines -- feeding the matcher's own source back
    // into itself corrupts the depth count (the same self-reference trap
    // documented in lexer.rs's `lexer_struct_has_no_derive_and_private_fields_structural`
    // test). Filtering on leading-whitespace instead sidesteps this: every
    // line inside `mod tests { ... }` is indented except its own closing
    // `}`, so a plain per-line check needs no brace characters in its own
    // source at all.
    #[test]
    fn mod_rs_has_no_declarations_outside_the_scanned_head() {
        let src = include_str!("mod.rs");
        let top_level: Vec<&str> = src
            .lines()
            .filter(|l| !l.is_empty() && !l.starts_with(char::is_whitespace))
            .filter(|l| !l.starts_with("//"))
            .filter(|l| !matches!(*l, "#[cfg(test)]" | "mod tests {" | "}"))
            .collect();
        assert_eq!(
            top_level,
            vec![
                "pub mod lexer;",
                "pub mod parser;",
                "pub mod plan;",
                "pub use parser::{Modifier, ParsedTerm, parse};",
                "pub use plan::QueryPlan;",
            ],
            "mod.rs must contain no top-level declaration anywhere in the file, including after the test module, beyond these 5 (AC-021)"
        );
    }
}
