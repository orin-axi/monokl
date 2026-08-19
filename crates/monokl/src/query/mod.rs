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
    // Visibility-narrowing-aware: a plain `starts_with("pub mod ")` /
    // `starts_with("pub use ")` filter is blind to a `pub(crate) mod x;` or
    // `pub(crate) use lexer::Token;` line -- such a line doesn't start with
    // the literal `"pub mod "` / `"pub use "` (the `(crate)` sits in
    // between), so it would silently slip past both the collection filter
    // and the exact-set assertion below, exposing `query::Token` in-crate
    // undetected -- the exact `pub(crate)`-evasion class lexer.rs's own
    // `lexer_helper_methods_are_private_structural` /
    // `lexer_exactly_two_public_methods_structural` structural checks
    // already close for Lexer's method visibility. `decl(kw)` below strips
    // any leading visibility modifier (`pub`, `pub(crate)`, `pub(super)`,
    // `pub(in ...)`) before checking the `kw` prefix, so a narrowed
    // declaration IS collected -- but the ORIGINAL (unstripped) trimmed line
    // is what's pushed, so it still fails the `assert_eq!` below against the
    // all-bare-`pub `-prefixed expected list, catching the narrowing as a
    // test failure rather than missing it. A bare (fully private) `mod x;`
    // or `use y;` line is caught the same way: `strip_vis` is a no-op on it,
    // so it's collected as `"mod x;"` / `"use y;"` verbatim, which likewise
    // fails the exact-set assertion.
    #[test]
    fn mod_rs_exact_reexport_surface_structural() {
        let src = include_str!("mod.rs");
        // Scoped to the pre-test-module portion of the file only -- the test
        // module itself contains its own `use crate::query::{...}` /
        // `mod tests {` lines, which would otherwise be picked up by the
        // same `mod `/`use ` keyword filter and corrupt the collected sets.
        let head = &src[..src.find("#[cfg(test)]").expect("no test module marker found in mod.rs")];

        fn strip_vis(line: &str) -> &str {
            let l = line.trim_start();
            match l.strip_prefix("pub") {
                Some(rest) => {
                    let rest = if rest.starts_with('(') {
                        match rest.find(')') {
                            Some(i) => &rest[i + 1..],
                            None => rest,
                        }
                    } else {
                        rest
                    };
                    rest.trim_start()
                }
                None => l,
            }
        }
        let decl = |kw: &str| -> Vec<&str> {
            head.lines().map(str::trim).filter(|l| strip_vis(l).starts_with(kw)).collect()
        };

        let mod_lines = decl("mod ");
        assert_eq!(
            mod_lines,
            vec!["pub mod lexer;", "pub mod parser;", "pub mod plan;"],
            "expected exactly 3 `pub mod` lines in this exact order, none visibility-narrowed or private"
        );

        let use_lines = decl("use ");
        assert_eq!(
            use_lines,
            vec!["pub use parser::{Modifier, ParsedTerm, parse};", "pub use plan::QueryPlan;"],
            "expected exactly 2 `pub use` lines in this exact order, naming exactly these items, none visibility-narrowed or private"
        );

        let lexer_decl_pos = src.find("pub mod lexer;").unwrap();
        let parser_decl_pos = src.find("pub mod parser;").unwrap();
        let plan_decl_pos = src.find("pub mod plan;").unwrap();
        let use_parser_pos = src.find("pub use parser::{Modifier, ParsedTerm, parse};").unwrap();
        let use_plan_pos = src.find("pub use plan::QueryPlan;").unwrap();
        assert!(lexer_decl_pos < parser_decl_pos);
        assert!(parser_decl_pos < plan_decl_pos);
        assert!(plan_decl_pos < use_parser_pos);
        assert!(use_parser_pos < use_plan_pos);

        // No `pub use` line names Lexer or Token -- they're only reachable
        // via `query::lexer::`, never re-exported at `query::` itself.
        for line in &use_lines {
            assert!(!line.contains("Lexer"), "unexpected Lexer re-export: {line}");
            assert!(!line.contains("Token"), "unexpected Token re-export: {line}");
        }
    }
}
