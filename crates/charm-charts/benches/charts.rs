#![allow(missing_docs)]

use std::hint::black_box;

use charm_charts::{BarChart, Sparkline};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn synthetic_series(points: usize) -> Vec<f64> {
    (0..points)
        .map(|i| ((i as f64) * 0.1).sin() * 50.0 + 50.0)
        .collect()
}

fn bench_sparkline(c: &mut Criterion) {
    let mut group = c.benchmark_group("sparkline");
    for points in [100_usize, 1_000] {
        let data = synthetic_series(points);
        let spark = Sparkline::new(&data);
        group.bench_with_input(BenchmarkId::from_parameter(points), &spark, |b, spark| {
            b.iter(|| black_box(spark).render());
        });
    }
    group.finish();
}

fn bench_barchart(c: &mut Criterion) {
    let labels: Vec<String> = (0..10).map(|i| format!("service-{i}")).collect();
    let label_refs: Vec<&str> = labels.iter().map(String::as_str).collect();
    let values: Vec<f64> = (0..10).map(|i| f64::from((i * 7) % 13 + 1)).collect();

    let horizontal = BarChart::new(&label_refs, &values).horizontal();
    let vertical = BarChart::new(&label_refs, &values).vertical();

    let mut group = c.benchmark_group("barchart_10_bars");
    group.bench_function("horizontal", |b| {
        b.iter(|| black_box(&horizontal).render());
    });
    group.bench_function("vertical", |b| {
        b.iter(|| black_box(&vertical).render());
    });
    group.finish();
}

criterion_group!(benches, bench_sparkline, bench_barchart);
criterion_main!(benches);
