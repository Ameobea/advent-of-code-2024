use std::time::Duration;

use criterion::{criterion_group, Criterion};

fn day6_bench(c: &mut Criterion) {
  let mut group = c.benchmark_group("day8");
  group.measurement_time(Duration::new(10, 0));

  group.bench_function("part1", |b| {
    b.iter(|| aoc_2024::day8::part1(aoc_2024::day8::INPUT))
  });
  group.bench_function("part2", |b| {
    b.iter(|| aoc_2024::day8::part2(aoc_2024::day8::INPUT))
  });

  group.finish();
}

criterion_group!(benches, day6_bench);
