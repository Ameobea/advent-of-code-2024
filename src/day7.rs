#![feature(array_chunks, array_windows, duration_constructors, portable_simd)]

#[cfg(feature = "local")]
pub const INPUT: &'static str = include_str!("../inputs/day7.txt");

use std::{fmt::Display, sync::Arc};

use itertools::{repeat_n, Itertools};
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};

fn parse_input(input: &str) -> Vec<(usize, Vec<usize>)> {
  input
    .lines()
    .map(|l| {
      let (f, rest) = l.split_once(": ").unwrap();
      (
        f.parse().unwrap(),
        rest
          .split_ascii_whitespace()
          .map(|n| n.parse().unwrap())
          .collect(),
      )
    })
    .collect()
}

#[derive(Clone, Copy, Debug)]
enum Op {
  Add,
  Mul,
  Concat,
}

// who cares man
// I don't want to fk around with logarithms
fn digit_count(n: usize) -> usize {
  if n < 10 {
    1
  } else if n < 100 {
    2
  } else if n < 1_000 {
    3
  } else if n < 10_000 {
    4
  } else if n < 100_000 {
    5
  } else if n < 1_000_000 {
    6
  } else if n < 10_000_000 {
    7
  } else if n < 100_000_000 {
    8
  } else if n < 1_000_000_000 {
    9
  } else {
    unimplemented!()
  }
}

fn get_multiplier_for_digit_count(digit_count: usize) -> usize {
  10usize.pow(digit_count as u32 - 1)
}

impl Op {
  fn apply(&self, a: usize, b: usize) -> usize {
    match self {
      Op::Add => a + b,
      Op::Mul => a * b,
      // Op::Concat => format!("{a}{b}").parse().unwrap(),
      Op::Concat => {
        let desired_a_digit_count = digit_count(b) + 1;
        let a_multiplier = get_multiplier_for_digit_count(desired_a_digit_count);
        a * a_multiplier + b
      },
    }
  }
}

#[test]
fn concat_correctness() {
  assert_eq!(Op::Concat.apply(12, 34), 1234);
  assert_eq!(Op::Concat.apply(543, 21), 54321);
  assert_eq!(Op::Concat.apply(5432, 1004), 54321004);
}

// we pass target as an optimization.  Since all operators only increase the output, if we ever
// exceed the output then we can bail out early.
fn calc(args: &[usize], operators: &[Op], target: usize) -> usize {
  let mut acc = operators[0].apply(unsafe { *args.get_unchecked(0) }, unsafe {
    *args.get_unchecked(1)
  });
  for (arg_ix, arg) in args.iter().enumerate().skip(2) {
    let op = unsafe { operators.get_unchecked(arg_ix - 1) };
    acc = op.apply(acc, *arg);
    if acc > target {
      return usize::MAX;
    }
  }
  acc
}

fn calc_n_target<const ARG_COUNT: usize>(args: &[usize], operators: &[Op], target: usize) -> usize {
  let mut acc = operators[0].apply(unsafe { *args.get_unchecked(0) }, unsafe {
    *args.get_unchecked(1)
  });
  for (arg_ix, arg) in (0..ARG_COUNT)
    .skip(2)
    .map(|arg_ix| (arg_ix, unsafe { args.get_unchecked(arg_ix) }))
  {
    let op = unsafe { operators.get_unchecked(arg_ix - 1) };
    acc = op.apply(acc, *arg);
    if arg_ix >= 3 && acc > target {
      return usize::MAX;
    }
  }
  acc
}

fn calc_n_no_target<const ARG_COUNT: usize>(args: &[usize], operators: &[Op]) -> usize {
  let mut acc = operators[0].apply(unsafe { *args.get_unchecked(0) }, unsafe {
    *args.get_unchecked(1)
  });
  for (arg_ix, arg) in (0..ARG_COUNT)
    .skip(2)
    .map(|arg_ix| (arg_ix, unsafe { args.get_unchecked(arg_ix) }))
  {
    let op = unsafe { operators.get_unchecked(arg_ix - 1) };
    acc = op.apply(acc, *arg);
  }
  acc
}

fn calc_n<const ARG_COUNT: usize>(args: &[usize], operators: &[Op], target: usize) -> usize {
  // if target > 100_000_000 {
  //   calc_n_no_target::<ARG_COUNT>(args, operators)
  // } else {
  calc_n_target::<ARG_COUNT>(args, operators, target)
  // }
}

fn calc_arb(args: &[usize], operators: &[Op], target: usize) -> usize {
  match args.len() {
    0 | 1 | 2 => unreachable!(),
    // 2 => calc_n::<2>(args, operators, target),
    3 => calc_n::<3>(args, operators, target),
    4 => calc_n::<4>(args, operators, target),
    5 => calc_n::<5>(args, operators, target),
    6 => calc_n::<6>(args, operators, target),
    7 => calc_n::<7>(args, operators, target),
    8 => calc_n::<8>(args, operators, target),
    9 => calc_n::<9>(args, operators, target),
    10 => calc_n::<10>(args, operators, target),
    11 => calc_n::<11>(args, operators, target),
    12 => calc_n::<12>(args, operators, target),
    other => unimplemented!("arg count: {other}"),
  }
}

pub fn part1(input: &str) -> impl Display {
  let input = parse_input(input);

  input
    .iter()
    .filter(|(res, args)| {
      let ops = repeat_n([Op::Add, Op::Mul], args.len() - 1).multi_cartesian_product();
      for ops in ops {
        if calc_arb(args, &ops, *res) == *res {
          return true;
        }
      }

      false
    })
    .map(|(val, _args)| *val)
    .sum::<usize>()
}

#[cached::proc_macro::cached]
fn get_permuted_ops(arg_count: usize) -> Arc<Vec<Vec<Op>>> {
  Arc::new(
    repeat_n([Op::Add, Op::Mul, Op::Concat], arg_count - 1)
      .multi_cartesian_product()
      .collect(),
  )
}

pub fn part2(input: &str) -> impl Display {
  let input = parse_input(input);

  input
    .par_iter()
    .filter(|(res, args)| {
      let ops = get_permuted_ops(args.len());
      ops
        .iter()
        // .par_bridge()
        .any(|ops| calc_arb(args, &ops, *res) == *res)
    })
    .map(|(val, _args)| *val)
    .sum::<usize>()
}

#[cfg(feature = "local")]
pub fn solve() {
  let part1 = part1(INPUT);
  println!("Part 1: {part1}");

  let part2 = part2(INPUT);
  println!("Part 2: {part2}");
}

pub fn run(input: &str) -> impl Display { part2(input) }
