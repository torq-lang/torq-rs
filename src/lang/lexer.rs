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

/// Character position is the UTF-8 character index and not the byte index. Because UTF-8
/// character encodings vary between 1 and 4 bytes, character positions in a source string are not
/// deterministic.
#[derive(Debug, PartialEq)]
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
    current_plus_1: Option<LexerIndex>,
    current_plus_2: Option<LexerIndex>,
}

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

const COMMENT_IS_MISSING_CLOSING_SEQUENCE: &str = "Comment is missing closing sequence '*/'";
const FLOATING_POINT_SUFFIX_MUST_BE_ONE_OF: &str = "Floating point suffix must be one of [fFdDmM]";
const INTEGER_SUFFIX_MUST_BE_ONE_OF: &str = "Integer suffix must be one of [lLmM]";
const INVALID_DECIMAL_NUMBER: &str = "Invalid decimal number";
const INVALID_FLOATING_POINT_NUMBER: &str = "Invalid floating point number";
const INVALID_HEXADECIMAL_NUMBER: &str = "Invalid hexadecimal number";
const INVALID_INTEGER_NUMBER: &str = "Invalid integer";

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

    fn is_block_comment_content(index1: Option<LexerIndex>, index2: Option<LexerIndex>,) -> bool {
        if index1.is_none() || index2.is_none() {
            return false;
        }
        index1.unwrap().char != '*' || index2.unwrap().char != '/'
    }

    fn is_delimiter(c: char) -> bool {
        Self::index_of_delimiter(c) > -1
    }

    fn is_digit(c: char) -> bool {
        c >= '0' && c <= '9'
    }

    fn is_eof_or_separator(index: Option<LexerIndex>) -> bool {
        index.is_none() || Self::is_separator(index.unwrap().char)
    }

    fn is_hex_digit(c: char) -> bool {
        c >= '0' && c <= '9' || c >= 'a' && c <= 'f' || c >= 'A' && c <= 'F'
    }

    fn is_line_comment_content(index: Option<LexerIndex>) -> bool {
        if index.is_none() {
            return false;
        }
        index.unwrap().char != '\n'
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

    fn make_invalid_number_err(
        &self,
        start: LexerIndex,
        token_type: TokenType,
    ) -> LexerError {
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

    fn make_token(
        &self,
        start: LexerIndex,
        stop: LexerIndex,
        token_type: TokenType,
    ) -> Token<'a> {
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
            current_plus_2: None,
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
        self.parse_keyword_or_ident()
    }

    fn next_char(&mut self) {
        if self.current_plus_1.is_some() {
            self.current = self.current_plus_1;
            if self.current_plus_2.is_some() {
                self.current_plus_1 = self.current_plus_2;
                self.current_plus_2 = None;
            } else {
                self.current_plus_1 = None;
            }
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
        while Self::is_block_comment_content(self.current, self.peek_1()) {
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
        panic!("Needs impl - parse double quoted string")
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
     * Pre-condition:
     *   'start' is the "0" char of the hex prefix.
     *   `current` is the "x" or "X" of the hex prefix.
     *
     * Post-condition:
     *   `current` is EOF or a separator (whitespace or delimiter).
     */
    fn parse_hex_int(&mut self, start: LexerIndex) -> Result<Token<'a>, LexerError> {
        if self.peek_1().is_none() || self.peek_2().is_none() {
            return Err(LexerError {
                message: INVALID_HEXADECIMAL_NUMBER,
                index: start,
            });
        }
        let p1 = self.current_plus_1.unwrap();
        let p2 = self.current_plus_2.unwrap();
        if !Self::is_hex_digit(p1.char) || !Self::is_hex_digit(p2.char) {
            return Err(LexerError {
                message: INVALID_HEXADECIMAL_NUMBER,
                index: start,
            });
        }
        // Make `p1` `current`
        self.next_char();
        // Make `p2` `current`
        self.next_char();
        // Accept `p2` and make the next char `current`
        self.next_char();
        // Ensure that our post-condition is an EOF or separator
        if !Self::is_eof_or_separator(self.current) {
            return Err(LexerError {
                message: INVALID_FLOATING_POINT_NUMBER,
                index: start,
            });
        }
        Ok(self.make_token(start, p2, TokenType::Int))
    }

    /*
     * Pre-condition:
     *   `current` is the first char of a keyword or ident.
     *
     * Post-condition:
     *   `current` is EOF or separator (whitespace or delimiter)
     */
    fn parse_keyword_or_ident(&mut self) -> Result<Token<'a>, LexerError> {
        panic!("Needs impl - parse keyword or ident")
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
        while Self::is_line_comment_content(self.peek_1()) {
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
                return self.parse_hex_int(start);
            }
        }
        while Self::is_some_digit(self.peek_1()) {
            self.next_char(); // Make the next digit `current`
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
            return self.parse_fractional_part(start);
        }
        if Self::is_eof_or_separator(self.peek_1()) {
            let token_start = start.byte_index as usize;
            let token_stop = self.current.unwrap().byte_index as usize + 1;
            // Accept `current` if next char is a separator
            if self.current_plus_1.is_some() {
                self.next_char();
            }
            return Ok(Token {
                value: &self.source[token_start..token_stop],
                byte_index: start.byte_index,
                token_type: TokenType::Int,
            });
        }
        // We have a `current_plus_1` because we peeked successfully above
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
            Err(self.make_invalid_number_err(start, token_type))
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
        panic!("Needs impl - parse quoted ident")
    }

    /*
     * Pre-condition:
     *   `current` is the first single quote char.
     *
     * Post-condition:
     *   `current` is the next char (or EOF) after the close quote char "'".
     */
    fn parse_single_quoted_str(&mut self) -> Result<Token<'a>, LexerError> {
        panic!("Needs impl - parse single quoted string")
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
            if second_index.char == second_chars[0] as char {
                // Accept second char and make `current` first char following the symbol
                self.next_char();
                return Ok(self.make_token(first_index, second_index, TokenType::TwoCharSym));
            } else if second_index.char == second_chars[1] as char {
                // Accept second char and make `current` first char following the symbol
                self.next_char();
                return Ok(self.make_token(first_index, second_index, TokenType::TwoCharSym));
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

    fn peek_2(&mut self) -> Option<LexerIndex> {
        if self.current_plus_1.is_none() {
            self.current_plus_1 = Self::fetch_next_char(&mut self.str_iter, &self.current);
            self.current_plus_2 = Self::fetch_next_char(&mut self.str_iter, &self.current_plus_1);
        } else if self.current_plus_2.is_none() {
            self.current_plus_2 = Self::fetch_next_char(&mut self.str_iter, &self.current_plus_1);
        }
        self.current_plus_2
    }

    fn skip_whitespace(&mut self) {
        while self.current.is_some() && Self::is_whitespace(self.current.unwrap().char) {
            self.next_char();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_parse_eq(source: &str) {
        let mut lexer_iter = LexerIter::new(source);
        let next_token = lexer_iter.next();
        assert_eq!(true, next_token.is_ok());
        assert_eq!(source, next_token.unwrap().value);
        let next_token = lexer_iter.next();
        assert_eq!(true, next_token.is_ok());
        assert_eq!(EOF_TOKEN, next_token.unwrap());
    }

    #[test]
    fn test_adjacent_one_char_symbols() {
        let source = "!#%()*+,-./:;<>@[]{}~=";
        let mut lexer_iter = LexerIter::new(source);
        assert_eq!("!", lexer_iter.next().unwrap().value);
        assert_eq!("#", lexer_iter.next().unwrap().value);
        assert_eq!("%", lexer_iter.next().unwrap().value);
        assert_eq!("(", lexer_iter.next().unwrap().value);
        assert_eq!(")", lexer_iter.next().unwrap().value);
        assert_eq!("*", lexer_iter.next().unwrap().value);
        assert_eq!("+", lexer_iter.next().unwrap().value);
        assert_eq!(",", lexer_iter.next().unwrap().value);
        assert_eq!("-", lexer_iter.next().unwrap().value);
        assert_eq!(".", lexer_iter.next().unwrap().value);
        assert_eq!("/", lexer_iter.next().unwrap().value);
        assert_eq!(":", lexer_iter.next().unwrap().value);
        assert_eq!(";", lexer_iter.next().unwrap().value);
        assert_eq!("<", lexer_iter.next().unwrap().value);
        assert_eq!(">", lexer_iter.next().unwrap().value);
        assert_eq!("@", lexer_iter.next().unwrap().value);
        assert_eq!("[", lexer_iter.next().unwrap().value);
        assert_eq!("]", lexer_iter.next().unwrap().value);
        assert_eq!("{", lexer_iter.next().unwrap().value);
        assert_eq!("}", lexer_iter.next().unwrap().value);
        assert_eq!("~", lexer_iter.next().unwrap().value);
        assert_eq!("=", lexer_iter.next().unwrap().value);
        assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
    }

    #[test]
    fn test_adjacent_two_char_symbols() {
        let source = "==!=<=>=&&->||:=::<:>:";
        let mut lexer_iter = LexerIter::new(source);
        // There are 11 two char symbols:
        // [b'=', b' '], //  0: !=
        // [b'&', b' '], //  4: &&
        // [b'>', b' '], // 11: ->
        // [b'=', b':'], // 14: :=, ::
        // [b'=', b':'], // 16: <=, <:
        // [b'=', b':'], // 17: ==
        // [b'=', b':'], // 18: >=, >:
        // [b'|', b' '], // 24: ||
        assert_eq!("==", lexer_iter.next().unwrap().value);
        assert_eq!("!=", lexer_iter.next().unwrap().value);
        assert_eq!("<=", lexer_iter.next().unwrap().value);
        assert_eq!(">=", lexer_iter.next().unwrap().value);
        assert_eq!("&&", lexer_iter.next().unwrap().value);
        assert_eq!("->", lexer_iter.next().unwrap().value);
        assert_eq!("||", lexer_iter.next().unwrap().value);
        assert_eq!(":=", lexer_iter.next().unwrap().value);
        assert_eq!("::", lexer_iter.next().unwrap().value);
        assert_eq!("<:", lexer_iter.next().unwrap().value);
        assert_eq!(">:", lexer_iter.next().unwrap().value);
        assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
    }

    #[test]
    fn test_block_comment() {
        let source = "/**/";
        let mut lexer_iter = LexerIter::new(source);
        assert_eq!("/**/", lexer_iter.next().unwrap().value);
        assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
        let source = "/*\n*/";
        let mut lexer_iter = LexerIter::new(source);
        assert_eq!("/*\n*/", lexer_iter.next().unwrap().value);
        assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
        let source = "/*a*/";
        let mut lexer_iter = LexerIter::new(source);
        assert_eq!("/*a*/", lexer_iter.next().unwrap().value);
        assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
        let source = "/*\na*/";
        let mut lexer_iter = LexerIter::new(source);
        assert_eq!("/*\na*/", lexer_iter.next().unwrap().value);
        assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
        let source = "/*a\n*/";
        let mut lexer_iter = LexerIter::new(source);
        assert_eq!("/*a\n*/", lexer_iter.next().unwrap().value);
        assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
        let source = "1/*a\n*/";
        let mut lexer_iter = LexerIter::new(source);
        assert_eq!("1", lexer_iter.next().unwrap().value);
        assert_eq!("/*a\n*/", lexer_iter.next().unwrap().value);
        assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
    }

    #[test]
    fn test_delimiters() {
        // All 27 delimiters:
        assert_eq!(true, LexerIter::is_delimiter('!'));
        assert_eq!(true, LexerIter::is_delimiter('"'));
        assert_eq!(true, LexerIter::is_delimiter('#'));
        assert_eq!(true, LexerIter::is_delimiter('%'));
        assert_eq!(true, LexerIter::is_delimiter('&'));
        assert_eq!(true, LexerIter::is_delimiter('\''));
        assert_eq!(true, LexerIter::is_delimiter('('));
        assert_eq!(true, LexerIter::is_delimiter(')'));
        assert_eq!(true, LexerIter::is_delimiter('*'));
        assert_eq!(true, LexerIter::is_delimiter('+'));
        assert_eq!(true, LexerIter::is_delimiter(','));
        assert_eq!(true, LexerIter::is_delimiter('-'));
        assert_eq!(true, LexerIter::is_delimiter('.'));
        assert_eq!(true, LexerIter::is_delimiter('/'));
        assert_eq!(true, LexerIter::is_delimiter(':'));
        assert_eq!(true, LexerIter::is_delimiter(';'));
        assert_eq!(true, LexerIter::is_delimiter('<'));
        assert_eq!(true, LexerIter::is_delimiter('='));
        assert_eq!(true, LexerIter::is_delimiter('>'));
        assert_eq!(true, LexerIter::is_delimiter('@'));
        assert_eq!(true, LexerIter::is_delimiter('['));
        assert_eq!(true, LexerIter::is_delimiter(']'));
        assert_eq!(true, LexerIter::is_delimiter('`'));
        assert_eq!(true, LexerIter::is_delimiter('{'));
        assert_eq!(true, LexerIter::is_delimiter('|'));
        assert_eq!(true, LexerIter::is_delimiter('}'));
        assert_eq!(true, LexerIter::is_delimiter('~'));
        // Not a delimiter
        assert_eq!(false, LexerIter::is_delimiter('?'));
        assert_eq!(false, LexerIter::is_delimiter('^'));
    }

    #[test]
    fn test_hex_int_with_small_x() {
        let source = "0xAE";
        let mut lexer_iter = LexerIter::new(source);
        let n = lexer_iter.next().unwrap();
        assert_eq!("0xAE", n.value);
        assert_eq!(0, n.byte_index);
        assert_eq!(TokenType::Int, n.token_type);
    }

    #[test]
    fn test_hex_int_with_large_x() {
        let source = "0Xae";
        let mut lexer_iter = LexerIter::new(source);
        let n = lexer_iter.next().unwrap();
        assert_eq!("0Xae", n.value);
        assert_eq!(0, n.byte_index);
        assert_eq!(TokenType::Int, n.token_type);
    }

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
    fn test_line_comment() {
        let source = "//";
        let mut lexer_iter = LexerIter::new(source);
        assert_eq!("//", lexer_iter.next().unwrap().value);
        assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
        let source = "// ";
        let mut lexer_iter = LexerIter::new(source);
        assert_eq!("// ", lexer_iter.next().unwrap().value);
        assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
        let source = "//a";
        let mut lexer_iter = LexerIter::new(source);
        assert_eq!("//a", lexer_iter.next().unwrap().value);
        assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
        let source = "//\n//";
        let mut lexer_iter = LexerIter::new(source);
        assert_eq!("//", lexer_iter.next().unwrap().value);
        assert_eq!("//", lexer_iter.next().unwrap().value);
        assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
        let source = "// \n//";
        let mut lexer_iter = LexerIter::new(source);
        assert_eq!("// ", lexer_iter.next().unwrap().value);
        assert_eq!("//", lexer_iter.next().unwrap().value);
        assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
        let source = "//a";
        let mut lexer_iter = LexerIter::new(source);
        assert_eq!("//a", lexer_iter.next().unwrap().value);
        assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
        let source = "//a\n//";
        let mut lexer_iter = LexerIter::new(source);
        assert_eq!("//a", lexer_iter.next().unwrap().value);
        assert_eq!("//", lexer_iter.next().unwrap().value);
        assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
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

    #[test]
    fn test_one_char_symbols() {
        // Note that the following symbols will fail because they are used for quoting:
        //     assert_single_token_parse("\"");
        //     assert_single_token_parse("'");
        //     assert_single_token_parse("`");
        // Remaining 23 delimiters are single char symbols:
        assert_parse_eq("!");
        assert_parse_eq("#");
        assert_parse_eq("%");
        assert_parse_eq("(");
        assert_parse_eq(")");
        assert_parse_eq("*");
        assert_parse_eq("+");
        assert_parse_eq(",");
        assert_parse_eq("-");
        assert_parse_eq(".");
        assert_parse_eq("/");
        assert_parse_eq(":");
        assert_parse_eq(";");
        assert_parse_eq("<");
        assert_parse_eq("=");
        assert_parse_eq(">");
        assert_parse_eq("@");
        assert_parse_eq("[");
        assert_parse_eq("]");
        assert_parse_eq("{");
        assert_parse_eq("|");
        assert_parse_eq("}");
        assert_parse_eq("~");
    }

    #[test]
    fn test_real() {
        let source = "23.1";
        let mut lexer_iter = LexerIter::new(source);
        let n = lexer_iter.next().unwrap();
        assert_eq!("23.1", n.value);
        assert_eq!(0, n.byte_index);
        assert_eq!(TokenType::Flt, n.token_type);
    }

    #[test]
    fn test_real_division() {
        let source = "23.1/13.2";
        let mut lexer_iter = LexerIter::new(source);
        let n = lexer_iter.next().unwrap();
        assert_eq!("23.1", n.value);
        assert_eq!(0, n.byte_index);
        assert_eq!(TokenType::Flt, n.token_type);
        let n = lexer_iter.next().unwrap();
        assert_eq!("/", n.value);
        assert_eq!(4, n.byte_index);
        assert_eq!(TokenType::OneCharSym, n.token_type);
    }

    #[test]
    fn test_real_division_with_exponents() {
        let source = "231.0e-1/132.0e-1";
        let mut lexer_iter = LexerIter::new(source);
        let n = lexer_iter.next().unwrap();
        assert_eq!("231.0e-1", n.value);
        assert_eq!(0, n.byte_index);
        assert_eq!(TokenType::Flt, n.token_type);
        let n = lexer_iter.next().unwrap();
        assert_eq!("/", n.value);
        assert_eq!(8, n.byte_index);
        assert_eq!(TokenType::OneCharSym, n.token_type);
        let n = lexer_iter.next().unwrap();
        assert_eq!("132.0e-1", n.value);
        assert_eq!(9, n.byte_index);
        assert_eq!(TokenType::Flt, n.token_type);
    }

    #[test]
    fn test_real_with_one_digit_exponent() {
        let source = "1.0e-1";
        let mut lexer_iter = LexerIter::new(source);
        let n = lexer_iter.next().unwrap();
        assert_eq!("1.0e-1", n.value);
        assert_eq!(0, n.byte_index);
        assert_eq!(TokenType::Flt, n.token_type);
    }

    #[test]
    fn test_real_with_two_digit_exponent() {
        let source = "1.0e+12";
        let mut lexer_iter = LexerIter::new(source);
        let n = lexer_iter.next().unwrap();
        assert_eq!("1.0e+12", n.value);
        assert_eq!(0, n.byte_index);
        assert_eq!(TokenType::Flt, n.token_type);
    }

    #[test]
    fn test_real_with_invalid_suffix_error() {
        let source = "23.0x";
        let mut lexer_iter = LexerIter::new(source);
        let r = lexer_iter.next();
        if r.is_ok() {
            panic!("Error expected");
        }
        assert_eq!(
            FLOATING_POINT_SUFFIX_MUST_BE_ONE_OF,
            r.err().unwrap().message
        );
    }

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
    fn test_real_with_trailing_suffix_error() {
        let source = "23.f";
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

    #[test]
    fn test_three_char_symbols() {
        assert_parse_eq("...");
    }

    #[test]
    fn test_three_equals() {
        let source = "===";
        let mut lexer_iter = LexerIter::new(source);
        assert_eq!("==", lexer_iter.next().unwrap().value);
        assert_eq!("=", lexer_iter.next().unwrap().value);
        assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
    }

    #[test]
    fn test_two_char_symbols() {
        // There are 11 two char symbols:
        // [b'=', b' '], //  0: !=
        // [b'&', b' '], //  4: &&
        // [b'>', b' '], // 11: ->
        // [b'=', b':'], // 14: :=, ::
        // [b'=', b':'], // 16: <=, <:
        // [b'=', b':'], // 17: ==
        // [b'=', b':'], // 18: >=, >:
        // [b'|', b' '], // 24: ||
        assert_parse_eq("!=");
        assert_parse_eq("&&");
        assert_parse_eq("->");
        assert_parse_eq(":=");
        assert_parse_eq("::");
        assert_parse_eq("<=");
        assert_parse_eq("<:");
        assert_parse_eq("==");
        assert_parse_eq(">=");
        assert_parse_eq(">:");
        assert_parse_eq("||");
    }

    #[test]
    fn test_two_periods() {
        let source = "..";
        let mut lexer_iter = LexerIter::new(source);
        assert_eq!(".", lexer_iter.next().unwrap().value);
        assert_eq!(".", lexer_iter.next().unwrap().value);
        assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
    }

    #[test]
    fn test_two_subtracts() {
        let source = "--";
        let mut lexer_iter = LexerIter::new(source);
        assert_eq!("-", lexer_iter.next().unwrap().value);
        assert_eq!("-", lexer_iter.next().unwrap().value);
        assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
    }

    #[test]
    fn test_four_periods() {
        let source = "....";
        let mut lexer_iter = LexerIter::new(source);
        assert_eq!("...", lexer_iter.next().unwrap().value);
        assert_eq!(".", lexer_iter.next().unwrap().value);
        assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
    }

    #[test]
    fn test_five_periods() {
        let source = ".....";
        let mut lexer_iter = LexerIter::new(source);
        assert_eq!("...", lexer_iter.next().unwrap().value);
        assert_eq!(".", lexer_iter.next().unwrap().value);
        assert_eq!(".", lexer_iter.next().unwrap().value);
        assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
    }
}
