use std::sync::{Arc, Mutex};

use fxhash::FxHashSet;
use pathfinding::directed::astar::astar;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

pub const INPUT: &str = include_str!("../inputs/day16.txt");

fn parse_input(input: &str) -> Vec<Vec<char>> {
  input.lines().map(|l| l.chars().collect()).collect()
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Debug)]
enum Dir {
  U,
  D,
  L,
  R,
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

  fn successors<'a>(
    grid: &'a [Vec<char>],
    &(dir, (x, y)): &(Dir, (usize, usize)),
  ) -> impl Iterator<Item = ((Dir, (usize, usize)), usize)> + 'a {
    [
      (Dir::L, [x - 1, y]),
      (Dir::R, [x + 1, y]),
      (Dir::U, [x, y - 1]),
      (Dir::D, [x, y + 1]),
    ]
    .into_iter()
    .filter_map(move |(o_dir, [o_x, o_y])| {
      // no need for bounds checking because the maze is always surrounded by a wall
      let o = grid[y][x];
      if o == '#' {
        return None;
      }
      Some(((o_dir, (o_x, o_y)), if o_dir == dir { 1 } else { 1001 }))
    })
  }

  let path = astar(
    &(Dir::R, start),
    |c| successors(&grid, c),
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
      let path = astar(
        &(Dir::R, start),
        |c| successors(&grid, c),
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

      let path = astar(
        path.0.last().unwrap(),
        |c| successors(&grid, c),
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
