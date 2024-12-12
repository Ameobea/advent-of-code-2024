pub const INPUT: &'static str = include_str!("../inputs/day10.txt");

fn parse_input(input: &str) -> Vec<Vec<usize>> {
  input
    .lines()
    .map(|l| l.chars().map(|c| c.to_string().parse().unwrap()).collect())
    .collect()
}

#[cfg(feature = "local")]
pub fn solve() {
  use fxhash::FxHashMap;
  use pathfinding::prelude::{astar, count_paths};

  let input = parse_input(INPUT);

  let mut dsts_by_src: FxHashMap<(usize, usize), Vec<(usize, usize)>> = FxHashMap::default();
  let mut starting_coords = Vec::new();
  let mut ending_coords = Vec::new();
  for x in 0..input[0].len() {
    for y in 0..input.len() {
      let val = input[y][x];
      if val == 0 {
        starting_coords.push((x, y));
      } else if val == 9 {
        ending_coords.push((x, y));
      }

      for [o_x, o_y] in [
        [x as isize - 1, y as isize],
        [x as isize + 1, y as isize],
        [x as isize, y as isize - 1],
        [x as isize, y as isize + 1],
      ] {
        if o_x < 0 || o_x >= input[0].len() as isize {
          continue;
        }
        if o_y < 0 || o_y >= input.len() as isize {
          continue;
        }
        let o_val = input[o_y as usize][o_x as usize];
        if (o_val as isize - val as isize) == 1 {
          dsts_by_src
            .entry((x, y))
            .or_default()
            .push((o_x as usize, o_y as usize));
        }
      }
    }
  }

  let mut total_score = 0usize;
  for &start in &starting_coords {
    for &end in &ending_coords {
      let path = astar(
        &start,
        |c| {
          dsts_by_src
            .get(c)
            .map(|v| v.as_slice())
            .unwrap_or_default()
            .into_iter()
            .map(|c| (*c, 1))
        },
        |_| 1,
        |s| *s == end,
      );
      if path.is_some() {
        total_score += 1;
      }
    }
  }

  let out = total_score;

  println!("Part 1: {out}");

  let mut out = 0usize;
  for start in starting_coords {
    for &end in &ending_coords {
      let uniq_paths = count_paths(
        start,
        |c| {
          dsts_by_src
            .get(c)
            .map(|v| v.as_slice())
            .unwrap_or_default()
            .into_iter()
            .copied()
        },
        |s| *s == end,
      );
      out += uniq_paths;
    }
  }

  println!("Part 2: {out}");
}
