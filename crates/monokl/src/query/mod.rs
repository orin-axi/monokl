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
}
