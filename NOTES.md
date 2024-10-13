# Notes

## Cargo commands

### Format project using `rustfmt`

```
cargo fmt
```

### Run only units tests in `lang::lexer::tests`

Redirect output to `unit_tests_output.txt`

```
cargo test --package torq_lang --lib lang::lexer::tests -- --show-output > ./test_output/unit_tests_output.txt
```

### Run only integration tests

Redirect output to `integration_tests_output.txt`

```
cargo test --package torq_lang --test '*' -- --show-output > ./test_output/integration_tests_output.txt
```

## Learning Rust

This section notes scenarios and solutions on my journey to idiomatic Rust.

### Traits to consider implementing for public types: Debug, Clone, Default, PartialOrd

Other Traits to consider: Hash, Eq, Ord, Send, Sync

A way to know if marker traits are implemented:
```rust
fn is_normal<T: Sized + Send + Sync + Unpin>() {}

fn normal_types() {
    is_normal::<NormalTypeGoesHere>();
}
```

### cannot borrow `*self` as mutable more than once at a time

> See: https://users.rust-lang.org/t/workaround-for-cannot-borrow-self-as-mutable-more-than-once-at-a-time/16286/2
    
> To be clear, the issue here is not with mutating an object multiple times in a sequence. For example, pushing to a Vec multiple times in a row is fine. Where Rust draws the line is when you start mutating an object while holding a reference to its internals.
>
> When you do this, there is a fair chance that the state to which you are holding a reference will be modified, in a way that can invalidate the reference. For example, pushing to a Vec while you hold a reference to it (such as an iterator) can lead to a reallocation, in which case your reference would become dangling. Depending on which language you are using, this could be an unpredictable runtime error (Java, C#) or security-critical undefined behaviour (C, C++). Rust cannot stand for either, so it prevents them by design.

Example:
~~~
use std::str::Chars;

#[derive(Debug)]
struct Token<'a> {
    value: &'a str,
}

struct BadLexer<'a> {
    source: &'a str,
    chars_iter: Chars<'a>,
}

impl<'a> BadLexer<'a> {
    fn next(&mut self) -> Token {
        if let Some(t) = self.try_parse_sym_one() {
            t
        } else if let Some(t) = self.try_parse_sym_two() {
            t
        } else {
            Token { value: "unknown" }
        }
    }

    fn try_parse_sym_one(&mut self) -> Option<Token> {
        None
    }

    fn try_parse_sym_two(&mut self) -> Option<Token> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_iter() {
        let source = "1".to_string();
        let mut lexer = BadLexer {
            source: &source,
            chars_iter: source.chars(),
        };
        let t = lexer.next();
        println!("{:?}", t);
    }
}
~~~

The problem:
~~~
error[E0499]: cannot borrow `*self` as mutable more than once at a time
  --> src/lang/example_error_01.rs:17:33
   |
14 |     fn next(&mut self) -> Token {
   |             - let's call the lifetime of this reference `'1`
15 |         if let Some(t) = self.try_parse_sym_one() {
   |                          ---- first mutable borrow occurs here
16 |             t
   |             - returning this value requires that `*self` is borrowed for `'1`
17 |         } else if let Some(t) = self.try_parse_sym_two() {
   |                                 ^^^^ second mutable borrow occurs here
~~~

A solution:
~~~
use std::str::Chars;

#[derive(Debug)]
struct Token<'a> {
    value: &'a str,
}

struct GoodLexer<'a> {
    source: &'a str,
}

impl<'a> GoodLexer<'a> {
    fn next(&self, chars_iter: &mut Chars) -> Token {
        if let Some(t) = self.try_parse_sym_one() {
            t
        } else if let Some(t) = self.try_parse_sym_two() {
            t
        } else {
            Token { value: "unknown" }
        }
    }

    fn try_parse_sym_one(&self) -> Option<Token> {
        None
    }

    fn try_parse_sym_two(&self) -> Option<Token> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_iter() {
        let source = "1".to_string();
        let chars_iter = &mut source.chars();
        let lexer = GoodLexer {
            source: &source,
        };
        let t = lexer.next(chars_iter);
        println!("{:?}", t);
    }
}
~~~