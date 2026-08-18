use super::lexer::{Lexer, Token};
use crate::error::{MonoklError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Modifier {
    Required,
    Excluded,
    Optional,
}

#[derive(Debug, Clone)]
pub struct ParsedTerm {
    pub modifier: Modifier,
    pub pattern: String,
    pub is_regex: bool,
}

pub fn parse(input: &str) -> Result<Vec<ParsedTerm>> {
    const LIMIT: usize = 64;
    if input.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut lexer = Lexer::new(input);
    let mut terms: Vec<ParsedTerm> = Vec::new();
    loop {
        let tok = lexer.next_token();
        match tok {
            Token::Eof => break,
            Token::Plus => {
                let content = lexer.next_token();
                if let Some(term) = content_to_term(content, Modifier::Required) {
                    terms.push(term);
                }
            }
            Token::Minus => {
                let content = lexer.next_token();
                if let Some(term) = content_to_term(content, Modifier::Excluded) {
                    terms.push(term);
                }
            }
            Token::RegexPrefix => {
                let pattern_tok = lexer.next_token();
                match pattern_tok {
                    Token::Word(raw) | Token::QuotedString(raw) => {
                        terms.push(ParsedTerm {
                            modifier: Modifier::Optional,
                            pattern: raw,
                            is_regex: true,
                        });
                    }
                    _ => {}
                }
            }
            Token::Word(w) => {
                terms.push(ParsedTerm {
                    modifier: Modifier::Optional,
                    pattern: regex::escape(&w),
                    is_regex: false,
                });
            }
            Token::QuotedString(s) => {
                terms.push(ParsedTerm {
                    modifier: Modifier::Optional,
                    pattern: regex::escape(&s),
                    is_regex: false,
                });
            }
        }
    }
    if terms.len() > LIMIT {
        return Err(MonoklError::TooManyTerms {
            count: terms.len(),
            limit: LIMIT,
        });
    }
    Ok(terms)
}

fn content_to_term(tok: Token, modifier: Modifier) -> Option<ParsedTerm> {
    match tok {
        Token::Word(w) => Some(ParsedTerm {
            modifier,
            pattern: regex::escape(&w),
            is_regex: false,
        }),
        Token::QuotedString(s) => Some(ParsedTerm {
            modifier,
            pattern: regex::escape(&s),
            is_regex: false,
        }),
        Token::RegexPrefix => None, // `+regex:foo` documented as silently losing modifier
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // AC-008: Modifier derives + exact 3-variant shape.
    fn assert_modifier_derives<T: std::fmt::Debug + Clone + PartialEq + Eq>() {}
    // AC-008: ParsedTerm derives (Debug, Clone only) + fields are pub (checked by direct access).
    fn assert_parsed_term_derives<T: std::fmt::Debug + Clone>() {}

    // AC-008: absence of PartialEq/Eq on ParsedTerm. Same const-specialization
    // technique as lexer.rs's Token absence probes (this project's established
    // fix for the "a bound check can only prove presence" gap, per
    // tests_types.rs's SPEC-006/007/008 PartialEqProbe/EqProbe pattern).
    struct PartialEqProbe<T>(std::marker::PhantomData<T>);
    trait NotPartialEq { const IS: bool = false; }
    impl<T> NotPartialEq for PartialEqProbe<T> {}
    impl<T: PartialEq> PartialEqProbe<T> { const IS: bool = true; }

    struct EqProbe<T>(std::marker::PhantomData<T>);
    trait NotEq { const IS: bool = false; }
    impl<T> NotEq for EqProbe<T> {}
    impl<T: Eq> EqProbe<T> { const IS: bool = true; }

    #[test]
    fn parsed_term_absence_of_partialeq_and_eq_pinned() {
        assert!(!PartialEqProbe::<ParsedTerm>::IS);
        assert!(!EqProbe::<ParsedTerm>::IS);
        // Positive control (proves the probes themselves aren't just always
        // false): Modifier, declared 2 lines above ParsedTerm in this same
        // file, DOES derive PartialEq + Eq.
        assert!(PartialEqProbe::<Modifier>::IS);
        assert!(EqProbe::<Modifier>::IS);
    }

    // AC-008: absence of Copy/Hash/PartialOrd/Ord on Modifier -- Modifier
    // already derives PartialEq + Eq (unlike Token, which derives neither),
    // so this probes the 4 traits an additive-derive mutation could still
    // append to Modifier's derive list undetected: Copy, Hash, PartialOrd,
    // Ord. Same const-specialization technique as lexer.rs's Token
    // absence-of-Eq/Hash/PartialOrd/Ord probes (this project's established
    // fix for the "a bound check can only prove presence" gap).
    struct ModifierCopyProbe<T>(std::marker::PhantomData<T>);
    trait NotModifierCopy { const IS: bool = false; }
    impl<T> NotModifierCopy for ModifierCopyProbe<T> {}
    impl<T: Copy> ModifierCopyProbe<T> { const IS: bool = true; }

    struct ModifierHashProbe<T>(std::marker::PhantomData<T>);
    trait NotModifierHash { const IS: bool = false; }
    impl<T> NotModifierHash for ModifierHashProbe<T> {}
    impl<T: std::hash::Hash> ModifierHashProbe<T> { const IS: bool = true; }

    struct ModifierPartialOrdProbe<T>(std::marker::PhantomData<T>);
    trait NotModifierPartialOrd { const IS: bool = false; }
    impl<T> NotModifierPartialOrd for ModifierPartialOrdProbe<T> {}
    impl<T: PartialOrd> ModifierPartialOrdProbe<T> { const IS: bool = true; }

    struct ModifierOrdProbe<T>(std::marker::PhantomData<T>);
    trait NotModifierOrd { const IS: bool = false; }
    impl<T> NotModifierOrd for ModifierOrdProbe<T> {}
    impl<T: Ord> ModifierOrdProbe<T> { const IS: bool = true; }

    #[test]
    fn modifier_absence_of_copy_hash_partialord_ord_pinned() {
        assert!(!ModifierCopyProbe::<Modifier>::IS);
        assert!(!ModifierHashProbe::<Modifier>::IS);
        assert!(!ModifierPartialOrdProbe::<Modifier>::IS);
        assert!(!ModifierOrdProbe::<Modifier>::IS);
        // Positive controls (prove each probe itself isn't just always false).
        assert!(ModifierCopyProbe::<usize>::IS);
        assert!(ModifierHashProbe::<usize>::IS);
        assert!(ModifierPartialOrdProbe::<usize>::IS);
        assert!(ModifierOrdProbe::<usize>::IS);
    }

    // AC-008: absence of serde Serialize/Deserialize on Modifier and
    // ParsedTerm -- same const-specialization technique as lexer.rs's Token
    // probe and plan.rs's QueryPlan probe (SPEC-006 AC-015 / SPEC-008 AC-002
    // precedent).
    struct SerProbe<T>(std::marker::PhantomData<T>);
    trait NotSer { const IS: bool = false; }
    impl<T> NotSer for SerProbe<T> {}
    impl<T: serde::Serialize> SerProbe<T> { const IS: bool = true; }

    struct DeProbe<T>(std::marker::PhantomData<T>);
    trait NotDe { const IS: bool = false; }
    impl<T> NotDe for DeProbe<T> {}
    impl<'de, T: serde::de::DeserializeOwned> DeProbe<T> { const IS: bool = true; }

    #[test]
    fn modifier_and_parsed_term_absence_of_serde_derive_pinned() {
        assert!(!SerProbe::<Modifier>::IS);
        assert!(!DeProbe::<Modifier>::IS);
        assert!(!SerProbe::<ParsedTerm>::IS);
        assert!(!DeProbe::<ParsedTerm>::IS);
        // Positive control (proves the probes themselves aren't just always
        // false): usize is Serialize + DeserializeOwned.
        assert!(SerProbe::<usize>::IS);
        assert!(DeProbe::<usize>::IS);
    }

    // AC-008: ParsedTerm's 3 fields are declared pub (not pub(crate)),
    // pinned via structural source inspection -- mirrors lexer.rs's private-
    // field check (lexer_struct_has_no_derive_and_private_fields_structural)
    // but in the public direction: asserts each field's declaration line
    // starts with a bare `pub {field}:`, not `pub(crate) {field}:` (which a
    // plain `contains("pub")` substring check would also satisfy).
    #[test]
    fn parsed_term_fields_are_declared_pub_structural() {
        let src = include_str!("parser.rs");
        let decl_start = src.find("pub struct ParsedTerm").unwrap_or_else(|| panic!("`pub struct ParsedTerm` not found in parser.rs"));
        let open = src[decl_start..].find('{').unwrap() + decl_start;
        let close = src[open..].find('}').unwrap() + open;
        let field_block = &src[open..=close];
        for field in ["modifier", "pattern", "is_regex"] {
            let needle = format!("{field}:");
            let idx = field_block.find(&needle)
                .unwrap_or_else(|| panic!("field `{field}` not found in ParsedTerm struct body"));
            let line_start = field_block[..idx].rfind('\n').map(|p| p + 1).unwrap_or(0);
            let line_end = field_block[idx..].find('\n').map(|p| idx + p).unwrap_or(field_block.len());
            let line = field_block[line_start..line_end].trim_start();
            assert!(
                line.starts_with(&format!("pub {field}:")),
                "ParsedTerm.{field} must be declared exactly `pub {field}:` (AC-008), found line {line:?}"
            );
        }
    }

    // AC-008: ParsedTerm's 3 fields' declaration order (modifier, pattern,
    // is_regex), pinned via structural source inspection -- ParsedTerm has
    // no derive that makes field order wire-observable, so this mirrors
    // mod.rs's / lexer.rs's Token order-pinning technique.
    #[test]
    fn parsed_term_field_declaration_order_structural() {
        let src = include_str!("parser.rs");
        let decl_start = src.find("pub struct ParsedTerm").unwrap_or_else(|| panic!("`pub struct ParsedTerm` not found in parser.rs"));
        let open = src[decl_start..].find('{').unwrap() + decl_start;
        let close = src[open..].find('}').unwrap() + open;
        let block = &src[open..=close];

        let fields = ["modifier", "pattern", "is_regex"];
        let mut last_pos: Option<usize> = None;
        for f in fields {
            let pos = block.find(f).unwrap_or_else(|| panic!("field `{f}` not found in ParsedTerm struct body"));
            if let Some(last) = last_pos {
                assert!(pos > last, "field `{f}` found out of the expected declaration order");
            }
            last_pos = Some(pos);
        }
    }

    // AC-008: Modifier's exact 3-variant set, with no room for a 4th. Unlike
    // Token (whose variant count is pinned "for free" by parse()'s own
    // exhaustive match), Modifier is only ever compared with `==` in this
    // codebase, so it needs its own exhaustive-match-with-no-wildcard-arm to
    // make an added variant a compile error, plus discriminant assertions
    // to pin declaration order (Modifier is fieldless, so its order IS
    // wire/discriminant-observable and testable, unlike Token).
    #[test]
    fn modifier_exhaustive_match_admits_exactly_three_variants() {
        fn describe(m: &Modifier) -> &'static str {
            match m {
                Modifier::Required => "required",
                Modifier::Excluded => "excluded",
                Modifier::Optional => "optional",
                // No wildcard arm: a 4th variant added to Modifier fails to
                // compile here.
            }
        }
        assert_eq!(describe(&Modifier::Required), "required");
        assert_eq!(describe(&Modifier::Excluded), "excluded");
        assert_eq!(describe(&Modifier::Optional), "optional");
    }

    #[test]
    fn modifier_discriminants_pinned_in_declaration_order() {
        assert_eq!(Modifier::Required as isize, 0);
        assert_eq!(Modifier::Excluded as isize, 1);
        assert_eq!(Modifier::Optional as isize, 2);
    }

    #[test]
    fn modifier_and_parsed_term_shapes() {
        assert_modifier_derives::<Modifier>();
        assert_parsed_term_derives::<ParsedTerm>();
        assert_eq!(Modifier::Required, Modifier::Required);
        assert_ne!(Modifier::Required, Modifier::Excluded);
        let t = ParsedTerm { modifier: Modifier::Optional, pattern: "x".to_string(), is_regex: false };
        // Fields are pub -- readable from an external module boundary in real use (plan.rs);
        // here we just confirm direct field access compiles from this same crate.
        assert_eq!(t.modifier, Modifier::Optional);
        assert_eq!(t.pattern, "x");
        assert!(!t.is_regex);
    }

    // AC-009: empty/whitespace-only (ASCII and Unicode) input short-circuits before any Lexer exists.
    #[test]
    fn empty_and_ascii_whitespace_input_returns_empty_vec() {
        assert!(parse("").unwrap().is_empty());
        assert!(parse("   \t\n  ").unwrap().is_empty());
    }

    #[test]
    fn nbsp_only_input_returns_empty_vec_via_unicode_trim() {
        // U+00A0 NBSP is Unicode whitespace (str::trim), so trim().is_empty() is true.
        let terms = parse("\u{00A0}").unwrap();
        assert!(terms.is_empty());
    }

    #[test]
    fn nbsp_prefixed_word_is_not_trimmed_away_and_lexer_keeps_the_nbsp() {
        // trim() strips the leading NBSP only for the emptiness check; the Lexer
        // still operates on the original untrimmed string, and its ASCII-only
        // skip_whitespace() does NOT skip U+00A0, so it becomes part of the word.
        let terms = parse("\u{00A0}foo").unwrap();
        assert_eq!(terms.len(), 1);
        assert_eq!(terms[0].modifier, Modifier::Optional);
        assert!(!terms[0].is_regex);
        assert_eq!(terms[0].pattern, "\u{00A0}foo");
    }

    // AC-010 / AC-012: Plus/Minus + Word or QuotedString content produce the
    // expected modifier with an escaped pattern; RegexPrefix/Plus/Minus/Eof as
    // content yield no term for that occurrence.
    #[test]
    fn plus_word_produces_required_escaped_term() {
        // "+foo.bar" (not "+foo") deliberately: "foo" has no regex
        // metacharacter, so regex::escape("foo") == "foo" and an assertion
        // re-computing regex::escape() from the same literal self-cancels
        // against a mutation that drops the escape() call entirely
        // (content_to_term's Word arm, AC-010). A hardcoded literal against
        // a metachar-containing input pins the escape actually happened.
        let terms = parse("+foo.bar").unwrap();
        assert_eq!(terms.len(), 1);
        assert_eq!(terms[0].modifier, Modifier::Required);
        assert_eq!(terms[0].pattern, "foo\\.bar");
        assert!(!terms[0].is_regex);
    }

    #[test]
    fn minus_quoted_produces_excluded_escaped_term() {
        let terms = parse("-\"a.b\"").unwrap();
        assert_eq!(terms.len(), 1);
        assert_eq!(terms[0].modifier, Modifier::Excluded);
        assert_eq!(terms[0].pattern, regex::escape("a.b"));
        assert!(!terms[0].is_regex);
    }

    #[test]
    fn bare_word_and_quoted_string_produce_optional_escaped_terms() {
        // Quoted content is "a.b" (not "a b") deliberately: a space is not a
        // regex metacharacter, so regex::escape("a b") == "a b" and an
        // assertion re-computing regex::escape() from the same literal
        // self-cancels against a mutation that drops the escape() call
        // entirely (the main loop's own Token::QuotedString arm, AC-012). A
        // hardcoded literal against a metachar-containing input pins the
        // escape actually happened.
        let terms = parse("foo.bar \"a.b\"").unwrap();
        assert_eq!(terms.len(), 2);
        assert_eq!(terms[0].modifier, Modifier::Optional);
        assert_eq!(terms[0].pattern, regex::escape("foo.bar"));
        assert!(!terms[0].is_regex);
        assert_eq!(terms[1].modifier, Modifier::Optional);
        assert_eq!(terms[1].pattern, "a\\.b");
    }

    #[test]
    fn dangling_plus_minus_at_eof_produce_no_term() {
        assert!(parse("+").unwrap().is_empty());
        assert!(parse("-").unwrap().is_empty());
    }

    #[test]
    fn plus_followed_by_plus_swallows_the_second_plus_as_content_then_reprocesses_foo() {
        // content = Token::Plus -> content_to_term returns None for the first
        // Plus occurrence. The second '+' is consumed as that lookahead and is
        // never itself seen by the main loop -- it is not rewound. The main
        // loop's next lexer.next_token() call then reads "foo" fresh as a bare
        // word, producing a single Optional (not Required) term -- the same
        // swallowed-lookahead-token pattern AC-011 documents for "regex: -foo".
        let terms = parse("+ +foo").unwrap();
        assert_eq!(terms.len(), 1);
        assert_eq!(terms[0].modifier, Modifier::Optional);
        assert!(!terms[0].is_regex);
        assert_eq!(terms[0].pattern, regex::escape("foo"));
    }

    // AC-007: whitespace between a modifier and its term is irrelevant.
    #[test]
    fn plus_and_minus_are_whitespace_insensitive_before_their_term() {
        let with_space = parse("+ foo").unwrap();
        let without_space = parse("+foo").unwrap();
        assert_eq!(with_space.len(), 1);
        assert_eq!(with_space[0].modifier, without_space[0].modifier);
        assert_eq!(with_space[0].pattern, without_space[0].pattern);
        assert_eq!(with_space[0].is_regex, without_space[0].is_regex);

        let with_space_m = parse("- foo").unwrap();
        let without_space_m = parse("-foo").unwrap();
        assert_eq!(with_space_m[0].modifier, without_space_m[0].modifier);
        assert_eq!(with_space_m[0].pattern, without_space_m[0].pattern);
    }

    // AC-011: bare RegexPrefix handling, including the "regex: -foo" full trace
    // and bare trailing "regex:" producing zero terms.
    #[test]
    fn regex_prefix_word_produces_optional_regex_term_raw_pattern() {
        let terms = parse("regex:fo.o").unwrap();
        assert_eq!(terms.len(), 1);
        assert_eq!(terms[0].modifier, Modifier::Optional);
        assert!(terms[0].is_regex);
        assert_eq!(terms[0].pattern, "fo.o"); // raw, NOT regex::escape'd
    }

    #[test]
    fn regex_prefix_quoted_produces_optional_regex_term_raw_pattern() {
        let terms = parse("regex:\"a.b\"").unwrap();
        assert_eq!(terms.len(), 1);
        assert!(terms[0].is_regex);
        assert_eq!(terms[0].pattern, "a.b");
    }

    #[test]
    fn bare_trailing_regex_prefix_produces_zero_terms() {
        assert!(parse("regex:").unwrap().is_empty());
    }

    // AC-004: the "regex:" prefix match is case-sensitive end-to-end through
    // the parser -- "REGEX:foo" is lexed as an ordinary Word and therefore
    // parses as a single Optional, non-regex, escaped-literal term on the
    // full "REGEX:foo" text, not a regex term on "foo".
    #[test]
    fn uppercase_regex_prefix_is_not_special_cased_by_the_parser() {
        let terms = parse("REGEX:foo").unwrap();
        assert_eq!(terms.len(), 1);
        assert_eq!(terms[0].modifier, Modifier::Optional);
        assert!(!terms[0].is_regex);
        assert_eq!(terms[0].pattern, regex::escape("REGEX:foo"));
    }

    #[test]
    fn regex_prefix_followed_by_minus_swallows_the_minus_and_reprocesses_foo() {
        // Fully traced per AC-011: "regex: -foo" -> exactly one Optional, non-regex
        // ParsedTerm on "foo"; the Minus token is consumed as pattern_tok and discarded.
        let terms = parse("regex: -foo").unwrap();
        assert_eq!(terms.len(), 1);
        assert_eq!(terms[0].modifier, Modifier::Optional);
        assert!(!terms[0].is_regex);
        assert_eq!(terms[0].pattern, "foo");
    }

    // AC-013: "+regex:foo" and "-regex:foo" degrade to a bare Optional,
    // non-regex, escaped-literal match on "foo" -- both modifier AND regex
    // semantics discarded, not merely the modifier.
    #[test]
    fn plus_regex_prefix_degrades_to_optional_escaped_literal_foo() {
        let terms = parse("+regex:foo").unwrap();
        assert_eq!(terms.len(), 1);
        assert_eq!(terms[0].modifier, Modifier::Optional);
        assert!(!terms[0].is_regex);
        assert_eq!(terms[0].pattern, regex::escape("foo"));
    }

    #[test]
    fn minus_regex_prefix_degrades_to_optional_escaped_literal_foo() {
        let terms = parse("-regex:foo").unwrap();
        assert_eq!(terms.len(), 1);
        assert_eq!(terms[0].modifier, Modifier::Optional);
        assert!(!terms[0].is_regex);
        assert_eq!(terms[0].pattern, regex::escape("foo"));
    }

    // AC-014: LIMIT=64 term cap, strictly greater-than.
    #[test]
    fn exactly_64_terms_succeeds() {
        let input: Vec<String> = (0..64).map(|i| format!("w{i}")).collect();
        let terms = parse(&input.join(" ")).unwrap();
        assert_eq!(terms.len(), 64);
    }

    #[test]
    fn sixty_five_terms_fails_with_too_many_terms() {
        let input: Vec<String> = (0..65).map(|i| format!("w{i}")).collect();
        let err = parse(&input.join(" ")).unwrap_err();
        match err {
            MonoklError::TooManyTerms { count, limit } => {
                assert_eq!(count, 65);
                assert_eq!(limit, 64);
            }
            other => panic!("expected TooManyTerms, got {other:?}"),
        }
        assert_eq!(err.to_string(), "query has too many terms: 65 > 64");
    }

    // AC-014: the LIMIT check runs only AFTER the main token-consuming loop
    // has fully completed, using the TRUE final term count -- not a check
    // inside the loop capped at some smaller number once the threshold is
    // crossed. A 64/65-boundary test alone can't distinguish "checked after
    // the loop with the true count" from "checked inside the loop, capped at
    // 65, then bailing early" -- both produce identical results at exactly
    // 64 or 65 terms. Parsing ~100 terms and asserting `count == 100` (not
    // 65, and not any other capped value) is the only way to observe that
    // every term was actually parsed before the check ran.
    #[test]
    fn limit_check_runs_after_full_loop_with_true_total_count_not_capped() {
        let input: Vec<String> = (0..100).map(|i| format!("w{i}")).collect();
        let err = parse(&input.join(" ")).unwrap_err();
        match err {
            MonoklError::TooManyTerms { count, limit } => {
                assert_eq!(count, 100);
                assert_eq!(limit, 64);
            }
            other => panic!("expected TooManyTerms, got {other:?}"),
        }
        assert_eq!(err.to_string(), "query has too many terms: 100 > 64");
    }

    // AC-015: no regex validity check at this layer -- an invalid pattern parses fine.
    #[test]
    fn invalid_regex_syntax_still_parses_successfully_unvalidated() {
        let terms = parse("regex:(unclosed").unwrap();
        assert_eq!(terms.len(), 1);
        assert!(terms[0].is_regex);
        assert_eq!(terms[0].pattern, "(unclosed");
    }
}
