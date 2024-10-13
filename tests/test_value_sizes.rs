use std::collections::HashMap;

use torq_lang::klvm::value::{Comp, Scalar, ScalarOrComp, ToBeDefined};

#[test]
fn show_rust_sizes() {
    // 1 byte
    println!("bool size: {}", size_of_val(&true));
    // 4 bytes
    println!("char size: {}", size_of_val(&'x'));
    // 4 bytes
    println!("i32 size: {}", size_of_val(&1));
    // 4 bytes
    println!("f32 size: {}", size_of_val(&1f32));
    // 8 bytes
    println!("i64 size: {}", size_of_val(&1i64));
    // 8 bytes
    println!("f64 size: {}", size_of_val(&1f64));
    // 16 bytes
    let s = "Hello";
    println!("&str size: {}", size_of_val(&s));
    // 16 bytes
    let s: Option<&str> = Some("Hello");
    println!("Some(&str) size: {}", size_of_val(&s));
    // 16 bytes
    let s: Option<&str> = None;
    println!("None of &str size: {}", size_of_val(&s));
    // 24 bytes
    let s = String::from("Hello");
    println!("String size: {}", size_of_val(&s));
    // 24 bytes -- A vector consists of three 8 byte fields for a pointer, length, and capacity
    println!("vec![true] size: {}", size_of_val(&vec![true]));
    // 48 bytes
    println!(
        "HashMap (empty) size: {}",
        size_of_val(&HashMap::<i32, i32>::new())
    );
}

#[test]
fn show_torq_sizes() {
    // 16 bytes
    let v = Scalar::Bool(true);
    println!("Scalar::Bool(true) size: {}", size_of_val(&v));
    assert_eq!(16, size_of_val(&v));
    // 32 bytes
    let v = Comp::Rec(ToBeDefined::new());
    println!("Comp::Rec(ToBeDefined::new()) size: {}", size_of_val(&v));
    assert_eq!(32, size_of_val(&v));
    // 32 bytes
    let v = ScalarOrComp::Scalar(Scalar::Bool(false));
    println!(
        "ScalarOrComp::Scalar(Scalar::Bool(false)) size: {}",
        size_of_val(&v)
    );
    assert_eq!(32, size_of_val(&v));
    // 32 bytes
    let v = ScalarOrComp::Comp(Comp::Rec(ToBeDefined::new()));
    println!(
        "ScalarOrComp::Comp(Comp::Rec(ToBeDefined::new())) size: {}",
        size_of_val(&v)
    );
    assert_eq!(32, size_of_val(&v));
}
