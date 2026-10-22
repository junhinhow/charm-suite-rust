#![allow(missing_docs)]

use std::fmt::Write as _;
use std::hint::black_box;

use charm_diff::{side_by_side_diff, unified_diff};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

const CHANGE_EVERY: usize = 20;

fn synthetic_pair(lines: usize) -> (String, String) {
    let mut old = String::new();
    let mut new = String::new();
    for i in 0..lines {
        let _ = writeln!(old, "line {i}: the quick brown fox jumps");
        if i % CHANGE_EVERY == 0 {
            let _ = writeln!(new, "line {i}: the lazy dog sleeps");
        } else {
            let _ = writeln!(new, "line {i}: the quick brown fox jumps");
        }
    }
    (old, new)
}

fn bench_unified(c: &mut Criterion) {
    let mut group = c.benchmark_group("unified_diff");
    for lines in [100_usize, 1_000, 5_000] {
        let (old, new) = synthetic_pair(lines);
        group.throughput(Throughput::Elements(lines as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(lines),
            &(old, new),
            |b, (o, n)| {
                b.iter(|| unified_diff(black_box(o), black_box(n)));
            },
        );
    }
    group.finish();
}

fn bench_side_by_side(c: &mut Criterion) {
    let (old, new) = synthetic_pair(1_000);
    c.bench_function("side_by_side_diff/1000_lines_w80", |b| {
        b.iter(|| side_by_side_diff(black_box(&old), black_box(&new), 80));
    });
}

criterion_group!(benches, bench_unified, bench_side_by_side);
criterion_main!(benches);
