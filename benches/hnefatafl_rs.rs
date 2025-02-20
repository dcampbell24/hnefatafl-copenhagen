use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};

use hnefatafl_copenhagen::hnefatafl_rs;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("hnefatafl_rs", |b| b.iter(hnefatafl_rs));
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = criterion_benchmark
}

criterion_main!(benches);
