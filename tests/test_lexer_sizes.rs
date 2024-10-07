use torq_lang::lang::lexer::{Token, TokenType};

#[test]
fn show_namaste_sizes() {
    let v = "नमस्ते";
    println!("Namaste length in bytes: {:?}", v.len());
    assert_eq!(18, v.len());
    println!("Namaste length in chars: {:?}", v.chars().count());
    assert_eq!(6, v.chars().count());
}

#[test]
fn show_three_crabs_sizes() {
    let v = "🦀🦀🦀";
    println!("Three crabs length in bytes: {:?}", v.len());
    assert_eq!(12, v.len());
    println!("Three crabs length in chars: {:?}", v.chars().count());
    assert_eq!(3, v.chars().count());
}

#[test]
fn show_token_size() {
    // 24 bytes
    let v = "";
    let token = Token {
        value: v,
        byte_index: 0,
        token_type: TokenType::Unknown,
    };
    println!("Token size: {:?}", size_of_val(&token));
    assert_eq!(24, size_of_val(&token));
}
