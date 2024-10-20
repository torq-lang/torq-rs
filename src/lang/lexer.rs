/*
 * Copyright (c) 2024 Torqware LLC. All rights reserved.
 *
 * You should have received a copy of the Torq Lang License v1.0 along with this program.
 * If not, see http://torq-lang.github.io/licensing/torq-lang-license-v1_0.
 */

//! Lexer creates tokens as slices of its source input. No additional strings are created.
//! Typically, the caller will iterate the source as tokens to build a vector of tokens.
//!
//! Max source size is 2,147,483,648 bytes.

use std::str::Chars;

#[cfg(test)]
mod tests;

// ERROR MESSAGES
const COMMENT_IS_MISSING_CLOSING_SEQUENCE: &str = "Comment is missing closing sequence '*/'";
const FLOATING_POINT_SUFFIX_MUST_BE_ONE_OF: &str = "Floating point suffix must be one of [fFdDmM]";
const IDENT_IS_MISSING_CLOSING_BACKTICK: &str = "Identifier is missing closing backtick";
const INTEGER_SUFFIX_MUST_BE_ONE_OF: &str = "Integer suffix must be one of [lLmM]";
const INVALID_DECIMAL_NUMBER: &str = "Invalid decimal number";
const INVALID_FLOATING_POINT_NUMBER: &str = "Invalid floating point number";
const INVALID_HEXADECIMAL_NUMBER: &str = "Invalid hexadecimal number";
const INVALID_INTEGER_NUMBER: &str = "Invalid integer";
const STR_IS_MISSING_CLOSING_DOUBLE_QUOTE: &str = "String is missing closing double quote";
const STR_IS_MISSING_CLOSING_SINGLE_QUOTE: &str = "String is missing closing single quote";
const UNRECOGNIZED_TOKEN: &str = "Unrecognized token";

// KEYWORDS
// Length = 2 (four words total)
const DO_VALUE: &str = "do";
const IF_VALUE: &str = "if";
const IN_VALUE: &str = "in";
const OF_VALUE: &str = "of";
// Length = 3 (six words total)
const ACT_VALUE: &str = "act";
const END_VALUE: &str = "end";
const EOF_VALUE: &str = "eof";
const FOR_VALUE: &str = "for";
const TRY_VALUE: &str = "try";
const VAR_VALUE: &str = "var";
// Length = 4 (ten words total)
const CASE_VALUE: &str = "case";
const ELSE_VALUE: &str = "else";
const FUNC_VALUE: &str = "func";
const NULL_VALUE: &str = "null";
const PROC_VALUE: &str = "proc";
const SELF_VALUE: &str = "self";
const SKIP_VALUE: &str = "skip";
const THEN_VALUE: &str = "then";
const TRUE_VALUE: &str = "true";
const WHEN_VALUE: &str = "when";
// Length = 5 (nine words total)
const ACTOR_VALUE: &str = "actor";
const BEGIN_VALUE: &str = "begin";
const BREAK_VALUE: &str = "break";
const CATCH_VALUE: &str = "catch";
const FALSE_VALUE: &str = "false";
const LOCAL_VALUE: &str = "local";
const SPAWN_VALUE: &str = "spawn";
const THROW_VALUE: &str = "throw";
const WHILE_VALUE: &str = "while";
// Length = 6 (three words total)
const ELSEIF_VALUE: &str = "elseif";
const IMPORT_VALUE: &str = "import";
const RETURN_VALUE: &str = "return";
// Length = 7
const FINALLY_VALUE: &str = "finally";
// Length = 8
const CONTINUE_VALUE: &str = "continue";

// WEAK KEYWORDS
const AS_VALUE: &str = "as";
const ASK_VALUE: &str = "ask";
const HANDLE_VALUE: &str = "handle";
const TELL_VALUE: &str = "tell";

// There are 27 delimiting chars that are the first char for one, two, or three char symbols.
// The following string slice is 27 bytes sorted.
//                                   1         2
//                         01 2345678901234567890123456
const DELIMITERS: &[u8] = "!\"#%&'()*+,-./:;<=>@[]`{|}~".as_bytes();

// There are 25 one char symbols below correlated to their position in the delimiter array.
// Positions 4 and 24 are blank (not used).
//                                         1         2
//                               01 2345678901234567890123456
const ONE_CHAR_SYMBOLS: &[u8] = "!\"#% '()*+,-./:;<=>@[]`{ }~".as_bytes();

// There are 11 two char symbols below correlated to their position in the delimiter array.
// The only positions used are 0, 4, 11, 14, 16, 17, 18, and 24.
const TWO_CHAR_SYMBOLS: [[u8; 2]; 27] = [
    [b'=', b' '], //  0: !=
    [b' ', b' '], //  1: not used
    [b' ', b' '], //  2: not used
    [b' ', b' '], //  3: not used
    [b'&', b' '], //  4: &&
    [b' ', b' '], //  5: not used
    [b' ', b' '], //  6: not used
    [b' ', b' '], //  7: not used
    [b' ', b' '], //  8: not used
    [b' ', b' '], //  9: not used
    [b' ', b' '], // 10: not used
    [b'>', b' '], // 11: ->
    [b' ', b' '], // 12: not used
    [b' ', b' '], // 13: not used
    [b'=', b':'], // 14: :=, ::
    [b' ', b' '], // 15: not used
    [b'=', b':'], // 16: <=, <:
    [b'=', b':'], // 17: ==
    [b'=', b':'], // 18: >=, >:
    [b' ', b' '], // 19: not used
    [b' ', b' '], // 20: not used
    [b' ', b' '], // 21: not used
    [b' ', b' '], // 22: not used
    [b' ', b' '], // 23: not used
    [b'|', b' '], // 24: ||
    [b' ', b' '], // 25: not used
    [b' ', b' '], // 26: not used
];

const EOF_TOKEN: Token = Token {
    value: "EOF",
    byte_index: -1,
    token_type: TokenType::Eof,
};

/// Character position is the UTF-8 character index and not the byte index. Because UTF-8
/// character encodings vary between 1 and 4 bytes, character positions in a source string are not
/// deterministic.
#[derive(Debug, PartialEq)]
pub struct Token<'a> {
    pub value: &'a str,
    pub byte_index: i32,
    pub token_type: TokenType,
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Char,
    Comment,
    Dec,
    Eof,
    Flt,
    Ident,
    Int,
    Keyword,
    OneCharSym,
    Str,
    ThreeCharSym,
    TwoCharSym,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct LexerIndex {
    char: char,
    char_index: i32,
    byte_index: i32,
}

pub struct LexerIter<'a> {
    source: &'a str,
    str_iter: Chars<'a>,
    current: Option<LexerIndex>,
    current_plus_1: Option<LexerIndex>,
}

#[derive(Debug, PartialEq)]
pub struct LexerError {
    message: &'static str,
    index: LexerIndex,
}

///
/// # Overview
///
/// ## Numbers
/// * Begin with a digit.
/// * Hex numbers begin with "0x" or "0X".
/// * Floating point numbers begin with a non-zero digit and have a fractional part.
/// * Scientific notation has an exponent marker "e" or "E" immediately after the fractional part.
/// * Scientific notation has an optional sign "-" or "+" after the exponent marker.
/// * Scientific notation has an exponent integer after the exponent designator or exponent sign.
/// * Any floating point number can be declared 64 bits with a suffix "d" or "D".
/// * Any floating point number can be declared 32 bits with a suffix "f" or "F".
/// * Numbers end with a separator comprised of whitespace or a delimiter.
///
/// ## Symbols
/// * Comprised of delimiters of one, two, or three chars in size.
/// * Parse eagerly such that "...." is parsed as "..." and "." instead of "." and "...".
///
/// ## Strings
/// * Quoted using double (") or single (') quotes.
/// * Quote chars and other special chars are escaped using the backslash (\) char.
/// * Backslash is escaped using a backslash.
///
/// ## Keywords
/// * Well-known alphanumeric char sequences.
///
/// ## Identifiers
/// * Alphanumeric char sequences not first recognized as identifiers.
/// * Any char sequence can be an identifier if quoted using backticks "`".
///
/// ## Comments
/// * Line comments begin with "//" char sequence and terminate with a newline char.
/// * Block comments begin with a "/*" char sequence and terminate with a "*/" char sequence.
///
impl<'a> LexerIter<'a> {
    fn current(&self) -> Option<LexerIndex> {
        self.current
    }

    fn fetch_next_char(str_iter: &mut Chars, current: &Option<LexerIndex>) -> Option<LexerIndex> {
        if let Some(next_char) = str_iter.next() {
            if let Some(current) = current {
                Some(LexerIndex {
                    char: next_char,
                    char_index: current.char_index + 1,
                    byte_index: current.byte_index + current.char.len_utf8() as i32,
                })
            } else {
                Some(LexerIndex {
                    char: next_char,
                    char_index: 0,
                    byte_index: 0,
                })
            }
        } else {
            None
        }
    }

    /*
     * Return 0 through 26 if delimiter is found. Otherwise, return -1.
     */
    fn index_of_delimiter(c: char) -> isize {
        let mut left = 0;
        let mut right = DELIMITERS.len() - 1;
        while left <= right {
            let mid = (left + right) / 2;
            match DELIMITERS[mid].cmp(&(c as u8)) {
                std::cmp::Ordering::Equal => return mid as isize,
                std::cmp::Ordering::Less => left = mid + 1,
                std::cmp::Ordering::Greater => right = mid - 1,
            }
        }
        -1
    }

    fn is_delimiter(c: char) -> bool {
        Self::index_of_delimiter(c) > -1
    }

    fn is_digit(c: char) -> bool {
        c >= '0' && c <= '9'
    }

    fn is_keyword(&self, start: usize, stop: usize) -> bool {
        //  There are 34 total keywords
        let length = stop - start + 1;
        if length == 4 {
            // There are 10 keywords of 4 chars
            self.is_keyword_of_4_chars(start, stop)
        } else if length == 5 {
            // There are 9 keywords of 5 chars
            self.is_keyword_of_5_chars(start, stop)
        } else if length == 3 {
            // There are 6 keywords of 3 chars
            self.is_keyword_of_3_chars(start, stop)
        } else if length == 2 {
            // There are 4 keywords of 2 chars
            self.is_keyword_of_2_chars(start, stop)
        } else if length == 6 {
            // There are 3 keywords of 6 chars
            self.is_keyword_of_6_chars(start, stop)
        } else if length == 7 {
            // There is 1 keyword of 7 chars
            self.is_keyword_of_7_chars(start, stop)
        } else if length == 8 {
            // There is 1 keyword of 8 chars
            self.is_keyword_of_8_chars(start, stop)
        } else {
            false
        }
    }

    fn is_keyword_of_2_chars(&self, start: usize, stop: usize) -> bool {
        let bytes = self.source.as_bytes();
        let b1 = bytes[start];
        if b1 == b'i' {
            let b2 = bytes[stop];
            b2 == b'n' || b2 == b'f'
        } else {
            if b1 == b'd' {
                bytes[stop] == b'o'
            } else if b1 == b'o' {
                bytes[stop] == b'f'
            } else {
                false
            }
        }
    }

    fn is_keyword_of_3_chars(&self, start: usize, stop: usize) -> bool {
        let bytes = self.source.as_bytes();
        let b1 = bytes[start];
        if b1 == b'v' {
            bytes[start + 1] == b'a' && bytes[stop] == b'r'
        } else if b1 == b'e' {
            let b2 = bytes[start + 1];
            if b2 == b'n' {
                bytes[stop] == b'd'
            } else if b2 == b'o' {
                bytes[stop] == b'f'
            } else {
                false
            }
        } else if b1 == b'f' {
            bytes[start + 1] == b'o' && bytes[stop] == b'r'
        } else if b1 == b't' {
            bytes[start + 1] == b'r' && bytes[stop] == b'y'
        } else if b1 == b'a' {
            bytes[start + 1] == b'c' && bytes[stop] == b't'
        } else {
            false
        }
    }

    fn is_keyword_of_4_chars(&self, start: usize, stop: usize) -> bool {
        let bytes = self.source.as_bytes();
        let b1 = bytes[start];
        if b1 < b'p' {
            // case, else, func, or null
            if b1 == b'c' {
                bytes[start + 1] == b'a' && bytes[start + 2] == b's' && bytes[stop] == b'e'
            } else if b1 == b'e' {
                bytes[start + 1] == b'l' && bytes[start + 2] == b's' && bytes[stop] == b'e'
            } else if b1 == b'f' {
                bytes[start + 1] == b'u' && bytes[start + 2] == b'n' && bytes[stop] == b'c'
            } else if b1 == b'n' {
                bytes[start + 1] == b'u' && bytes[start + 2] == b'l' && bytes[stop] == b'l'
            } else {
                false
            }
        } else {
            // proc, self, skip, then, true, or when
            if b1 == b'p' {
                bytes[start + 1] == b'r' && bytes[start + 2] == b'o' && bytes[stop] == b'c'
            } else if b1 == b's' {
                let b2 = bytes[start + 1];
                if b2 == b'e' {
                    bytes[start + 2] == b'l' && bytes[stop] == b'f'
                } else if b2 == b'k' {
                    bytes[start + 2] == b'i' && bytes[stop] == b'p'
                } else {
                    false
                }
            } else if b1 == b't' {
                let b2 = bytes[start + 1];
                if b2 == b'h' {
                    bytes[start + 2] == b'e' && bytes[stop] == b'n'
                } else if b2 == b'r' {
                    bytes[start + 2] == b'u' && bytes[stop] == b'e'
                } else {
                    false
                }
            } else if b1 == b'w' {
                bytes[start + 1] == b'h' && bytes[start + 2] == b'e' && bytes[stop] == b'n'
            } else {
                false
            }
        }
    }

    fn is_keyword_of_5_chars(&self, start: usize, stop: usize) -> bool {
        let bytes = self.source.as_bytes();
        let b1 = bytes[start];
        if b1 < b'l' {
            // actor, begin, break, catch, false,
            if b1 == b'a' {
                bytes[start + 1] == b'c'
                    && bytes[start + 2] == b't'
                    && bytes[start + 3] == b'o'
                    && bytes[stop] == b'r'
            } else if b1 == b'b' {
                let b2 = bytes[start + 1];
                if b2 == b'e' {
                    bytes[start + 2] == b'g' && bytes[start + 3] == b'i' && bytes[stop] == b'n'
                } else if b2 == b'r' {
                    bytes[start + 2] == b'e' && bytes[start + 3] == b'a' && bytes[stop] == b'k'
                } else {
                    false
                }
            } else if b1 == b'c' {
                bytes[start + 1] == b'a'
                    && bytes[start + 2] == b't'
                    && bytes[start + 3] == b'c'
                    && bytes[stop] == b'h'
            } else if b1 == b'f' {
                bytes[start + 1] == b'a'
                    && bytes[start + 2] == b'l'
                    && bytes[start + 3] == b's'
                    && bytes[stop] == b'e'
            } else {
                false
            }
        } else {
            // local, spawn, throw, while
            if b1 == b'l' {
                bytes[start + 1] == b'o'
                    && bytes[start + 2] == b'c'
                    && bytes[start + 3] == b'a'
                    && bytes[stop] == b'l'
            } else if b1 == b's' {
                bytes[start + 1] == b'p'
                    && bytes[start + 2] == b'a'
                    && bytes[start + 3] == b'w'
                    && bytes[stop] == b'n'
            } else if b1 == b't' {
                bytes[start + 1] == b'h'
                    && bytes[start + 2] == b'r'
                    && bytes[start + 3] == b'o'
                    && bytes[stop] == b'w'
            } else if b1 == b'w' {
                bytes[start + 1] == b'h'
                    && bytes[start + 2] == b'i'
                    && bytes[start + 3] == b'l'
                    && bytes[stop] == b'e'
            } else {
                false
            }
        }
    }

    fn is_keyword_of_6_chars(&self, start: usize, stop: usize) -> bool {
        // elseif, import, and return
        let bytes = self.source.as_bytes();
        let b1 = bytes[start];
        if b1 == b'e' {
            bytes[start + 1] == b'l'
                && bytes[start + 2] == b's'
                && bytes[start + 3] == b'e'
                && bytes[start + 4] == b'i'
                && bytes[stop] == b'f'
        } else if b1 == b'i' {
            bytes[start + 1] == b'm'
                && bytes[start + 2] == b'p'
                && bytes[start + 3] == b'o'
                && bytes[start + 4] == b'r'
                && bytes[stop] == b't'
        } else if b1 == b'r' {
            bytes[start + 1] == b'e'
                && bytes[start + 2] == b't'
                && bytes[start + 3] == b'u'
                && bytes[start + 4] == b'r'
                && bytes[stop] == b'n'
        } else {
            false
        }
    }

    fn is_keyword_of_7_chars(&self, start: usize, stop: usize) -> bool {
        // finally
        let bytes = self.source.as_bytes();
        let b1 = bytes[start];
        if b1 == b'f' {
            bytes[start + 1] == b'i'
                && bytes[start + 2] == b'n'
                && bytes[start + 3] == b'a'
                && bytes[start + 4] == b'l'
                && bytes[start + 5] == b'l'
                && bytes[stop] == b'y'
        } else {
            false
        }
    }

    fn is_keyword_of_8_chars(&self, start: usize, stop: usize) -> bool {
        // continue
        let bytes = self.source.as_bytes();
        let b1 = bytes[start];
        if b1 == b'c' {
            bytes[start + 1] == b'o'
                && bytes[start + 2] == b'n'
                && bytes[start + 3] == b't'
                && bytes[start + 4] == b'i'
                && bytes[start + 5] == b'n'
                && bytes[start + 6] == b'u'
                && bytes[stop] == b'e'
        } else {
            false
        }
    }

    fn is_keyword_or_ident_char(c: char) -> bool {
        c >= '0' && c <= '9' || c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c == '_'
    }

    fn is_eof_or_separator(index: Option<LexerIndex>) -> bool {
        index.is_none() || Self::is_separator(index.unwrap().char)
    }

    fn is_separator(c: char) -> bool {
        Self::is_whitespace(c) || Self::is_delimiter(c)
    }

    fn is_some_char(index: Option<LexerIndex>, c: char) -> bool {
        if index.is_none() {
            return false;
        }
        index.unwrap().char == c
    }

    fn is_some_digit(index: Option<LexerIndex>) -> bool {
        if index.is_none() {
            return false;
        }
        Self::is_digit(index.unwrap().char)
    }

    /*
     * U+0009 (horizontal tab, '\t')
     * U+000A (line feed, '\n')
     * U+000B (vertical tab)
     * U+000C (form feed)
     * U+000D (carriage return, '\r')
     * U+0020 (space, ' ')
     * U+0085 (next line)
     * U+200E (left-to-right mark)
     * U+200F (right-to-left mark)
     * U+2028 (line separator)
     * U+2029 (paragraph separator)
     */
    fn is_whitespace(c: char) -> bool {
        if c >= '\u{0009}' && c <= '\u{000D}' {
            return true;
        }
        if c == ' ' {
            return true;
        }
        if c == '\u{0085}' {
            return true;
        }
        if c == '\u{200E}' || c == '\u{200F}' {
            return true;
        }
        c == '\u{2028}' || c == '\u{2029}'
    }

    fn make_invalid_number_err(&self, start: LexerIndex, token_type: TokenType) -> LexerError {
        let m = if token_type == TokenType::Int {
            INVALID_INTEGER_NUMBER
        } else {
            INVALID_DECIMAL_NUMBER
        };
        LexerError {
            message: m,
            index: start,
        }
    }

    fn make_token(&self, start: LexerIndex, stop: LexerIndex, token_type: TokenType) -> Token<'a> {
        let token_start = start.byte_index as usize;
        let token_stop = stop.byte_index as usize + 1;
        Token {
            value: &self.source[token_start..token_stop],
            byte_index: start.byte_index,
            token_type,
        }
    }

    fn new(source: &'a str) -> Self {
        let mut str_iter = source.chars();
        let current = Self::fetch_next_char(&mut str_iter, &None);
        LexerIter {
            source,
            str_iter,
            current,
            current_plus_1: None,
        }
    }

    /*
     * Pre-condition:
     *   `current` is EOF, a separator, or the first char of the next token to parse.
     *
     * Post-condition:
     *   `current` is EOF, a separator, or the first char of the next token to parse.
     */
    fn next(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace();
        if self.current.is_none() {
            return Ok(EOF_TOKEN);
        }
        let current = self.current().unwrap();
        if Self::is_digit(current.char) {
            return self.parse_num();
        }
        if current.char == '\'' {
            return self.parse_single_quoted_str();
        }
        if current.char == '"' {
            return self.parse_double_quoted_str();
        }
        if current.char == '`' {
            return self.parse_quoted_ident();
        }
        if current.char == '/' && self.peek_1().is_some() {
            let p1 = self.current_plus_1.unwrap().char;
            if p1 == '/' {
                return self.parse_line_comment();
            } else if p1 == '*' {
                return self.parse_block_comment();
            }
        }
        let index = Self::index_of_delimiter(current.char);
        if index > -1 {
            // We know the delimiter is not for comments or quoted values.
            // The parse must be for a character or symbol.
            return self.parse_symbol(index as usize);
        }
        // Otherwise, we are parsing a keyword or identifier
        if Self::is_keyword_or_ident_char(current.char) {
            self.parse_keyword_or_ident()
        } else {
            Err(LexerError {
                message: UNRECOGNIZED_TOKEN,
                index: current,
            })
        }
    }

    fn next_char(&mut self) {
        if self.current_plus_1.is_some() {
            self.current = self.current_plus_1;
            self.current_plus_1 = None;
        } else {
            self.current = Self::fetch_next_char(&mut self.str_iter, &self.current);
        }
    }

    /*
     * Pre-condition:
     *   `current` is the "/" char of a block comment start.
     *   `current_plus_1` is the "*" char of the block comment start.
     *
     * Post-condition:
     *   `current` is the next char (or EOF) after the end comment sequence (asterisk-slash).
     */
    fn parse_block_comment(&mut self) -> Result<Token<'a>, LexerError> {
        let start = self.current.unwrap();
        // Accept `current` "/" and make "*" the new `current`
        self.next_char();
        // Accept `current` "*" and load next char or EOF as `current`
        self.next_char();
        // Accept all block chars except the ending "*/" char sequence
        fn is_block_content(index1: Option<LexerIndex>, index2: Option<LexerIndex>) -> bool {
            if index1.is_none() || index2.is_none() {
                return false;
            }
            index1.unwrap().char != '*' || index2.unwrap().char != '/'
        }
        while is_block_content(self.current, self.peek_1()) {
            self.next_char();
        }
        if self.current.is_none() || self.current_plus_1.is_none() {
            return Err(LexerError {
                message: COMMENT_IS_MISSING_CLOSING_SEQUENCE,
                index: start,
            });
        }
        // Accept `current` "*" and make "/" the new `current`
        self.next_char();
        let stop = self.current.unwrap();
        // Accept `current` "/" and load next char or EOF as `current`
        self.next_char();
        Ok(self.make_token(start, stop, TokenType::Comment))
    }

    /*
     * Pre-condition:
     *   `current` is the first double quote char.
     *
     * Post-condition:
     *   `current` is the next char (or EOF) after the close quote char '"'.
     */
    fn parse_double_quoted_str(&mut self) -> Result<Token<'a>, LexerError> {
        let start = self.current.unwrap();
        // Accept beginning double quote
        self.next_char();
        // Accept remaining string content
        fn is_str_content(index: Option<LexerIndex>) -> bool {
            if index.is_none() {
                return false;
            }
            index.unwrap().char != '"'
        }
        while is_str_content(self.current) {
            let c = self.current().unwrap().char;
            self.next_char();
            // Accept escaped character
            if c == '\\' {
                self.next_char();
            }
        }
        if self.current.is_none() {
            return Err(LexerError {
                message: STR_IS_MISSING_CLOSING_DOUBLE_QUOTE,
                index: start,
            });
        }
        let stop = self.current().unwrap();
        // Ensure that we have met our post-condition
        self.next_char();
        Ok(self.make_token(start, stop, TokenType::Str))
    }

    /*
     * Pre-condition:
     *   `start` is the first digit of a number.
     *   `current` is the "e" or "E" of the exponent part.
     *
     * Post-condition:
     *   `current` is EOF or a separator (whitespace or delimiter).
     */
    fn parse_fractional_exponent(&mut self, start: LexerIndex) -> Result<Token<'a>, LexerError> {
        // Accept the `e` or 'E` char
        self.next_char();
        if self.current.is_none() {
            return Err(LexerError {
                message: INVALID_FLOATING_POINT_NUMBER,
                index: start,
            });
        }
        // Check for the optional "-" or "+" chars
        let current_char = self.current.unwrap().char;
        if current_char == '-' || current_char == '+' {
            // Accept the `-` or '+` char
            self.next_char();
            if self.current.is_none() {
                return Err(LexerError {
                    message: INVALID_FLOATING_POINT_NUMBER,
                    index: start,
                });
            }
        }
        // We know that `current` is some char, but it must be a digit
        if !Self::is_digit(self.current.unwrap().char) {
            return Err(LexerError {
                message: INVALID_FLOATING_POINT_NUMBER,
                index: start,
            });
        }
        // Accept `current` digit while `current_plus_1` is a digit
        while Self::is_some_digit(self.peek_1()) {
            self.next_char();
        }
        let last_exponent_digit = self.current.unwrap();
        // Return a token if certain the optional suffix is not present
        if Self::is_eof_or_separator(self.peek_1()) {
            // Accept `current` if next char is a separator
            if self.current_plus_1.is_some() {
                self.next_char();
            }
            return Ok(self.make_token(start, last_exponent_digit, TokenType::Flt));
        }
        // Load the optional suffix if present
        let p1 = self.current_plus_1.unwrap().char;
        if p1 == 'f' || p1 == 'F' || p1 == 'd' || p1 == 'D' {
            // Make suffix `current`
            self.next_char();
        }
        // We don't have an EOF or separator--`current` will be a digit or suffix
        let stop = self.current.unwrap();
        // Accept last digit or suffix, loading `current` with the first char beyond our number
        self.next_char();
        // Ensure that we have met our post-condition
        if !Self::is_eof_or_separator(self.current) {
            return Err(LexerError {
                message: INVALID_FLOATING_POINT_NUMBER,
                index: start,
            });
        }
        Ok(self.make_token(start, stop, TokenType::Flt))
    }

    /*
     * Pre-condition:
     *   `start` is the first digit of a number.
     *   `current` is the first digit of the fractional part.
     *
     * Post-condition:
     *   `current` is EOF or a separator (whitespace or delimiter).
     */
    fn parse_fractional_part(&mut self, start: LexerIndex) -> Result<Token<'a>, LexerError> {
        while Self::is_some_digit(self.peek_1()) {
            self.next_char();
        }
        let last_fractional_digit = self.current.unwrap();
        // Return a token if the optional exponent or suffix is not present
        if Self::is_eof_or_separator(self.peek_1()) {
            // Accept `current` if next char is a separator
            if self.current_plus_1.is_some() {
                self.next_char();
            }
            return Ok(self.make_token(start, last_fractional_digit, TokenType::Flt));
        }
        // 'current_plus_1' is loaded because we peeked above
        let possible_exponent_or_suffix = self.current_plus_1.unwrap();
        if possible_exponent_or_suffix.char == 'e' || possible_exponent_or_suffix.char == 'E' {
            // Make 'e' or 'E' `current`
            self.next_char();
            return self.parse_fractional_exponent(start);
        }
        // We don't have an EOF, separator, or exponent--`current_plus_1` must be a suffix
        let token_type = match possible_exponent_or_suffix.char {
            'f' | 'F' | 'd' | 'D' => TokenType::Flt,
            'm' | 'M' => TokenType::Dec,
            _ => {
                return Err(LexerError {
                    message: FLOATING_POINT_SUFFIX_MUST_BE_ONE_OF,
                    index: start,
                })
            }
        };
        // Make suffix `current`
        self.next_char();
        // Accept suffix
        self.next_char();
        if Self::is_eof_or_separator(self.current) {
            Ok(self.make_token(start, possible_exponent_or_suffix, token_type))
        } else {
            Err(self.make_invalid_number_err(start, token_type))
        }
    }

    /*
     * Keyword and identifier facts:
     *   - Keyword length is between 2 and 8, inclusively
     *   - Keyword domain is a subdomain of identifiers
     *
     * Pre-condition:
     *   `current` is the first char of a keyword or ident.
     *
     * Post-condition:
     *   `current` is EOF or separator (whitespace or delimiter)
     */
    fn parse_keyword_or_ident(&mut self) -> Result<Token<'a>, LexerError> {
        let start = self.current.unwrap();
        fn is_keyword_or_ident_content(index: Option<LexerIndex>) -> bool {
            if index.is_none() {
                return false;
            }
            LexerIter::is_keyword_or_ident_char(index.unwrap().char)
        }
        while is_keyword_or_ident_content(self.peek_1()) {
            self.next_char();
        }
        let stop = self.current.unwrap();
        // Ensure that we have met our post-condition
        self.next_char();
        if self.is_keyword(start.byte_index as usize, stop.byte_index as usize) {
            Ok(self.make_token(start, stop, TokenType::Keyword))
        } else {
            Ok(self.make_token(start, stop, TokenType::Ident))
        }
    }

    /*
     * Pre-condition:
     *   `current` is the first "/" char of the line comment.
     *   `current_plus_1` is the second "/" of the line comment.
     *
     * Post-condition:
     *   `current` is the next char (or EOF) after the end comment (newline char).
     */
    fn parse_line_comment(&mut self) -> Result<Token<'a>, LexerError> {
        let start = self.current.unwrap();
        // Accept `current` "/" and make second "/" the new `current`
        self.next_char();
        // Accept all line chars except newline or EOF
        fn is_line_content(index: Option<LexerIndex>) -> bool {
            if index.is_none() {
                return false;
            }
            index.unwrap().char != '\n'
        }
        while is_line_content(self.peek_1()) {
            self.next_char();
        }
        let stop = self.current.unwrap();
        // Accept either the second "/" or the last char of the line comment
        self.next_char();
        Ok(self.make_token(start, stop, TokenType::Comment))
    }

    /*
     * Pre-condition:
     *   `current` is the first digit of a number.
     *
     * Post-condition:
     *   `current` is EOF or a separator (whitespace or delimiter).
     */
    fn parse_num(&mut self) -> Result<Token<'a>, LexerError> {
        let start = self.current.unwrap();
        if start.char == '0' && self.peek_1().is_some() {
            let p1 = self.current_plus_1.unwrap().char;
            if p1 == 'x' || p1 == 'X' {
                // Make "x" or "X"  `current`
                self.next_char();
                if self.peek_1().is_none() {
                    return Err(LexerError {
                        message: INVALID_HEXADECIMAL_NUMBER,
                        index: start,
                    });
                }
                // Accept first hex digit
                self.next_char();
                // Accept remaining hex characters
                fn is_hex_content(index: Option<LexerIndex>) -> bool {
                    if index.is_none() {
                        return false;
                    }
                    let c = index.unwrap().char;
                    c >= '0' && c <= '9' || c >= 'a' && c <= 'f' || c >= 'A' && c <= 'F'
                }
                while is_hex_content(self.peek_1()) {
                    self.next_char();
                }
            }
        } else {
            while Self::is_some_digit(self.peek_1()) {
                // Make the next digit `current`
                self.next_char();
            }
        }
        // `current` is now the last digit of the whole number
        if Self::is_some_char(self.peek_1(), '.') {
            // Make the period `current`
            self.next_char();
            // Make the first char after the period 'current'
            self.next_char();
            // The first character after a period must be a digit
            if self.current.is_none() || !Self::is_digit(self.current.unwrap().char) {
                return Err(LexerError {
                    message: INVALID_FLOATING_POINT_NUMBER,
                    index: start,
                });
            }
            self.parse_fractional_part(start)
        } else {
            if Self::is_eof_or_separator(self.peek_1()) {
                let stop = self.current.unwrap();
                let token = self.make_token(start, stop, TokenType::Int);
                if self.current_plus_1.is_some() {
                    self.next_char();
                }
                Ok(token)
            } else {
                // We have a possible suffix (not EOF or a separator).
                // We have `current_plus_1` loaded because we peeked successfully above.
                let possible_suffix = self.current_plus_1.unwrap();
                let token_type = match possible_suffix.char {
                    'l' | 'L' => TokenType::Int,
                    'm' | 'M' => TokenType::Dec,
                    _ => {
                        return Err(LexerError {
                            message: INTEGER_SUFFIX_MUST_BE_ONE_OF,
                            index: start,
                        })
                    }
                };
                // Make suffix `current`
                self.next_char();
                // Accept suffix
                self.next_char();
                if Self::is_eof_or_separator(self.current) {
                    Ok(self.make_token(start, possible_suffix, token_type))
                } else {
                    // Suffix is combined with another char
                    Err(self.make_invalid_number_err(start, token_type))
                }
            }
        }
    }

    /*
     * Pre-condition:
     *   `current` is the first backtick char.
     *
     * Post-condition:
     *   `current` is the next char (or EOF) after the close quote char "`".
     */
    fn parse_quoted_ident(&mut self) -> Result<Token<'a>, LexerError> {
        let start = self.current.unwrap();
        // Accept beginning backtick
        self.next_char();
        // Accept remaining ident content
        fn is_ident_content(index: Option<LexerIndex>) -> bool {
            if index.is_none() {
                return false;
            }
            index.unwrap().char != '`'
        }
        while is_ident_content(self.current) {
            let c = self.current().unwrap().char;
            self.next_char();
            // Accept escaped character
            if c == '\\' {
                self.next_char();
            }
        }
        if self.current.is_none() {
            return Err(LexerError {
                message: IDENT_IS_MISSING_CLOSING_BACKTICK,
                index: start,
            });
        }
        let stop = self.current().unwrap();
        // Ensure that we have met our post-condition
        self.next_char();
        Ok(self.make_token(start, stop, TokenType::Ident))
    }

    /*
     * Pre-condition:
     *   `current` is the first single quote char.
     *
     * Post-condition:
     *   `current` is the next char (or EOF) after the close quote char "'".
     */
    fn parse_single_quoted_str(&mut self) -> Result<Token<'a>, LexerError> {
        let start = self.current.unwrap();
        // Accept beginning single quote
        self.next_char();
        // Accept remaining string content
        fn is_str_content(index: Option<LexerIndex>) -> bool {
            if index.is_none() {
                return false;
            }
            index.unwrap().char != '\''
        }
        while is_str_content(self.current) {
            let c = self.current().unwrap().char;
            self.next_char();
            // Accept escaped character
            if c == '\\' {
                self.next_char();
            }
        }
        if self.current.is_none() {
            return Err(LexerError {
                message: STR_IS_MISSING_CLOSING_SINGLE_QUOTE,
                index: start,
            });
        }
        let stop = self.current().unwrap();
        // Ensure that we have met our post-condition
        self.next_char();
        Ok(self.make_token(start, stop, TokenType::Str))
    }

    /*
     * Pre-condition:
     *   `index` points to the first char of a one to three character symbol.
     *   `current` is a char matching the delimiter at `index`.
     *
     * Post-condition:
     *   `current` is EOF, a separator, or the first char of the next token to parse.
     */
    fn parse_symbol(&mut self, index: usize) -> Result<Token<'a>, LexerError> {
        let first_index = self.current.unwrap();
        // Accept first char and make `current` next char or EOF
        self.next_char();
        if self.current.is_some() {
            let second_index = self.current.unwrap();
            // Is this a three char symbol?
            if first_index.char == '.' {
                if second_index.char == '.' && Self::is_some_char(self.peek_1(), '.') {
                    // Accept second char and make third char `current`
                    self.next_char();
                    let third_index = self.current.unwrap();
                    // Accept third char and make first char following the symbol `current`
                    self.next_char();
                    return Ok(self.make_token(first_index, third_index, TokenType::ThreeCharSym));
                }
            }
            // Is this a two char symbol?
            let second_chars: [u8; 2] = TWO_CHAR_SYMBOLS[index];
            if second_chars[0] != b' ' {
                if second_index.char == second_chars[0] as char {
                    // Accept second char and make `current` first char following the symbol
                    self.next_char();
                    return Ok(self.make_token(first_index, second_index, TokenType::TwoCharSym));
                } else if second_chars[1] != b' ' && second_index.char == second_chars[1] as char {
                    // Accept second char and make `current` first char following the symbol
                    self.next_char();
                    return Ok(self.make_token(first_index, second_index, TokenType::TwoCharSym));
                }
            }
        }
        // We are definitely a one char symbol
        Ok(self.make_token(first_index, first_index, TokenType::OneCharSym))
    }

    fn peek_1(&mut self) -> Option<LexerIndex> {
        if self.current_plus_1.is_none() {
            self.current_plus_1 = Self::fetch_next_char(&mut self.str_iter, &self.current);
        }
        self.current_plus_1
    }

    fn skip_whitespace(&mut self) {
        while self.current.is_some() && Self::is_whitespace(self.current.unwrap().char) {
            self.next_char();
        }
    }
}
