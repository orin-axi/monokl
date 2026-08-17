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
    #[test]
    fn mod_rs_exact_reexport_surface_structural() {
        let src = include_str!("mod.rs");

        let mod_lines: Vec<&str> =
            src.lines().map(str::trim_start).filter(|l| l.starts_with("pub mod ")).collect();
        assert_eq!(
            mod_lines,
            vec!["pub mod lexer;", "pub mod parser;", "pub mod plan;"],
            "expected exactly 3 `pub mod` lines in this exact order"
        );

        let use_lines: Vec<&str> =
            src.lines().map(str::trim_start).filter(|l| l.starts_with("pub use ")).collect();
        assert_eq!(
            use_lines,
            vec!["pub use parser::{Modifier, ParsedTerm, parse};", "pub use plan::QueryPlan;"],
            "expected exactly 2 `pub use` lines in this exact order, naming exactly these items"
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
