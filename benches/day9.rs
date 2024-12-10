use std::time::Duration;

use aoc_2024::helpers::leak_to_page_aligned;
use criterion::{criterion_group, Criterion};

fn day6_bench(c: &mut Criterion) {
  let mut group = c.benchmark_group("day9");
  group.measurement_time(Duration::new(10, 0));

  let aligned_input = leak_to_page_aligned(aoc_2024::day9::INPUT);

  //   group.bench_function("part1", |b| {
  //     b.iter(|| aoc_2024::day9::part1(aoc_2024::day9::INPUT))
  //   });
  group.bench_function("part2", |b| b.iter(|| aoc_2024::day9::part2(aligned_input)));

  group.finish();
}

criterion_group!(benches, day6_bench);
