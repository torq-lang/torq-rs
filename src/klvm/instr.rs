/*
 * Copyright (c) 2024 Torqware LLC. All rights reserved.
 *
 * You should have received a copy of the Torq Lang License v1.0 along with this program.
 * If not, see http://torq-lang.github.io/licensing/torq-lang-license-v1_0.
 */

trait Instr {
    fn compute();
}

struct AddInt32Ident<'a> {
    left: i32,
    right: &'a str,
    target: &'a str,
}

impl<'a> AddInt32Ident<'a> {
    fn compute() {}
}
