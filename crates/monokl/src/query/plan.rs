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

    // AC-016: QueryPlan derives (Debug, Clone only) -- no serde derive.
    // A plain generic bound check (assert_query_plan_derives below) can only
    // prove Debug+Clone ARE implemented, never that Serialize/Deserialize are
    // absent -- same const-specialization technique as lexer.rs's Token
    // absence probes and parser.rs's ParsedTerm absence probes, applied here
    // to serde::Serialize / serde::de::DeserializeOwned specifically
    // (mirroring tests_types.rs's SerProbe/DeProbe pattern from SPEC-006/007/008).
    struct SerProbe<T>(std::marker::PhantomData<T>);
    trait NotSer { const IS: bool = false; }
    impl<T> NotSer for SerProbe<T> {}
    impl<T: serde::Serialize> SerProbe<T> { const IS: bool = true; }

    struct DeProbe<T>(std::marker::PhantomData<T>);
    trait NotDe { const IS: bool = false; }
    impl<T> NotDe for DeProbe<T> {}
    impl<'de, T: serde::de::DeserializeOwned> DeProbe<T> { const IS: bool = true; }

    #[test]
    fn query_plan_absence_of_serde_derive_pinned() {
        assert!(!SerProbe::<QueryPlan>::IS);
        assert!(!DeProbe::<QueryPlan>::IS);
        // Positive control (proves the probes themselves aren't just always
        // false): usize is Serialize + DeserializeOwned.
        assert!(SerProbe::<usize>::IS);
        assert!(DeProbe::<usize>::IS);
    }

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

    // AC-016: QueryPlan's 4 fields are declared pub (not pub(crate)), pinned
    // via structural source inspection -- mirrors lexer.rs's private-field
    // check but in the public direction: asserts each field's declaration
    // line starts with a bare `pub {field}:`, not `pub(crate) {field}:`
    // (which a plain `contains("pub")` substring check would also satisfy).
    #[test]
    fn query_plan_fields_are_declared_pub_structural() {
        let src = include_str!("plan.rs");
        let decl_start = src.find("pub struct QueryPlan").unwrap_or_else(|| panic!("`pub struct QueryPlan` not found in plan.rs"));
        let open = src[decl_start..].find('{').unwrap() + decl_start;
        let close = src[open..].find('}').unwrap() + open;
        let field_block = &src[open..=close];
        for field in ["terms", "required", "excluded", "scored"] {
            let needle = format!("{field}:");
            let idx = field_block.find(&needle)
                .unwrap_or_else(|| panic!("field `{field}` not found in QueryPlan struct body"));
            let line_start = field_block[..idx].rfind('\n').map(|p| p + 1).unwrap_or(0);
            let line_end = field_block[idx..].find('\n').map(|p| idx + p).unwrap_or(field_block.len());
            let line = field_block[line_start..line_end].trim_start();
            assert!(
                line.starts_with(&format!("pub {field}:")),
                "QueryPlan.{field} must be declared exactly `pub {field}:` (AC-016), found line {line:?}"
            );
        }
    }

    // AC-016: QueryPlan's 4 fields' declaration order (terms, required,
    // excluded, scored), pinned via structural source inspection -- mirrors
    // mod.rs's / lexer.rs's Token order-pinning technique.
    #[test]
    fn query_plan_field_declaration_order_structural() {
        let src = include_str!("plan.rs");
        let decl_start = src.find("pub struct QueryPlan").unwrap_or_else(|| panic!("`pub struct QueryPlan` not found in plan.rs"));
        let open = src[decl_start..].find('{').unwrap() + decl_start;
        let close = src[open..].find('}').unwrap() + open;
        let block = &src[open..=close];

        let fields = ["terms", "required", "excluded", "scored"];
        let mut last_pos: Option<usize> = None;
        for f in fields {
            let pos = block.find(f).unwrap_or_else(|| panic!("field `{f}` not found in QueryPlan struct body"));
            if let Some(last) = last_pos {
                assert!(pos > last, "field `{f}` found out of the expected declaration order");
            }
            last_pos = Some(pos);
        }
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

    // AC-022: directly-constructed QueryPlan with an out-of-bounds index
    // panics in search_patterns() rather than erroring or returning empty.
    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn search_patterns_panics_on_out_of_bounds_index_in_directly_constructed_plan() {
        let qp = QueryPlan { terms: vec![], required: vec![5], excluded: vec![], scored: vec![] };
        let _ = qp.search_patterns();
    }

    // AC-019: search_patterns() only filters by category membership (required
    // then scored, in that order) -- it never cross-checks an index against
    // `excluded`. A directly-constructed QueryPlan (legal since all 4 fields
    // are pub, AC-016) can place an excluded-looking term's index in
    // `required` or `scored` directly, and its pattern still surfaces. Index
    // 0's own term has Modifier::Excluded and index 0 also appears in
    // `excluded`, but because it's ALSO listed in `required`, its pattern
    // "e0" still appears in search_patterns()'s output -- proving the method
    // does no cross-check against the excluded vec at all.
    #[test]
    fn search_patterns_directly_constructed_plan_ignores_excluded_membership() {
        let qp = QueryPlan {
            terms: vec![term(Modifier::Excluded, "e0"), term(Modifier::Required, "r0")],
            required: vec![0, 1],
            excluded: vec![0],
            scored: vec![],
        };
        assert_eq!(qp.search_patterns(), vec!["e0", "r0"]);
    }

    // AC-016: QueryPlan derives (Debug, Clone only) -- absence of Default is
    // not provable by a plain generic bound check (which can only prove a
    // trait IS implemented), so this uses the same const-specialization
    // probe technique as this file's own SerProbe/DeProbe above. Per the
    // gate's own analysis, PartialEq/Hash/Copy are compile-impossible
    // additions to QueryPlan (Vec<ParsedTerm> has no Eq since ParsedTerm
    // lacks PartialEq), so Default is the only realistic derive-list gap
    // worth probing here.
    struct DefaultProbe<T>(std::marker::PhantomData<T>);
    trait NotDefault { const IS: bool = false; }
    impl<T> NotDefault for DefaultProbe<T> {}
    impl<T: Default> DefaultProbe<T> { const IS: bool = true; }

    #[test]
    fn query_plan_absence_of_default_pinned() {
        assert!(!DefaultProbe::<QueryPlan>::IS);
        // Positive control (proves the probe itself isn't just always
        // false): usize implements Default.
        assert!(DefaultProbe::<usize>::IS);
    }
}
