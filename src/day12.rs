use fxhash::{FxHashMap, FxHashSet};
use pathfinding::prelude::dfs_reach;

#[cfg(feature = "local")]
pub const INPUT: &'static str = include_str!("../inputs/day12.txt");

fn parse_input(input: &str) -> Vec<Vec<char>> {
  input.lines().map(|l| l.chars().collect()).collect()
}

fn compute_links(input: &[Vec<char>]) -> FxHashMap<(usize, usize), Vec<(usize, usize)>> {
  let mut links: FxHashMap<(usize, usize), Vec<(usize, usize)>> = FxHashMap::default();

  for y in 0..input.len() {
    for x in 0..input[0].len() {
      let val = input[y][x];
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

        let o = input[o_y as usize][o_x as usize];
        if o == val {
          links
            .entry((x, y))
            .or_default()
            .push((o_x as usize, o_y as usize));
        }
      }
    }
  }

  links
}

fn part1(input: &str) -> usize {
  let input = parse_input(input);

  let mut visited_coords: FxHashSet<(usize, usize)> = FxHashSet::default();
  let links = compute_links(&input);

  for (&(x, y), others) in links.iter() {
    for &(o_x, o_y) in others {
      assert_eq!(input[y][x], input[o_y][o_x]);
    }
  }

  let mut cost = 0usize;
  for y in 0..input.len() {
    for x in 0..input[0].len() {
      if visited_coords.contains(&(x, y)) {
        continue;
      }
      let val = input[y][x];
      visited_coords.insert((x, y));

      let mut area = 0usize;
      let mut perim = 0usize;
      let neighbors = dfs_reach((x, y), |coord| {
        links
          .get(coord)
          .map(|v| v.as_slice())
          .unwrap_or_default()
          .iter()
          .copied()
      });
      for (x, y) in neighbors {
        assert_eq!(input[y][x], val);

        area += 1;
        visited_coords.insert((x, y));
        let val = input[y][x];

        for [o_x, o_y] in [
          [x as isize - 1, y as isize],
          [x as isize + 1, y as isize],
          [x as isize, y as isize - 1],
          [x as isize, y as isize + 1],
        ] {
          if o_x < 0 || o_x >= input[0].len() as isize {
            perim += 1;
            continue;
          }
          if o_y < 0 || o_y >= input.len() as isize {
            perim += 1;
            continue;
          }

          let o = input[o_y as usize][o_x as usize];
          if o == val {
            continue;
          } else {
            perim += 1;
          }
        }
      }

      cost += area * perim;
    }
  }

  cost
}

fn part2(input: &str) -> usize {
  let input = parse_input(input);

  let mut visited_coords: FxHashSet<(usize, usize)> = FxHashSet::default();
  let links = compute_links(&input);

  #[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
  enum Dir {
    L,
    R,
    U,
    D,
  }

  let mut border_dirs = vec![vec![vec![] as Vec<Dir>; input[0].len()]; input.len()];

  for y in 0..input.len() {
    for x in 0..input[0].len() {
      let val = input[y][x];

      let mut dirs = Vec::new();
      for (dir, [o_x, o_y]) in [
        (Dir::L, [x as isize - 1, y as isize]),
        (Dir::R, [x as isize + 1, y as isize]),
        (Dir::U, [x as isize, y as isize - 1]),
        (Dir::D, [x as isize, y as isize + 1]),
      ] {
        if o_x < 0 {
          dirs.push(Dir::L);
          continue;
        } else if o_x >= input[0].len() as isize {
          dirs.push(Dir::R);
          continue;
        }
        if o_y < 0 {
          dirs.push(Dir::U);
          continue;
        } else if o_y >= input.len() as isize {
          dirs.push(Dir::D);
          continue;
        }

        let o = input[o_y as usize][o_x as usize];
        if o == val {
          continue;
        } else {
          dirs.push(dir);
        }
      }

      border_dirs[y][x] = dirs;
    }
  }

  let mut cost = 0usize;
  for y in 0..input.len() {
    for x in 0..input[0].len() {
      if visited_coords.contains(&(x, y)) {
        continue;
      }
      let val = input[y][x];
      visited_coords.insert((x, y));

      let mut area = 0usize;
      let mut side_count = 0usize;
      let neighbors = dfs_reach((x, y), |coord| {
        links
          .get(coord)
          .map(|v| v.as_slice())
          .unwrap_or_default()
          .iter()
          .copied()
      });

      let mut visited_borders: FxHashSet<((usize, usize), Dir)> = FxHashSet::default();
      for (x, y) in neighbors {
        assert_eq!(input[y][x], val);

        area += 1;
        visited_coords.insert((x, y));
        let val = input[y][x];

        let dirs = border_dirs[y][x].as_slice();
        for &dir in dirs {
          if visited_borders.contains(&((x, y), dir)) {
            continue;
          }
          visited_borders.insert(((x, y), dir));
          side_count += 1;

          match dir {
            Dir::L | Dir::R => {
              // traverse up/down
              for y in (0..y).rev() {
                if input[y][x] != val {
                  break;
                }
                let o_borders = &border_dirs[y][x];
                if !o_borders.contains(&dir) {
                  break;
                }

                assert!(visited_borders.insert(((x, y), dir)));
              }
              for y in y + 1..input.len() {
                if input[y][x] != val {
                  break;
                }
                let o_borders = &border_dirs[y][x];
                if !o_borders.contains(&dir) {
                  break;
                }

                assert!(visited_borders.insert(((x, y), dir)));
              }
            },
            Dir::U | Dir::D => {
              // traverse left/right
              for x in (0..x).rev() {
                if input[y][x] != val {
                  break;
                }
                let o_borders = &border_dirs[y][x];
                if !o_borders.contains(&dir) {
                  break;
                }

                assert!(visited_borders.insert(((x, y), dir)));
              }
              for x in x + 1..input[0].len() {
                if input[y][x] != val {
                  break;
                }
                let o_borders = &border_dirs[y][x];
                if !o_borders.contains(&dir) {
                  break;
                }

                assert!(visited_borders.insert(((x, y), dir)));
              }
            },
          }
        }
      }

      cost += area * side_count;
    }
  }

  cost
}

#[cfg(feature = "local")]
pub fn solve() {
  let cost = part1(INPUT);
  println!("Part 1: {cost}");

  let cost = part2(INPUT);
  println!("Part 2: {cost}");
}
