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
use std::string::ToString;

/// Character position is the UTF-8 character index and not the byte index. Because UTF-8
/// character encodings vary between 1 and 4 bytes, character positions in a source string are not
/// deterministic.
#[derive(Debug)]
pub struct Token<'a> {
    pub value: &'a str,
    pub byte_index: i32,
    pub token_type: TokenType,
}

const EOF_TOKEN: Token = Token {
    value: "EOF",
    byte_index: -1,
    token_type: TokenType::Eof,
};

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
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct LexerIndex {
    char: char,
    char_index: i32,
    byte_index: i32,
}

struct LexerIter<'a> {
    source: &'a str,
    str_iter: Chars<'a>,
    current: Option<LexerIndex>,
    peek_1: Option<LexerIndex>,
    peek_2: Option<LexerIndex>,
}

const INVALID_FLOATING_POINT_NUMBER: &str = "Invalid floating point number";

#[derive(Debug, PartialEq)]
pub struct LexerError {
    message: &'static str,
    index: LexerIndex,
}

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

    fn is_digit(c: char) -> bool {
        c >= '0' && c <= '9'
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

    fn new(source: &'a str) -> Self {
        let mut str_iter = source.chars();
        let current = Self::fetch_next_char(&mut str_iter, &None);
        LexerIter {
            source: source,
            str_iter: str_iter,
            current: current,
            peek_1: None,
            peek_2: None,
        }
    }

    fn next(&mut self) -> Result<Token, LexerError> {
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
        panic!("Needs impl - Unknown token")
    }

    fn next_char(&mut self) {
        if self.peek_1.is_some() {
            self.current = self.peek_1;
            if self.peek_2.is_some() {
                self.peek_1 = self.peek_2;
                self.peek_2 = None;
            } else {
                self.peek_1 = None;
            }
        } else {
            self.current = Self::fetch_next_char(&mut self.str_iter, &self.current);
        }
    }

    /*
     * Invariants:
     *   `start` is the first digit of a number expression.
     *   `current` is the first digit of the fractional part.
     */
    fn parse_fractional_part(&mut self, start: LexerIndex) -> Result<Token<'a>, LexerError> {
        /*
        // Accept first digit [0-9] and continue accepting digits until a non-digit is found
        while (++stop < source.length()) {
            if (!isDigit(source.charAt(stop))) {
                break;
            }
        }
        if (stop == source.length() || isWhiteSpaceAt(stop) || isDelimiterAt(stop)) {
            charPos = stop;
            return new LexerToken(LexerTokenType.FLT_TOKEN, source, start, stop);
        }
        char c = source.charAt(stop);
        if (c == 'e' || c == 'E') {
            return parseExponentSuffix(start, stop);
        }
        LexerTokenType tokenType;
        stop++;
        if (c == 'f' || c == 'F') {
            tokenType = LexerTokenType.FLT_TOKEN;
        } else if (c == 'd' || c == 'D') {
            tokenType = LexerTokenType.FLT_TOKEN;
        } else if (c == 'm' || c == 'M') {
            tokenType = LexerTokenType.DEC_TOKEN;
        } else {
            LexerToken invalidToken = new LexerToken(LexerTokenType.FLT_TOKEN, source, start, stop);
            throw new LexerError(invalidToken, "Floating point suffix must be one of [fFdDmM]");
        }
        if (stop == source.length() || isWhiteSpaceAt(stop) || isDelimiterAt(stop)) {
            charPos = stop;
            return new LexerToken(tokenType, source, start, stop);
        }
        // We have a nonsensical continuation of what looked like a floating point literal
        charPos = stop + 1; // accept the nonsensical char as part of the value
        LexerToken invalidToken = new LexerToken(tokenType, source, start, charPos);
        if (tokenType == LexerTokenType.DEC_TOKEN) {
            throw new LexerError(invalidToken, INVALID_DECIMAL_NUMBER);
        }
        throw new LexerError(invalidToken, INVALID_FLOATING_POINT_NUMBER);
         */
        while Self::is_some_digit(self.peek_1) {
            self.next_char();
        }
        // Parse case:
        //    Exponent suffix
        //    One of [fFdDmM] as suffix
        //    A nonsensical continuation of a valid suffix
        panic!("Needs impl - parse fractional part")
    }

    /*
     * Invariants:
     *   `start` is the same as `current`.
     *   `current` is the first (zero) digit of a hex integer.
     */
    fn parse_hex_int(&mut self, start: LexerIndex) -> Result<Token<'a>, LexerError> {
        panic!("Needs impl - parse hex integer")
    }

    /*
     * Invariants:
     *   `start` is the first digit of an integer expression.
     *   `current` is the first char after the integer expression.
     */
    fn parse_int_suffix(&mut self, start: LexerIndex) -> Result<Token<'a>, LexerError> {
        /*
        if (stop == source.length()) {
            // End of File
            charPos = stop;
            return new LexerToken(LexerTokenType.INT_TOKEN, source, start, stop);
        }
        char c = source.charAt(stop);
        LexerTokenType tokenType;
        if (c == 'l' || c == 'L') {
            stop++;
            tokenType = LexerTokenType.INT_TOKEN;
        } else if (c == 'm' || c == 'M') {
            stop++;
            tokenType = LexerTokenType.DEC_TOKEN;
        } else {
            tokenType = LexerTokenType.INT_TOKEN;
        }
        if (stop == source.length() || isWhiteSpaceAt(stop) || isDelimiterAt(stop)) {
            charPos = stop;
            return new LexerToken(tokenType, source, start, charPos);
        } else {
            // We have a nonsensical continuation of what looked like an integer literal
            charPos = stop + 1;
            LexerToken invalidToken = new LexerToken(LexerTokenType.INT_TOKEN, source, start, charPos);
            throw new LexerError(invalidToken, INVALID_INTEGER);
        }
         */
        let peek_1 = self.peek_1();
        if peek_1.is_none() {
            let current = self.current.unwrap();
            let token_start = start.byte_index as usize;
            let token_stop = current.byte_index as usize + 1;
            return Ok(Token {
                value: &self.source[token_start..token_stop],
                byte_index: start.byte_index,
                token_type: TokenType::Int,
            });
        }
        panic!("Needs impl - parse int suffix")
    }

    /*
     * Invariants:
     *   `start` is the same as `current`.
     *   `current` is the first digit of a non-hex number.
     */
    fn parse_non_hex_num(&mut self, start: LexerIndex) -> Result<Token<'a>, LexerError> {
        while Self::is_some_digit(self.peek_1) {
            self.next_char();
        }
        if Self::is_some_char(self.peek_1(), '.') {
            self.next_char(); // accept the period
            self.next_char(); // load next char
            if self.current.is_none() || !Self::is_digit(self.current.unwrap().char) {
                return Err(LexerError {
                    message: INVALID_FLOATING_POINT_NUMBER,
                    index: start,
                });
            }
            return self.parse_fractional_part(start);
        }
        self.parse_int_suffix(start)
    }

    /*
     * Invariants:
     *   `current` is the first digit of a number.
     */
    fn parse_num(&mut self) -> Result<Token<'a>, LexerError> {
        let start = self.current.unwrap();
        self.next_char(); // Accept first digit
        if self.current.is_none() {
            // Return a single-digit token
            let token_start = start.byte_index as usize;
            return Ok(Token {
                value: &self.source[token_start..token_start + 1],
                byte_index: token_start as i32,
                token_type: TokenType::Int,
            });
        }
        let current = self.current.unwrap();
        if current.char == '0' {
            // We know we have at least one more char
            let peek = self.peek_1().unwrap();
            if peek.char == 'x' || peek.char == 'X' {
                return self.parse_hex_int(start);
            }
        }
        self.parse_non_hex_num(start)
    }

    fn parse_single_quoted_str(&mut self) -> Result<Token<'a>, LexerError> {
        panic!("Needs impl - parse single quoted string")
    }

    fn peek_1(&mut self) -> Option<LexerIndex> {
        if self.peek_1.is_none() {
            self.peek_1 = Self::fetch_next_char(&mut self.str_iter, &self.current);
        }
        self.peek_1
    }

    fn peek_1_is_digit(&mut self) -> bool {
        let p1 = self.peek_1();
        if p1.is_some() {
            Self::is_digit(p1.unwrap().char)
        } else {
            false
        }
    }

    fn peek_2(&mut self) -> Option<LexerIndex> {
        if self.peek_1.is_none() {
            self.peek_1 = Self::fetch_next_char(&mut self.str_iter, &self.current);
            self.peek_2 = Self::fetch_next_char(&mut self.str_iter, &self.current);
        } else if self.peek_2.is_none() {
            self.peek_2 = Self::fetch_next_char(&mut self.str_iter, &self.current);
        }
        self.peek_2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter_chars() {
        let crab = '🦀';
        assert_eq!(4, crab.len_utf8());
        let source = "1+🦀+2";
        let mut lexer_iter = LexerIter::new(source);
        assert_eq!(true, lexer_iter.current.is_some());
        assert_eq!('1', lexer_iter.current.unwrap().char);
        assert_eq!(0, lexer_iter.current.unwrap().byte_index);
        assert_eq!(0, lexer_iter.current.unwrap().char_index);
        lexer_iter.next_char();
        assert_eq!('+', lexer_iter.current.unwrap().char);
        assert_eq!(1, lexer_iter.current.unwrap().byte_index);
        assert_eq!(1, lexer_iter.current.unwrap().char_index);
        lexer_iter.next_char();
        assert_eq!('🦀', lexer_iter.current.unwrap().char);
        assert_eq!(2, lexer_iter.current.unwrap().byte_index);
        assert_eq!(2, lexer_iter.current.unwrap().char_index);
        lexer_iter.next_char();
        assert_eq!('+', lexer_iter.current.unwrap().char);
        assert_eq!(6, lexer_iter.current.unwrap().byte_index);
        assert_eq!(3, lexer_iter.current.unwrap().char_index);
        lexer_iter.next_char();
        assert_eq!('2', lexer_iter.current.unwrap().char);
        assert_eq!(7, lexer_iter.current.unwrap().byte_index);
        assert_eq!(4, lexer_iter.current.unwrap().char_index);
        lexer_iter.next_char();
        assert_eq!(true, lexer_iter.current.is_none());
    }

    #[test]
    fn test_multi_digit_int() {
        let source = "23";
        let mut lexer_iter = LexerIter::new(source);
        let n = lexer_iter.next().unwrap();
        assert_eq!("23", n.value);
        assert_eq!(0, n.byte_index);
        assert_eq!(TokenType::Int, n.token_type);
    }

    // #[test]
    // fn test_real() {
    //     let source = "23.1";
    //     let mut lexer_iter = LexerIter::new(source);
    //     let n = lexer_iter.next().unwrap();
    //     assert_eq!("23.1", n.value);
    //     assert_eq!(0, n.byte_index);
    //     assert_eq!(TokenType::Flt, n.token_type);
    // }

    #[test]
    fn test_real_with_trailing_period_error() {
        let source = "23.";
        let mut lexer_iter = LexerIter::new(source);
        let r = lexer_iter.next();
        if r.is_ok() {
            panic!("Error expected");
        }
        assert_eq!(INVALID_FLOATING_POINT_NUMBER, r.err().unwrap().message);
    }

    #[test]
    fn test_single_digit_int() {
        let source = "1";
        let mut lexer_iter = LexerIter::new(source);
        let n = lexer_iter.next().unwrap();
        assert_eq!("1", n.value);
        assert_eq!(0, n.byte_index);
        assert_eq!(TokenType::Int, n.token_type);
    }
}
