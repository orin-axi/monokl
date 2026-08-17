#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Plus,
    Minus,
    RegexPrefix,
    QuotedString(String),
    Word(String),
    Eof,
}

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn remaining(&self) -> &str {
        &self.input[self.pos..]
    }

    fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_whitespace() {
                self.pos += ch.len_utf8();
            } else {
                break;
            }
        }
    }

    fn read_word(&mut self) -> String {
        let mut s = String::new();
        loop {
            match self.peek() {
                None => break,
                Some(ch) if ch.is_ascii_whitespace() => break,
                Some(ch) => {
                    s.push(ch);
                    self.pos += ch.len_utf8();
                }
            }
        }
        s
    }

    fn read_quoted(&mut self) -> String {
        let mut s = String::new();
        loop {
            match self.advance() {
                None | Some('"') => break,
                Some('\\') => {
                    match self.peek() {
                        Some('"') => {
                            self.pos += 1;
                            s.push('"');
                        }
                        _ => s.push('\\'),
                    }
                }
                Some(ch) => s.push(ch),
            }
        }
        s
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        match self.peek() {
            None => Token::Eof,
            Some('+') => { self.pos += 1; Token::Plus }
            Some('-') => { self.pos += 1; Token::Minus }
            Some('"') => { self.pos += 1; Token::QuotedString(self.read_quoted()) }
            Some(_) => {
                if self.remaining().starts_with("regex:") {
                    self.pos += "regex:".len();
                    Token::RegexPrefix
                } else {
                    Token::Word(self.read_word())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // AC-001: exactly 6 variants, Debug+Clone+PartialEq only, no Eq/Copy/serde.
    fn assert_token_derives<T: std::fmt::Debug + Clone + PartialEq>() {}
    #[test]
    fn token_derive_bound_and_variant_shapes() {
        assert_token_derives::<Token>();
        let _ = [Token::Plus, Token::Minus, Token::RegexPrefix, Token::Eof,
                 Token::QuotedString(String::new()), Token::Word(String::new())];
        assert_eq!(Token::Plus, Token::Plus);
        assert_ne!(Token::Plus, Token::Minus);
    }

    // AC-004: dispatch order for each first-character case, including exact byte consumption.
    #[test]
    fn next_token_eof_on_empty_input() {
        let mut l = Lexer::new("");
        assert_eq!(l.next_token(), Token::Eof);
    }

    #[test]
    fn next_token_plus_consumes_one_byte_regardless_of_follower() {
        let mut l = Lexer::new("+");
        assert_eq!(l.next_token(), Token::Plus);
        assert_eq!(l.pos, 1);

        let mut l2 = Lexer::new("+x");
        assert_eq!(l2.next_token(), Token::Plus);
        assert_eq!(l2.pos, 1);
    }

    #[test]
    fn next_token_minus_consumes_one_byte_regardless_of_follower() {
        let mut l = Lexer::new("-");
        assert_eq!(l.next_token(), Token::Minus);
        assert_eq!(l.pos, 1);
    }

    #[test]
    fn next_token_quote_consumes_opening_quote_then_delegates() {
        let mut l = Lexer::new("\"ab\"");
        assert_eq!(l.next_token(), Token::QuotedString("ab".to_string()));
    }

    #[test]
    fn next_token_regex_prefix_checked_before_word_fallback() {
        let mut l = Lexer::new("regex:foo");
        assert_eq!(l.next_token(), Token::RegexPrefix);
        assert_eq!(l.pos, 6);
        assert_eq!(l.next_token(), Token::Word("foo".to_string()));
    }

    #[test]
    fn next_token_word_fallback_for_non_special_non_regex_input() {
        let mut l = Lexer::new("hello");
        assert_eq!(l.next_token(), Token::Word("hello".to_string()));
    }

    #[test]
    fn next_token_skips_leading_ascii_whitespace_unconditionally() {
        let mut l = Lexer::new("   \t\n  +foo");
        assert_eq!(l.next_token(), Token::Plus);
        assert_eq!(l.next_token(), Token::Word("foo".to_string()));
    }

    // AC-005: read_word greedily consumes until whitespace/EOF; '+'/'-'/'"'
    // and "regex:" are ordinary mid-word characters.
    #[test]
    fn word_does_not_split_on_internal_plus() {
        let mut l = Lexer::new("foo+bar");
        assert_eq!(l.next_token(), Token::Word("foo+bar".to_string()));
        assert_eq!(l.next_token(), Token::Eof);
    }

    #[test]
    fn word_does_not_split_on_internal_minus_quote_or_regex_prefix() {
        let mut l = Lexer::new("a-b\"cregex:d");
        assert_eq!(l.next_token(), Token::Word("a-b\"cregex:d".to_string()));
    }

    #[test]
    fn word_stops_at_whitespace() {
        let mut l = Lexer::new("foo bar");
        assert_eq!(l.next_token(), Token::Word("foo".to_string()));
        assert_eq!(l.next_token(), Token::Word("bar".to_string()));
    }

    // AC-006: read_quoted's single-escape rule and silent EOF tolerance.
    // Example 1: quote f o o quote -> "foo", 3-char payload.
    #[test]
    fn quoted_basic_no_backslash() {
        let mut l = Lexer::new("\"foo\"");
        assert_eq!(l.next_token(), Token::QuotedString("foo".to_string()));
    }

    // Example 2: quote a backslash quote b quote -> "a\"b" (escaped quote, 3-char payload).
    #[test]
    fn quoted_escaped_quote_appended_literally_does_not_terminate() {
        let mut l = Lexer::new("\"a\\\"b\"");
        let tok = l.next_token();
        assert_eq!(tok, Token::QuotedString("a\"b".to_string()));
        if let Token::QuotedString(s) = tok {
            assert_eq!(s.chars().count(), 3);
        }
    }
}
