use std::time::Duration;

use criterion::{criterion_group, Criterion};

fn day3_bench(c: &mut Criterion) {
  let mut group = c.benchmark_group("day3");
  group.measurement_time(Duration::new(10, 0));

  group.bench_function("part1", |b| {
    b.iter(|| aoc_2024::day3::parse_and_compute::<false>(aoc_2024::day3::INPUT))
  });
  group.bench_function("part2", |b| {
    b.iter(|| aoc_2024::day3::parse_and_compute::<true>(aoc_2024::day3::INPUT))
  });

  group.finish();
}

criterion_group!(benches, day3_bench);
