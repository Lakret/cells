use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs::read_to_string;

use cells::table::parse_from_input;
use cells::topological::topological_sort;

pub fn refcell_benchmark(c: &mut Criterion) {
  let megatable_raw = read_to_string("./sample_tables/megatable.json").unwrap();
  let (_, exprs) = parse_from_input(&megatable_raw).unwrap();

  c.bench_function("topological_sort", |b| {
    b.iter(|| topological_sort(black_box(&exprs)))
  });
}

criterion_group!(benches, refcell_benchmark);
criterion_main!(benches);
