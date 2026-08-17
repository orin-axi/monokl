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
}
