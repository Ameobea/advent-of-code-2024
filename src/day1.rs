use std::collections::HashMap;

const INPUT: &'static str = include_str!("../inputs/day1.txt");

fn parse_input() -> Vec<(usize, usize)> {
  INPUT
    .lines()
    .map(|l| {
      let mut spl = l.split_ascii_whitespace();
      let a = spl.next().unwrap().parse::<usize>().unwrap();
      let b = spl.next().unwrap().parse::<usize>().unwrap();
      (a, b)
    })
    .collect()
}

pub fn solve() {
  let input = parse_input();

  let mut a = input.iter().map(|(a, _b)| *a).collect::<Vec<_>>();
  a.sort_unstable();
  let mut b = input.iter().map(|(_a, b)| *b).collect::<Vec<_>>();
  b.sort_unstable();

  let out = a
    .iter()
    .copied()
    .zip(b.iter().copied())
    .map(|(a, b)| ((a as isize) - (b as isize)).abs())
    .sum::<isize>();

  println!("Part 1: {out}");

  let mut appearance_counts: HashMap<usize, usize> = HashMap::new();
  for &b in &b {
    *appearance_counts.entry(b).or_default() += 1;
  }

  let mut sum = 0usize;
  for &a in &a {
    let appearances = appearance_counts.get(&a).copied().unwrap_or(0);
    sum += a * appearances;
  }

  println!("Part 2: {sum}");
}
