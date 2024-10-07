/*
 * Copyright (c) 2024 Torqware LLC. All rights reserved.
 *
 * You should have received a copy of the Torq Lang License v1.0 along with this program.
 * If not, see http://torq-lang.github.io/licensing/torq-lang-license-v1_0.
 */

pub enum Scalar {
    Bool(bool),
    Char(char),
    Flt32(f32),
    Flt64(f64),
    Int32(i32),
    Int64(i64),
}

pub enum Comp {
    Obj(ToBeDefined),
    Rec(ToBeDefined),
    Tuple(ToBeDefined),
}

pub enum ScalarOrComp {
    Scalar(Scalar),
    Comp(Comp),
}

pub struct ToBeDefined {
    value: Vec<String>,
}

impl ToBeDefined {
    pub fn new() -> ToBeDefined {
        ToBeDefined { value: vec![] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
}
