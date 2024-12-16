use std::sync::{Arc, Mutex};

use fxhash::FxHashSet;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

pub const INPUT: &str = include_str!("../inputs/day16.txt");

fn parse_input(input: &str) -> Vec<Vec<char>> {
  input.lines().map(|l| l.chars().collect()).collect()
}

pub fn solve() {
  let grid = parse_input(INPUT);

  let mut start = (0, 0);
  let mut end = (0, 0);
  for y in 0..grid.len() {
    for x in 0..grid.len() {
      if grid[y][x] == 'S' {
        start = (x, y)
      } else if grid[y][x] == 'E' {
        end = (x, y)
      }
    }
  }

  #[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Debug)]
  enum Dir {
    U,
    D,
    L,
    R,
  }

  let path = pathfinding::directed::astar::astar(
    &(Dir::R, start),
    |&(dir, (x, y))| {
      let mut outs = Vec::new();

      for (o_dir, [o_x, o_y]) in [
        (Dir::L, [x as isize - 1, y as isize]),
        (Dir::R, [x as isize + 1, y as isize]),
        (Dir::U, [x as isize, y as isize - 1]),
        (Dir::D, [x as isize, y as isize + 1]),
      ] {
        if o_x < 0 || o_y < 0 || o_x >= grid[0].len() as _ || o_y >= grid.len() as _ {
          continue;
        }

        if o_dir == Dir::L && dir == Dir::R {
          continue;
        }
        if o_dir == Dir::R && dir == Dir::L {
          continue;
        }
        if o_dir == Dir::D && dir == Dir::U {
          continue;
        }
        if o_dir == Dir::U && dir == Dir::D {
          continue;
        }

        let o = grid[y][x];
        if o == '#' {
          continue;
        }
        outs.push((
          o_dir,
          (o_x as usize, o_y as usize),
          if o_dir == dir { 1 } else { 1001 },
        ));
      }

      outs
        .into_iter()
        .map(|(dir, coord, cost)| ((dir, coord), cost))
    },
    |_| 1,
    |c| c.1 == end,
  )
  .unwrap();

  println!("Part 1: {}", path.1);

  let min_cost = path.1;
  let valid_tiles = Arc::new(Mutex::new(FxHashSet::default()));
  let valid_tiles_clone = valid_tiles.clone();

  let width = grid[0].len();
  (0..grid.len())
    .into_par_iter()
    .flat_map(|y| (0..width).into_par_iter().map(move |x| (x, y)))
    .for_each(move |(x, y)| {
      let path = pathfinding::directed::astar::astar(
        &(Dir::R, start),
        |&(dir, (x, y))| {
          let mut outs = Vec::new();
          let (turn_dirs, turn_cost) = match dir {
            Dir::U => (vec![Dir::L, Dir::R], 1000),
            Dir::D => (vec![Dir::L, Dir::R], 1000),
            Dir::L => (vec![Dir::U, Dir::D], 1000),
            Dir::R => (vec![Dir::U, Dir::D], 1000),
          };
          for turn_dir in turn_dirs {
            outs.push((turn_dir, (x, y), turn_cost));
          }

          for (o_dir, [o_x, o_y]) in [
            (Dir::L, [x as isize - 1, y as isize]),
            (Dir::R, [x as isize + 1, y as isize]),
            (Dir::U, [x as isize, y as isize - 1]),
            (Dir::D, [x as isize, y as isize + 1]),
          ] {
            if o_x < 0 || o_y < 0 || o_x >= grid[0].len() as _ || o_y >= grid.len() as _ {
              continue;
            }

            if o_dir == Dir::L && dir == Dir::R {
              continue;
            }
            if o_dir == Dir::R && dir == Dir::L {
              continue;
            }
            if o_dir == Dir::D && dir == Dir::U {
              continue;
            }
            if o_dir == Dir::U && dir == Dir::D {
              continue;
            }

            let o = grid[y][x];
            if o == '#' {
              continue;
            }
            outs.push((
              o_dir,
              (o_x as usize, o_y as usize),
              if o_dir == dir { 1 } else { 1001 },
            ));
          }

          outs
            .into_iter()
            .map(|(dir, coord, cost)| ((dir, coord), cost))
        },
        |_| 1,
        |c| c.1 == (x, y),
      );

      let Some(path) = path else {
        return;
      };
      let firpart_cost = path.1;
      if firpart_cost > min_cost {
        return;
      }

      let path = pathfinding::directed::astar::astar(
        path.0.last().unwrap(),
        |&(dir, (x, y))| {
          let mut outs = Vec::new();
          let (turn_dirs, turn_cost) = match dir {
            Dir::U => (vec![Dir::L, Dir::R], 1000),
            Dir::D => (vec![Dir::L, Dir::R], 1000),
            Dir::L => (vec![Dir::U, Dir::D], 1000),
            Dir::R => (vec![Dir::U, Dir::D], 1000),
          };
          for turn_dir in turn_dirs {
            outs.push((turn_dir, (x, y), turn_cost));
          }

          for (o_dir, [o_x, o_y]) in [
            (Dir::L, [x as isize - 1, y as isize]),
            (Dir::R, [x as isize + 1, y as isize]),
            (Dir::U, [x as isize, y as isize - 1]),
            (Dir::D, [x as isize, y as isize + 1]),
          ] {
            if o_x < 0 || o_y < 0 || o_x >= grid[0].len() as _ || o_y >= grid.len() as _ {
              continue;
            }

            if o_dir == Dir::L && dir == Dir::R {
              continue;
            }
            if o_dir == Dir::R && dir == Dir::L {
              continue;
            }
            if o_dir == Dir::D && dir == Dir::U {
              continue;
            }
            if o_dir == Dir::U && dir == Dir::D {
              continue;
            }

            let o = grid[y][x];
            if o == '#' {
              continue;
            }
            outs.push((
              o_dir,
              (o_x as usize, o_y as usize),
              if o_dir == dir { 1 } else { 1001 },
            ));
          }

          outs
            .into_iter()
            .map(|(dir, coord, cost)| ((dir, coord), cost))
        },
        |_| 1,
        |c| c.1 == end,
      );

      let Some(path) = path else {
        return;
      };
      if path.1 + firpart_cost > min_cost {
        return;
      }

      let mut valid_tiles = valid_tiles.lock().unwrap();
      for p in path.0 {
        valid_tiles.insert(p.1);
      }
    });

  println!("Part 2: {}", valid_tiles_clone.lock().unwrap().len())
}
