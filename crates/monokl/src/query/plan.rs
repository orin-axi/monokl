use super::parser::{Modifier, ParsedTerm};

#[derive(Debug, Clone)]
pub struct QueryPlan {
    pub terms: Vec<ParsedTerm>,
    pub required: Vec<usize>,
    pub excluded: Vec<usize>,
    pub scored: Vec<usize>,
}

impl QueryPlan {
    pub fn from_terms(terms: Vec<ParsedTerm>) -> Self {
        let required: Vec<usize> = terms.iter().enumerate()
            .filter(|(_, t)| t.modifier == Modifier::Required)
            .map(|(i, _)| i).collect();
        let excluded: Vec<usize> = terms.iter().enumerate()
            .filter(|(_, t)| t.modifier == Modifier::Excluded)
            .map(|(i, _)| i).collect();
        let scored: Vec<usize> = terms.iter().enumerate()
            .filter(|(_, t)| t.modifier == Modifier::Optional)
            .map(|(i, _)| i).collect();
        Self { terms, required, excluded, scored }
    }

    pub fn is_empty(&self) -> bool { self.terms.is_empty() }

    pub fn search_patterns(&self) -> Vec<&str> {
        self.required.iter().chain(self.scored.iter())
            .map(|&i| self.terms[i].pattern.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // AC-016: QueryPlan derives (Debug, Clone only) + all 4 fields pub.
    fn assert_query_plan_derives<T: std::fmt::Debug + Clone>() {}
    #[test]
    fn query_plan_shape_and_pub_fields() {
        assert_query_plan_derives::<QueryPlan>();
        // Direct struct-literal construction from outside from_terms, relying on
        // all 4 fields being pub (AC-016), which AC-019/AC-022 both depend on.
        let qp = QueryPlan { terms: vec![], required: vec![], excluded: vec![], scored: vec![] };
        assert!(qp.terms.is_empty());
        assert!(qp.required.is_empty());
        assert!(qp.excluded.is_empty());
        assert!(qp.scored.is_empty());
    }

    #[test]
    fn from_terms_on_empty_vec_produces_all_empty_fields() {
        let qp = QueryPlan::from_terms(Vec::new());
        assert!(qp.terms.is_empty());
        assert!(qp.required.is_empty());
        assert!(qp.excluded.is_empty());
        assert!(qp.scored.is_empty());
    }

    // AC-017: from_terms partitions by modifier, preserving relative order;
    // every index appears in exactly one category.
    fn term(modifier: Modifier, pattern: &str) -> ParsedTerm {
        ParsedTerm { modifier, pattern: pattern.to_string(), is_regex: false }
    }

    #[test]
    fn from_terms_partitions_preserving_relative_order() {
        let terms = vec![
            term(Modifier::Optional, "s0"),
            term(Modifier::Required, "r0"),
            term(Modifier::Excluded, "e0"),
            term(Modifier::Required, "r1"),
            term(Modifier::Optional, "s1"),
            term(Modifier::Excluded, "e1"),
        ];
        let qp = QueryPlan::from_terms(terms);
        assert_eq!(qp.required, vec![1, 3]);
        assert_eq!(qp.excluded, vec![2, 5]);
        assert_eq!(qp.scored, vec![0, 4]);
        // Every index appears in exactly one category.
        let mut all: Vec<usize> = qp.required.iter().chain(qp.excluded.iter()).chain(qp.scored.iter()).copied().collect();
        all.sort_unstable();
        assert_eq!(all, vec![0, 1, 2, 3, 4, 5]);
    }

    // AC-018: is_empty() checks total term count, not required/scored specifically.
    #[test]
    fn is_empty_true_only_when_terms_is_fully_empty() {
        let qp = QueryPlan::from_terms(Vec::new());
        assert!(qp.is_empty());

        // Exclusion-only query: zero Required/Optional terms, but is_empty()
        // still reports false because it checks total term count.
        let excl_only = QueryPlan::from_terms(vec![term(Modifier::Excluded, "e0"), term(Modifier::Excluded, "e1")]);
        assert!(!excl_only.is_empty());
        assert!(excl_only.search_patterns().is_empty());
    }

    // AC-019: search_patterns() returns required first, then scored, in each
    // category's own preserved order; excluded never appears; ordering is by
    // category, not by original input position.
    #[test]
    fn search_patterns_required_then_scored_excludes_excluded() {
        let terms = vec![
            term(Modifier::Optional, "s0"),   // idx 0
            term(Modifier::Required, "r0"),   // idx 1
            term(Modifier::Excluded, "e0"),   // idx 2
        ];
        let qp = QueryPlan::from_terms(terms);
        // required = [1], scored = [0] -- required's pattern first despite
        // appearing second in the original input.
        assert_eq!(qp.search_patterns(), vec!["r0", "s0"]);
    }

    #[test]
    fn search_patterns_preserves_each_categorys_own_relative_order() {
        let terms = vec![
            term(Modifier::Required, "r0"),
            term(Modifier::Optional, "s0"),
            term(Modifier::Required, "r1"),
            term(Modifier::Optional, "s1"),
        ];
        let qp = QueryPlan::from_terms(terms);
        assert_eq!(qp.search_patterns(), vec!["r0", "r1", "s0", "s1"]);
    }
}
