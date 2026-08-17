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
        let terms = parse("+foo").unwrap();
        assert_eq!(terms.len(), 1);
        assert_eq!(terms[0].modifier, Modifier::Required);
        assert_eq!(terms[0].pattern, "foo");
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
        let terms = parse("foo.bar \"a b\"").unwrap();
        assert_eq!(terms.len(), 2);
        assert_eq!(terms[0].modifier, Modifier::Optional);
        assert_eq!(terms[0].pattern, regex::escape("foo.bar"));
        assert!(!terms[0].is_regex);
        assert_eq!(terms[1].modifier, Modifier::Optional);
        assert_eq!(terms[1].pattern, regex::escape("a b"));
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
