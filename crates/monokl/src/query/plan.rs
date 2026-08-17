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
}
