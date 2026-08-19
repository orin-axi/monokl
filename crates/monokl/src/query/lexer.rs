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

    // AC-001: absence of Eq/Hash/PartialOrd/Ord. A plain generic bound-check
    // function (like assert_token_derives above) can only prove a trait IS
    // implemented, never that it is absent -- this project's own
    // tests_types.rs (SPEC-006/007/008) establishes the fix: a const-
    // specialization probe pair per trait, where the "not implemented" const
    // is the default (false) and only overridden to true by a blanket impl
    // gated on the trait bound. If the probed type doesn't implement the
    // trait, only the default (false) impl applies.
    struct EqProbe<T>(std::marker::PhantomData<T>);
    trait NotEq { const IS: bool = false; }
    impl<T> NotEq for EqProbe<T> {}
    impl<T: Eq> EqProbe<T> { const IS: bool = true; }

    struct HashProbe<T>(std::marker::PhantomData<T>);
    trait NotHash { const IS: bool = false; }
    impl<T> NotHash for HashProbe<T> {}
    impl<T: std::hash::Hash> HashProbe<T> { const IS: bool = true; }

    struct PartialOrdProbe<T>(std::marker::PhantomData<T>);
    trait NotPartialOrd { const IS: bool = false; }
    impl<T> NotPartialOrd for PartialOrdProbe<T> {}
    impl<T: PartialOrd> PartialOrdProbe<T> { const IS: bool = true; }

    struct OrdProbe<T>(std::marker::PhantomData<T>);
    trait NotOrd { const IS: bool = false; }
    impl<T> NotOrd for OrdProbe<T> {}
    impl<T: Ord> OrdProbe<T> { const IS: bool = true; }

    #[test]
    fn token_absence_of_eq_hash_partialord_ord_pinned() {
        assert!(!EqProbe::<Token>::IS);
        assert!(!HashProbe::<Token>::IS);
        assert!(!PartialOrdProbe::<Token>::IS);
        assert!(!OrdProbe::<Token>::IS);
        // Positive controls (prove each probe itself isn't just always false).
        assert!(EqProbe::<usize>::IS);
        assert!(HashProbe::<usize>::IS);
        assert!(PartialOrdProbe::<usize>::IS);
        assert!(OrdProbe::<usize>::IS);
    }

    // AC-001: absence of serde Serialize/Deserialize on Token. Same
    // const-specialization technique as plan.rs's QueryPlan serde-absence
    // probe (SPEC-006 AC-015 / SPEC-008 AC-002 precedent this criterion
    // itself cites).
    struct SerProbe<T>(std::marker::PhantomData<T>);
    trait NotSer { const IS: bool = false; }
    impl<T> NotSer for SerProbe<T> {}
    impl<T: serde::Serialize> SerProbe<T> { const IS: bool = true; }

    struct DeProbe<T>(std::marker::PhantomData<T>);
    trait NotDe { const IS: bool = false; }
    impl<T> NotDe for DeProbe<T> {}
    impl<'de, T: serde::de::DeserializeOwned> DeProbe<T> { const IS: bool = true; }

    #[test]
    fn token_absence_of_serde_derive_pinned() {
        assert!(!SerProbe::<Token>::IS);
        assert!(!DeProbe::<Token>::IS);
        // Positive control (proves the probes themselves aren't just always
        // false): usize is Serialize + DeserializeOwned.
        assert!(SerProbe::<usize>::IS);
        assert!(DeProbe::<usize>::IS);
    }

    // AC-001: Token's exactly-6-variants declaration order (Plus, Minus,
    // RegexPrefix, QuotedString, Word, Eof), pinned via structural source
    // inspection -- a fieldless-comparison test can't observe enum variant
    // declaration ORDER for a data-carrying enum (no discriminant cast is
    // possible once any variant holds a payload), so this mirrors mod.rs's
    // existing line-order-pinning technique instead: read this file's own
    // verbatim source and assert each variant name appears at a strictly
    // increasing text position within Token's own declaration block.
    #[test]
    fn token_variant_declaration_order_structural() {
        let src = include_str!("lexer.rs");
        let decl_start = src.find("pub enum Token").unwrap_or_else(|| panic!("`pub enum Token` not found in lexer.rs"));
        let open = src[decl_start..].find('{').unwrap() + decl_start;
        let close = src[open..].find('}').unwrap() + open;
        let block = &src[open..=close];

        let variants = ["Plus", "Minus", "RegexPrefix", "QuotedString", "Word", "Eof"];
        let mut last_pos: Option<usize> = None;
        for v in variants {
            let pos = block.find(v).unwrap_or_else(|| panic!("variant `{v}` not found in Token enum body"));
            if let Some(last) = last_pos {
                assert!(pos > last, "variant `{v}` found out of the expected declaration order");
            }
            last_pos = Some(pos);
        }
    }

    // AC-002: Lexer<'a> implements none of Debug/Clone/PartialEq/Eq/Hash/
    // PartialOrd/Ord/Default/Copy. The structural text-window check below
    // (lexer_struct_has_no_derive_and_private_fields_structural) only scans
    // the source window immediately preceding `pub struct Lexer` up to the
    // last blank line -- a `#[derive(Debug, Clone)]` placed on its own line
    // with a blank line separating it from the struct (legal Rust) would
    // slip past that window undetected. AC-002 is purely a trait-absence
    // claim, which this codebase's established const-specialization probe
    // technique (see Token's Eq/Hash/PartialOrd/Ord absence probes above,
    // and parser.rs's ParsedTerm probes) proves directly at the type level,
    // regardless of where in the source a derive attribute is placed --
    // this closes that blind spot; the structural check below still covers
    // its own private-field-detection role.
    struct LexerDebugProbe<T>(std::marker::PhantomData<T>);
    trait NotLexerDebug { const IS: bool = false; }
    impl<T> NotLexerDebug for LexerDebugProbe<T> {}
    impl<T: std::fmt::Debug> LexerDebugProbe<T> { const IS: bool = true; }

    struct LexerCloneProbe<T>(std::marker::PhantomData<T>);
    trait NotLexerClone { const IS: bool = false; }
    impl<T> NotLexerClone for LexerCloneProbe<T> {}
    impl<T: Clone> LexerCloneProbe<T> { const IS: bool = true; }

    struct LexerPartialEqProbe<T>(std::marker::PhantomData<T>);
    trait NotLexerPartialEq { const IS: bool = false; }
    impl<T> NotLexerPartialEq for LexerPartialEqProbe<T> {}
    impl<T: PartialEq> LexerPartialEqProbe<T> { const IS: bool = true; }

    struct LexerEqProbe<T>(std::marker::PhantomData<T>);
    trait NotLexerEq { const IS: bool = false; }
    impl<T> NotLexerEq for LexerEqProbe<T> {}
    impl<T: Eq> LexerEqProbe<T> { const IS: bool = true; }

    struct LexerHashProbe<T>(std::marker::PhantomData<T>);
    trait NotLexerHash { const IS: bool = false; }
    impl<T> NotLexerHash for LexerHashProbe<T> {}
    impl<T: std::hash::Hash> LexerHashProbe<T> { const IS: bool = true; }

    struct LexerPartialOrdProbe<T>(std::marker::PhantomData<T>);
    trait NotLexerPartialOrd { const IS: bool = false; }
    impl<T> NotLexerPartialOrd for LexerPartialOrdProbe<T> {}
    impl<T: PartialOrd> LexerPartialOrdProbe<T> { const IS: bool = true; }

    struct LexerOrdProbe<T>(std::marker::PhantomData<T>);
    trait NotLexerOrd { const IS: bool = false; }
    impl<T> NotLexerOrd for LexerOrdProbe<T> {}
    impl<T: Ord> LexerOrdProbe<T> { const IS: bool = true; }

    struct LexerDefaultProbe<T>(std::marker::PhantomData<T>);
    trait NotLexerDefault { const IS: bool = false; }
    impl<T> NotLexerDefault for LexerDefaultProbe<T> {}
    impl<T: Default> LexerDefaultProbe<T> { const IS: bool = true; }

    struct LexerCopyProbe<T>(std::marker::PhantomData<T>);
    trait NotLexerCopy { const IS: bool = false; }
    impl<T> NotLexerCopy for LexerCopyProbe<T> {}
    impl<T: Copy> LexerCopyProbe<T> { const IS: bool = true; }

    #[test]
    fn lexer_implements_none_of_the_common_derivable_traits_pinned() {
        assert!(!LexerDebugProbe::<Lexer<'static>>::IS);
        assert!(!LexerCloneProbe::<Lexer<'static>>::IS);
        assert!(!LexerPartialEqProbe::<Lexer<'static>>::IS);
        assert!(!LexerEqProbe::<Lexer<'static>>::IS);
        assert!(!LexerHashProbe::<Lexer<'static>>::IS);
        assert!(!LexerPartialOrdProbe::<Lexer<'static>>::IS);
        assert!(!LexerOrdProbe::<Lexer<'static>>::IS);
        assert!(!LexerDefaultProbe::<Lexer<'static>>::IS);
        assert!(!LexerCopyProbe::<Lexer<'static>>::IS);
        // Positive controls (prove each probe itself isn't just always
        // false): usize implements all 9 of these traits.
        assert!(LexerDebugProbe::<usize>::IS);
        assert!(LexerCloneProbe::<usize>::IS);
        assert!(LexerPartialEqProbe::<usize>::IS);
        assert!(LexerEqProbe::<usize>::IS);
        assert!(LexerHashProbe::<usize>::IS);
        assert!(LexerPartialOrdProbe::<usize>::IS);
        assert!(LexerOrdProbe::<usize>::IS);
        assert!(LexerDefaultProbe::<usize>::IS);
        assert!(LexerCopyProbe::<usize>::IS);
    }

    // AC-002: Lexer<'a> carries NO derive attribute of any kind. This is an
    // absence-of-any-attribute-at-all claim, not a specific-trait claim a
    // const-specialization probe can address (there's no single trait to
    // probe for "has some derive"), so it's pinned via structural source
    // inspection instead: read this file's own verbatim source and confirm
    // no `#[derive(` attribute appears directly above `pub struct Lexer`,
    // and that neither of its 2 fields (input, pos) is declared `pub`.
    #[test]
    fn lexer_struct_has_no_derive_and_private_fields_structural() {
        // NB: `include_str!` pulls in this very test file, including this
        // test's own source text -- so the substrings searched for below
        // must never be spelled out literally in this function's own code
        // (e.g. as a field-name needle), or the test would trivially match
        // itself. Field names are assembled at runtime via `format!` to
        // avoid that self-reference trap, and the search is scoped to the
        // struct's own declaration block (not the whole file) so the test
        // module further down can't be accidentally scanned either.
        let src = include_str!("lexer.rs");
        let decl_start = src
            .find("pub struct Lexer<'a>")
            .unwrap_or_else(|| panic!("`pub struct Lexer<'a>` not found in lexer.rs"));
        let preceding = &src[..decl_start];
        // The nearest blank line before the struct decl is a safe boundary:
        // this codebase's convention (see Token above) places a derive
        // attribute on the line immediately above its struct/enum, with no
        // blank line in between -- so a window starting after the last blank
        // line captures any derive attribute meant for THIS struct without
        // reaching back into a prior, unrelated item's own derive.
        let window_start = preceding.rfind("\n\n").map(|p| p + 2).unwrap_or(0);
        let window = &preceding[window_start..decl_start];
        assert!(
            !window.contains("#[derive("),
            "found unexpected #[derive(...)] immediately before `pub struct Lexer`: {window:?}"
        );

        let open = src[decl_start..].find('{').unwrap() + decl_start;
        let close = src[open..].find('}').unwrap() + open;
        let field_block = &src[open..=close];
        for field in ["input", "pos"] {
            // Line-scoped prefix check (not a `pub {field}:` substring search)
            // so `pub(crate) {field}:` and `pub(super) {field}:` are caught
            // too, not just the bare `pub {field}:` spelling -- a private
            // field's declaration line starts, after trimming, directly with
            // `{field}:`; any visibility modifier prefixes it.
            let needle = format!("{field}:");
            let idx = field_block.find(&needle)
                .unwrap_or_else(|| panic!("field `{field}` not found in Lexer struct body"));
            let line_start = field_block[..idx].rfind('\n').map(|p| p + 1).unwrap_or(0);
            let line_end = field_block[idx..].find('\n').map(|p| idx + p).unwrap_or(field_block.len());
            let line = field_block[line_start..line_end].trim_start();
            assert!(
                line.starts_with(&needle),
                "Lexer.{field} must be private (AC-002), found line {line:?} not starting with `{needle}` -- a pub/pub(crate)/pub(super) modifier would prefix it"
            );
        }
    }

    // AC-003: Lexer's public API surface is exactly `new` and `next_token`;
    // its 6 other methods (remaining, peek, advance, skip_whitespace,
    // read_word, read_quoted) are all private. Structural source inspection,
    // since a compile-time bound check has no way to assert a method is NOT
    // callable from outside the module.
    #[test]
    fn lexer_helper_methods_are_private_structural() {
        let src = include_str!("lexer.rs");
        for name in ["remaining", "peek", "advance", "skip_whitespace", "read_word", "read_quoted"] {
            // Line-scoped prefix check (not a `pub fn {name}` substring
            // search) so `pub(crate) fn {name}` and `pub(super) fn {name}`
            // are caught too, not just the bare `pub fn {name}` spelling --
            // a truly-private method's declaration line starts, after
            // trimming, directly with `fn `; any visibility modifier
            // prefixes it.
            let needle = format!("fn {name}");
            let idx = src.find(&needle)
                .unwrap_or_else(|| panic!("helper method `{name}` not found in lexer.rs"));
            let line_start = src[..idx].rfind('\n').map(|p| p + 1).unwrap_or(0);
            let line_end = src[idx..].find('\n').map(|p| idx + p).unwrap_or(src.len());
            let line = src[line_start..line_end].trim_start();
            assert!(
                line.starts_with("fn "),
                "helper method `{name}` is not declared with a bare `fn ` prefix (found line {line:?}) -- AC-003 requires it private, not pub/pub(crate)/pub(super)"
            );
        }
        // Positive control: `new` and `next_token` ARE pub.
        assert!(src.contains("pub fn new"));
        assert!(src.contains("pub fn next_token"));
    }

    // AC-003: Lexer's public API surface is EXACTLY 2 methods -- `new` and
    // `next_token` -- and no others. The per-name check above
    // (lexer_helper_methods_are_private_structural) can only catch a
    // mutation that makes one of the 6 ALREADY-NAMED helpers pub; it cannot
    // catch (a) a mutation that adds a brand-new 3rd `pub fn` to the impl
    // block (a method under a name this test suite never enumerates), (b) a
    // SECOND `impl<'a> Lexer<'a> { ... }` block elsewhere in the file
    // carrying an extra `pub fn`, since a first-match-only search would
    // never see it, or (c) a `pub(crate) fn` added to the existing block,
    // since a `starts_with("pub fn ")` filter doesn't match `pub(crate) fn`.
    // This test therefore: iterates every line in the file looking for an
    // impl-block declaration for Lexer (not just the first, and not tied to
    // one exact lifetime spelling -- see below), brace-matches each
    // occurrence independently via brace-depth matching (the naive
    // first-`}`-after-`{` approach used elsewhere in this file for the
    // struct/field block doesn't work here, since function bodies inside the
    // impl block contain their own nested braces) to find its own extent,
    // and -- within each block -- collects every line whose trimmed start
    // begins with "pub" and contains "fn " (a broad candidate filter that
    // catches `pub(crate) fn`/`pub(super) fn` as well as bare `pub fn`, so a
    // privacy-narrowed-but-still-public-looking method can't slip past
    // undetected). The union of all such lines across every impl block
    // occurrence must equal exactly `["pub fn new", "pub fn next_token"]` --
    // any additional method, public or `pub(crate)`, that isn't literally
    // one of those two exact strings makes the assertion fail, since it
    // either doesn't match "pub fn ..." at all or shows up as an unwanted
    // extra element.
    //
    // Impl-block detection is spelling-variant-tolerant: rather than
    // matching one literal string like `"impl<'a> Lexer<'a> {"`, this scans
    // every line whose trimmed text starts with "impl", contains "Lexer",
    // and ends with "{" -- so `impl<'b> Lexer<'b> {` or the
    // modern-idiomatic `impl Lexer<'_> {` (both legal, semantically
    // identical ways to write an impl block for this same type) are caught
    // too, not just the one lifetime spelling this file happens to use
    // today. A second impl block added under any such spelling, carrying an
    // extra `pub fn`, is therefore still detected.
    #[test]
    fn lexer_exactly_two_public_methods_structural() {
        let src = include_str!("lexer.rs");

        let mut pub_fn_lines: Vec<String> = Vec::new();
        let mut occurrences = 0;
        let mut byte_offset = 0usize;
        for line in src.split_inclusive('\n') {
            let trimmed = line.trim();
            let is_lexer_impl_decl =
                trimmed.starts_with("impl") && trimmed.contains("Lexer") && trimmed.ends_with('{');
            let line_len = line.len();
            if !is_lexer_impl_decl {
                byte_offset += line_len;
                continue;
            }
            occurrences += 1;
            let brace_rel = line.rfind('{').unwrap_or_else(|| {
                panic!("line matched as an impl-block decl but has no `{{`: {line:?}")
            });
            let open = byte_offset + brace_rel;
            byte_offset += line_len;
            debug_assert_eq!(&src[open..open + 1], "{");

            let mut depth: i32 = 0;
            let mut close = None;
            for (i, ch) in src[open..].char_indices() {
                match ch {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            close = Some(open + i);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let close = close.unwrap_or_else(|| {
                panic!("no matching close brace found for impl block starting at byte {open}")
            });
            let block = &src[open..=close];

            for l in block.lines() {
                let trimmed = l.trim_start();
                if trimmed.starts_with("pub") && trimmed.contains("fn ") {
                    let fn_pos = trimmed.find("fn ").unwrap_or_else(|| {
                        panic!("`contains(\"fn \")` true but `find(\"fn \")` failed for line {trimmed:?}")
                    });
                    let paren_rel = trimmed[fn_pos..].find('(').unwrap_or(trimmed.len() - fn_pos);
                    pub_fn_lines.push(trimmed[..fn_pos + paren_rel].trim_end().to_string());
                }
            }
        }
        assert!(occurrences > 0, "no `impl ... Lexer ... {{` block found in lexer.rs");

        assert_eq!(pub_fn_lines, vec!["pub fn new".to_string(), "pub fn next_token".to_string()]);
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

    // AC-004: the "regex:" prefix check requires the trailing colon --
    // "regexp" (starts with "regex" but has no colon) must NOT match the
    // prefix check and must fall through to read_word() as an ordinary
    // word. A mutant check of `starts_with("regex")` (colon dropped) would
    // still match "regexp"'s first 5 bytes and, since the byte-skip
    // afterward is the hardcoded `"regex:".len()` == 6, would consume all 6
    // bytes of "regexp" as a RegexPrefix token, leaving nothing behind.
    #[test]
    fn regex_prefix_requires_the_trailing_colon() {
        let mut l = Lexer::new("regexp");
        assert_eq!(l.next_token(), Token::Word("regexp".to_string()));
        assert_eq!(l.next_token(), Token::Eof);

        let mut l2 = Lexer::new("regex foo");
        assert_eq!(l2.next_token(), Token::Word("regex".to_string()));
        assert_eq!(l2.next_token(), Token::Word("foo".to_string()));
    }

    // AC-004: the "regex:" prefix check is case-sensitive -- "REGEX:"/"Regex:"
    // are ordinary Word content, not RegexPrefix.
    #[test]
    fn next_token_regex_prefix_check_is_case_sensitive_uppercase() {
        let mut l = Lexer::new("REGEX:foo");
        assert_eq!(l.next_token(), Token::Word("REGEX:foo".to_string()));
    }

    #[test]
    fn next_token_regex_prefix_check_is_case_sensitive_mixed_case() {
        let mut l = Lexer::new("Regex:foo");
        assert_eq!(l.next_token(), Token::Word("Regex:foo".to_string()));
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

    // AC-005: read_word's stopping predicate is the real `is_ascii_whitespace()`,
    // not a narrower one (e.g. plain space, or space+tab only) -- every other
    // read_word test in this file uses a plain space or no separator at all, so
    // none of them can distinguish the real predicate from a narrower mutant.
    #[test]
    fn word_stops_at_every_ascii_whitespace_kind_not_just_space() {
        let mut l = Lexer::new("foo\tbar\nbaz\rqux");
        assert_eq!(l.next_token(), Token::Word("foo".to_string()));
        assert_eq!(l.next_token(), Token::Word("bar".to_string()));
        assert_eq!(l.next_token(), Token::Word("baz".to_string()));
        assert_eq!(l.next_token(), Token::Word("qux".to_string()));
        assert_eq!(l.next_token(), Token::Eof);
    }

    // AC-005: read_word's stopping predicate covers form feed (0x0C) too --
    // `is_ascii_whitespace()` includes it, but the sibling test above only
    // exercises tab/newline/CR, so a hand-rolled predicate like
    // `matches!(ch, ' '|'\t'|'\n'|'\r')` (missing form feed) would still
    // pass every other test in this file while failing to stop at 0x0C.
    #[test]
    fn word_stops_at_form_feed() {
        let mut l = Lexer::new("foo\x0Cbar");
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

    // Example 3: quote a backslash backslash b quote -> "a\\b" (two literal
    // backslashes, 4-char payload) -- neither backslash is treated as an escape.
    #[test]
    fn quoted_double_backslash_neither_is_an_escape() {
        let mut l = Lexer::new("\"a\\\\b\"");
        let tok = l.next_token();
        assert_eq!(tok, Token::QuotedString("a\\\\b".to_string()));
        if let Token::QuotedString(s) = tok {
            assert_eq!(s.chars().count(), 4);
            assert_eq!(s.matches('\\').count(), 2);
        }
    }

    // Example 4: quote a backslash, EOF immediately after -- unterminated AND
    // trailing-escape tolerance compose silently, no error, 2-char payload.
    #[test]
    fn quoted_unterminated_with_trailing_backslash_tolerated_silently() {
        let mut l = Lexer::new("\"a\\");
        let tok = l.next_token();
        assert_eq!(tok, Token::QuotedString("a\\".to_string()));
        if let Token::QuotedString(s) = tok {
            assert_eq!(s.chars().count(), 2);
        }
    }

    #[test]
    fn quoted_unterminated_no_backslash_returns_accumulated_chars_no_error() {
        let mut l = Lexer::new("\"abc");
        assert_eq!(l.next_token(), Token::QuotedString("abc".to_string()));
    }

    // AC-020: multi-byte UTF-8 safety, exact traced example.
    #[test]
    fn word_consumes_multibyte_utf8_char_as_one_char_no_panic() {
        let mut l = Lexer::new("+café");
        assert_eq!(l.next_token(), Token::Plus);
        assert_eq!(l.next_token(), Token::Word("café".to_string()));
    }

    #[test]
    fn quoted_consumes_multibyte_utf8_chars_no_panic() {
        let mut l = Lexer::new("\"日本語\"");
        assert_eq!(l.next_token(), Token::QuotedString("日本語".to_string()));
    }

    // AC-006: read_quoted's wildcard arm appends any character other than a
    // closing quote or backslash to the result as-is -- including ASCII
    // whitespace. Every other quoted-string test in this file uses a
    // whitespace-free payload, so none of them can distinguish the real
    // wildcard arm from a mutant that special-cases whitespace (truncating
    // at the first space, or silently dropping spaces from the payload).
    #[test]
    fn quoted_string_contains_ascii_whitespace_verbatim() {
        let mut l = Lexer::new("\"a b\tc\"");
        assert_eq!(l.next_token(), Token::QuotedString("a b\tc".to_string()));
        assert_eq!(l.next_token(), Token::Eof);
    }
}
