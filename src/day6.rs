#![feature(array_chunks, array_windows, duration_constructors, portable_simd)]

pub const INPUT_BYTES: &'static [u8] = include_bytes!("../inputs/day6.txt");

use std::{
  fmt::Display,
  sync::atomic::{AtomicUsize, Ordering},
};

use fxhash::FxHashSet;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

const GRID_SIZE: usize = 130;
const GRID_SIZE_PLUS_NEWLINE: usize = GRID_SIZE + 1;

static mut GUARD_POS: (usize, usize) = (0, 0);

type Grid = [bool; GRID_SIZE * GRID_SIZE];

static mut GRID: Grid = [false; GRID_SIZE * GRID_SIZE];

fn grid() -> &'static mut Grid { unsafe { &mut GRID } }

pub fn parse_input(input: &[u8]) -> ((usize, usize), &'static Grid) {
  for (y, row) in input.array_chunks::<GRID_SIZE_PLUS_NEWLINE>().enumerate() {
    for (x, &c) in row[..GRID_SIZE].iter().enumerate() {
      if c == '^' as u8 {
        unsafe {
          GUARD_POS = (x, y);
        }
        continue;
      }
      if c == '#' as u8 {
        unsafe { *grid().get_unchecked_mut(y * GRID_SIZE + x) = true };
      }
    }
  }

  (unsafe { GUARD_POS }, grid())
}

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Dir {
  Up,
  Left,
  Down,
  Right,
}

fn sim_part2(
  obstruction_pos: (usize, usize),
  grid: &Grid,
  mut guard_pos: (usize, usize),
  mut dir: Dir,
) -> bool {
  let mut visited_positions_directions: Vec<bool> = Vec::new();
  visited_positions_directions.resize(GRID_SIZE * GRID_SIZE * 4, false);

  // returns `false` if marking a spot that's already been marked
  let mut insert_visited_pos_dir = |guard_pos: (usize, usize), dir: Dir| -> bool {
    let dir_multiplier = match dir {
      Dir::Up => 0,
      Dir::Left => 1,
      Dir::Down => 2,
      Dir::Right => 3,
    };
    let ix = dir_multiplier * (GRID_SIZE * GRID_SIZE) + guard_pos.1 * GRID_SIZE + guard_pos.0;
    if unsafe { *visited_positions_directions.get_unchecked(ix) } {
      return false;
    }

    unsafe {
      *visited_positions_directions.get_unchecked_mut(ix) = true;
    }
    true
  };

  insert_visited_pos_dir(guard_pos, dir);

  'outer: loop {
    loop {
      let next_pos = match dir {
        Dir::Up => {
          if guard_pos.1 == 0 {
            break 'outer;
          }
          (guard_pos.0, guard_pos.1 - 1)
        },
        Dir::Left => {
          if guard_pos.0 == 0 {
            break 'outer;
          }
          (guard_pos.0 - 1, guard_pos.1)
        },
        Dir::Down => {
          if guard_pos.1 >= (GRID_SIZE - 1) {
            break 'outer;
          }
          (guard_pos.0, guard_pos.1 + 1)
        },
        Dir::Right => {
          if guard_pos.0 >= (GRID_SIZE - 1) {
            break 'outer;
          }
          (guard_pos.0 + 1, guard_pos.1)
        },
      };

      if next_pos == obstruction_pos
        || unsafe { *grid.get_unchecked(next_pos.1 * GRID_SIZE + next_pos.0) }
      {
        break;
      }

      guard_pos = next_pos;
      if !insert_visited_pos_dir(guard_pos, dir) {
        return true;
      }
    }

    dir = match dir {
      Dir::Up => Dir::Right,
      Dir::Left => Dir::Up,
      Dir::Down => Dir::Left,
      Dir::Right => Dir::Down,
    };
    continue;
  }

  false
}

pub fn part2(input: &[u8]) -> usize {
  let (guard_pos, grid) = parse_input(input);

  let loop_count = AtomicUsize::new(0);
  let xs = 0..GRID_SIZE;
  let ys = 0..GRID_SIZE;
  xs.into_par_iter().for_each(|x| {
    for y in ys.clone() {
      let obstruction_pos = (x, y);
      if obstruction_pos == guard_pos {
        continue;
      }
      if unsafe { *grid.get_unchecked(obstruction_pos.1 * GRID_SIZE + obstruction_pos.0) } {
        continue;
      }

      let did_loop = sim_part2(obstruction_pos, &grid, guard_pos, Dir::Up);
      if did_loop {
        loop_count.fetch_add(1, Ordering::Relaxed);
      }
    }
  });

  loop_count.load(Ordering::Relaxed)
}

pub fn solve() {
  let (guard_pos, grid) = parse_input(INPUT_BYTES);
  let mut dir = Dir::Up;
  let mut guard_pos = (guard_pos.0 as isize, guard_pos.1 as isize);

  let mut visited_positions: FxHashSet<(isize, isize)> = FxHashSet::default();
  visited_positions.insert(guard_pos);
  loop {
    let next_pos = match dir {
      Dir::Up => (guard_pos.0, guard_pos.1 - 1),
      Dir::Left => (guard_pos.0 - 1, guard_pos.1),
      Dir::Down => (guard_pos.0, guard_pos.1 + 1),
      Dir::Right => (guard_pos.0 + 1, guard_pos.1),
    };
    if next_pos.0 < 0
      || next_pos.0 >= GRID_SIZE as isize
      || next_pos.1 < 0
      || next_pos.1 >= GRID_SIZE as isize
    {
      break;
    }
    if grid[next_pos.1 as usize * GRID_SIZE + next_pos.0 as usize] {
      dir = match dir {
        Dir::Up => Dir::Right,
        Dir::Left => Dir::Up,
        Dir::Down => Dir::Left,
        Dir::Right => Dir::Down,
      };
      continue;
    } else {
      guard_pos = next_pos;
      visited_positions.insert(guard_pos);
    }
  }

  let out = visited_positions.len();

  println!("Part 1: {out}");

  let loop_count = part2(INPUT_BYTES);

  println!("Part 2: {loop_count}");
}

pub fn run(input: &[u8]) -> impl Display { part2(input) }
