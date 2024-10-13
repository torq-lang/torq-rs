/*
 * Copyright (c) 2024 Torqware LLC. All rights reserved.
 *
 * You should have received a copy of the Torq Lang License v1.0 along with this program.
 * If not, see http://torq-lang.github.io/licensing/torq-lang-license-v1_0.
 */

use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn linear_search<T: PartialEq>(arr: &[T], target: &T) -> Option<usize> {
    for (index, item) in arr.iter().enumerate() {
        if item == target {
            return Some(index);
        }
    }
    None
}

pub fn binary_search<T: Ord>(arr: &[T], target: &T) -> Option<usize> {
    let mut left = 0;
    let mut right = arr.len() - 1;
    while left <= right {
        let mid = (left + right) / 2;
        match arr[mid].cmp(target) {
            std::cmp::Ordering::Equal => return Some(mid),
            std::cmp::Ordering::Less => left = mid + 1,
            std::cmp::Ordering::Greater => right = mid - 1,
        }
    }
    None
}

pub fn bench_searches(c: &mut Criterion) {
    const TOTAL: usize = 500;
    const MID_INDEX: i32 = (TOTAL / 2) as i32;
    let mut arr: [i32; TOTAL] = [0; TOTAL];
    for i in 0..TOTAL {
        arr[i] = i as i32;
    }

    c.bench_function("linear_search_first", |b| {
        b.iter(|| linear_search(black_box(&arr), black_box(&0)))
    });
    c.bench_function("linear_search_mid", |b| {
        b.iter(|| linear_search(black_box(&arr), black_box(&MID_INDEX)))
    });
    c.bench_function("linear_search_last", |b| {
        b.iter(|| linear_search(black_box(&arr), black_box(&(TOTAL as i32 - 1))))
    });
    c.bench_function("binary_search_first", |b| {
        b.iter(|| binary_search(black_box(&arr), black_box(&0)))
    });
    c.bench_function("binary_search_mid", |b| {
        b.iter(|| binary_search(black_box(&arr), black_box(&MID_INDEX)))
    });
    c.bench_function("binary_search_last", |b| {
        b.iter(|| binary_search(black_box(&arr), black_box(&(TOTAL as i32 - 1))))
    });
}

criterion_group!(benches, bench_searches);
criterion_main!(benches);
