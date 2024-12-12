use itertools::Either;

pub const INPUT: &'static str = include_str!("../inputs/day11.txt");

fn parse_input(input: &str) -> Vec<usize> {
  input
    .split_ascii_whitespace()
    .map(|s| s.parse().unwrap())
    .collect()
}

fn digit_count(n: usize) -> usize {
  if n < 10 {
    return 1;
  }

  n.ilog10() as usize + 1
}

fn get_multiplier_for_digit_count(digit_count: usize) -> usize {
  10usize.pow(digit_count as u32 - 1)
}

fn split_middle(v: usize, digit_count: usize) -> (usize, usize) {
  let lhs_multiplier = get_multiplier_for_digit_count(digit_count / 2 + 1);
  let left = v / lhs_multiplier;
  let right = v - left * lhs_multiplier;

  (left, right)
}

#[test]
fn split_middle_c() {
  let n = 123456;
  assert_eq!(split_middle(n, digit_count(n)), (123, 456));
}

fn next(val: usize) -> Either<[usize; 1], [usize; 2]> {
  let digit_count = digit_count(val);
  match val {
    0 => Either::Left([1]),
    _ if digit_count % 2 == 0 => {
      let (left, right) = split_middle(val, digit_count);
      Either::Right([left, right])
    },
    _ => Either::Left([val * 2024]),
  }
}

// 2024
// 20, 24
// 2, 0, 2, 4
// 4048, 1, 4048, 8096
// 40, 48, 2024, 40, 48, 80, 96
// 4, 0, 4, 8, 20, 24, 4, 0, 4, 8, 8, 0, 9, 6
// 8096, 1, 8096, 16192, 2, 0, 2, 4, 8096, 16192, 16192, 1, 18261, 12144

// 1
// 2
// 4
// 4
// 7
// 14
// 16
// 20
// 39
// 62
// 81
// 110
// 200
// 328
// 418

#[cfg(feature = "local")]
pub fn solve() {
  let input = parse_input(INPUT);

  let mut stones = input;
  for _ in 0..25 {
    stones = stones
      .into_iter()
      .flat_map(|s| {
        next(s)
          .map_left(|l| l.into_iter())
          .map_right(|r| r.into_iter())
      })
      .collect();
  }

  let out = stones.len();
  println!("Part 1: {out}");
}
