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
fn test_double_quoted_str() {
    // Empty string
    let source = r#""""#;
    let mut lexer_iter = LexerIter::new(source);
    let n = lexer_iter.next().unwrap();
    assert_eq!(r#""""#, n.value);
    assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
    // "simple string"
    let source = r#""simple string""#;
    let mut lexer_iter = LexerIter::new(source);
    let n = lexer_iter.next().unwrap();
    assert_eq!(r#""simple string""#, n.value);
    assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
    // "A" + "B"
    let source = r#""A" + "B""#;
    let mut lexer_iter = LexerIter::new(source);
    let n = lexer_iter.next().unwrap();
    assert_eq!(r#""A""#, n.value);
    let n = lexer_iter.next().unwrap();
    assert_eq!("+", n.value);
    let n = lexer_iter.next().unwrap();
    assert_eq!(r#""B""#, n.value);
    assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
    // "A" + "B"
    let source = r#""A"
                          +
                          "B""#;
    let mut lexer_iter = LexerIter::new(source);
    let n = lexer_iter.next().unwrap();
    assert_eq!(r#""A""#, n.value);
    let n = lexer_iter.next().unwrap();
    assert_eq!("+", n.value);
    let n = lexer_iter.next().unwrap();
    assert_eq!(r#""B""#, n.value);
    assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
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
fn test_idents() {
    let source = "hello world";
    let mut lexer_iter = LexerIter::new(source);
    let n = lexer_iter.next().unwrap();
    assert_eq!("hello", n.value);
    assert_eq!(TokenType::Ident, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("world", n.value);
    assert_eq!(TokenType::Ident, n.token_type);
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
fn test_lexing() {
    let source = r#"
        /*
         * The classic factorial function in a
         * continuation-passing style.
         */
        actor Factorial() in
            func fact(x) in
                // Use continuation-passing style
                func fact_cps(n, k) in
                    if n < 2 then k
                    else fact_cps(n - 1, n * k) end
                end
                fact_cps(x, 1)
            end
            handle ask x in
                fact(x)
            end
        end"#;
    let mut lexer_iter = LexerIter::new(source);
    let comment = r#"/*
         * The classic factorial function in a
         * continuation-passing style.
         */"#;
    let n = lexer_iter.next().unwrap();
    assert_eq!(comment, n.value);
    assert_eq!(TokenType::Comment, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("actor", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("Factorial", n.value);
    assert_eq!(TokenType::Ident, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("(", n.value);
    assert_eq!(TokenType::OneCharSym, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!(")", n.value);
    assert_eq!(TokenType::OneCharSym, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("in", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("func", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("fact", n.value);
    assert_eq!(TokenType::Ident, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("(", n.value);
    assert_eq!(TokenType::OneCharSym, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("x", n.value);
    assert_eq!(TokenType::Ident, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!(")", n.value);
    assert_eq!(TokenType::OneCharSym, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("in", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("// Use continuation-passing style", n.value);
    assert_eq!(TokenType::Comment, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("func", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("fact_cps", n.value);
    assert_eq!(TokenType::Ident, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("(", n.value);
    assert_eq!(TokenType::OneCharSym, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("n", n.value);
    assert_eq!(TokenType::Ident, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!(",", n.value);
    assert_eq!(TokenType::OneCharSym, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("k", n.value);
    assert_eq!(TokenType::Ident, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!(")", n.value);
    assert_eq!(TokenType::OneCharSym, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("in", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("if", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("n", n.value);
    assert_eq!(TokenType::Ident, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("<", n.value);
    assert_eq!(TokenType::OneCharSym, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("2", n.value);
    assert_eq!(TokenType::Int, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("then", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("k", n.value);
    assert_eq!(TokenType::Ident, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("else", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("fact_cps", n.value);
    assert_eq!(TokenType::Ident, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("(", n.value);
    assert_eq!(TokenType::OneCharSym, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("n", n.value);
    assert_eq!(TokenType::Ident, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("-", n.value);
    assert_eq!(TokenType::OneCharSym, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("1", n.value);
    assert_eq!(TokenType::Int, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!(",", n.value);
    assert_eq!(TokenType::OneCharSym, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("n", n.value);
    assert_eq!(TokenType::Ident, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("*", n.value);
    assert_eq!(TokenType::OneCharSym, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("k", n.value);
    assert_eq!(TokenType::Ident, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!(")", n.value);
    assert_eq!(TokenType::OneCharSym, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("end", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("end", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("fact_cps", n.value);
    assert_eq!(TokenType::Ident, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("(", n.value);
    assert_eq!(TokenType::OneCharSym, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("x", n.value);
    assert_eq!(TokenType::Ident, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!(",", n.value);
    assert_eq!(TokenType::OneCharSym, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("1", n.value);
    assert_eq!(TokenType::Int, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!(")", n.value);
    assert_eq!(TokenType::OneCharSym, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("end", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    // `handle` and `ask` will be recognized later as a "weak" keywords
    let n = lexer_iter.next().unwrap();
    assert_eq!("handle", n.value);
    assert_eq!(TokenType::Ident, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("ask", n.value);
    assert_eq!(TokenType::Ident, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("x", n.value);
    assert_eq!(TokenType::Ident, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("in", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("fact", n.value);
    assert_eq!(TokenType::Ident, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("(", n.value);
    assert_eq!(TokenType::OneCharSym, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("x", n.value);
    assert_eq!(TokenType::Ident, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!(")", n.value);
    assert_eq!(TokenType::OneCharSym, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("end", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("end", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!(EOF_TOKEN, n);
    assert_eq!(TokenType::Eof, n.token_type);
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
fn test_keywords() {
    let source = "act actor begin break case catch continue do else elseif end eof false finally for func if import in local null of proc return self skip spawn then throw true try var when while";
    let mut lexer_iter = LexerIter::new(source);
    let n = lexer_iter.next().unwrap();
    assert_eq!("act", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("actor", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("begin", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("break", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("case", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("catch", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("continue", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("do", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("else", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("elseif", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("end", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("eof", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("false", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("finally", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("for", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("func", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("if", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("import", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("in", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("local", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("null", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("of", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("proc", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("return", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("self", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("skip", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("spawn", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("then", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("throw", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("true", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("try", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("var", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("when", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!("while", n.value);
    assert_eq!(TokenType::Keyword, n.token_type);
    let n = lexer_iter.next().unwrap();
    assert_eq!(EOF_TOKEN, n);
    assert_eq!(TokenType::Eof, n.token_type);
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
    //     "\"", "'", "`"
    // The remaining 23 delimiters are single char symbols:
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
fn test_quoted_ident() {
    // Empty identifier (will be rejected by parser)
    let source = r#"``"#;
    let mut lexer_iter = LexerIter::new(source);
    let n = lexer_iter.next().unwrap();
    assert_eq!(r#"``"#, n.value);
    assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
    // `this is an ident`
    let source = "`this is an ident`";
    let mut lexer_iter = LexerIter::new(source);
    let n = lexer_iter.next().unwrap();
    assert_eq!("`this is an ident`", n.value);
    assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
    // `A` + `B`
    let source = "`A` + `B`";
    let mut lexer_iter = LexerIter::new(source);
    let n = lexer_iter.next().unwrap();
    assert_eq!("`A`", n.value);
    let n = lexer_iter.next().unwrap();
    assert_eq!("+", n.value);
    let n = lexer_iter.next().unwrap();
    assert_eq!("`B`", n.value);
    assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
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
fn test_single_quoted_str() {
    // Empty string
    let source = r#"''"#;
    let mut lexer_iter = LexerIter::new(source);
    let n = lexer_iter.next().unwrap();
    assert_eq!(r#"''"#, n.value);
    assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
    // 'simple string'
    let source = r#"'simple string'"#;
    let mut lexer_iter = LexerIter::new(source);
    let n = lexer_iter.next().unwrap();
    assert_eq!(r#"'simple string'"#, n.value);
    assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
    // 'A' + 'B'
    let source = r#"'A' + 'B'"#;
    let mut lexer_iter = LexerIter::new(source);
    let n = lexer_iter.next().unwrap();
    assert_eq!(r#"'A'"#, n.value);
    let n = lexer_iter.next().unwrap();
    assert_eq!("+", n.value);
    let n = lexer_iter.next().unwrap();
    assert_eq!(r#"'B'"#, n.value);
    assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
    // 'A' + 'B'
    let source = r#"'A'
                          +
                          'B'"#;
    let mut lexer_iter = LexerIter::new(source);
    let n = lexer_iter.next().unwrap();
    assert_eq!(r#"'A'"#, n.value);
    let n = lexer_iter.next().unwrap();
    assert_eq!("+", n.value);
    let n = lexer_iter.next().unwrap();
    assert_eq!(r#"'B'"#, n.value);
    assert_eq!(EOF_TOKEN, lexer_iter.next().unwrap());
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
