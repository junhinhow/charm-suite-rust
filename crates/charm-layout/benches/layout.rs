#![allow(missing_docs)]

use std::hint::black_box;

use charm_layout::{Direction, Flex, Grid};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn flex_row(children: usize) -> Flex {
    let ratio = 1.0 / children as f64;
    (0..children).fold(
        Flex::new(Direction::Row).set_size(120, 24).set_gap(1),
        |flex, i| flex.add_child(ratio, format!("panel {i}")),
    )
}

fn grid_4_cols(children: usize) -> Grid {
    Grid::new(4)
        .set_size(120, 24)
        .set_gap(1)
        .add_children((0..children).map(|i| format!("cell {i}")))
}

fn bench_flex(c: &mut Criterion) {
    let mut group = c.benchmark_group("flex_row");
    for children in [2_usize, 5, 20] {
        let flex = flex_row(children);
        group.bench_with_input(BenchmarkId::from_parameter(children), &flex, |b, flex| {
            b.iter(|| black_box(flex).render());
        });
    }
    group.finish();
}

fn bench_grid(c: &mut Criterion) {
    let grid = grid_4_cols(20);
    c.bench_function("grid_4_cols/20_children", |b| {
        b.iter(|| black_box(&grid).render());
    });
}

criterion_group!(benches, bench_flex, bench_grid);
criterion_main!(benches);
