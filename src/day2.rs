const INPUT: &'static str = include_str!("../inputs/day2.txt");

fn parse_input() -> Vec<Vec<isize>> {
  INPUT
    .lines()
    .map(|l| {
      l.split_ascii_whitespace()
        .map(|s| s.parse().unwrap())
        .collect::<Vec<_>>()
    })
    .collect()
}

fn row_safe(row: &[isize]) -> bool {
  let increasing = row[1] > row[0];

  for [last, cur] in row.array_windows::<2>().copied() {
    let now_increasing = cur > last;
    if increasing != now_increasing {
      return false;
    }

    let diff = (cur - last).abs();
    if diff < 1 || diff > 3 {
      return false;
    }
  }

  true
}

pub fn solve() {
  let input = parse_input();

  let safe_count = input.iter().filter(|row| row_safe(row)).count();
  println!("Part 1: {safe_count}");

  let mut safe_count = 0usize;
  let iter = input.into_iter();
  // brute-force; I don't care to try to keep track of the state for that stuff manually
  'row: for row in iter {
    for ix_to_remove in 0..row.len() {
      let mut row = row.clone();
      row.remove(ix_to_remove);
      let is_safe = row_safe(&row);

      if is_safe {
        safe_count += 1;
        continue 'row;
      }
    }
  }

  println!("Part 2: {safe_count}");
}
