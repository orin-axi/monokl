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
}
