#![feature(array_chunks, array_windows, portable_simd)]

use std::{
  fmt::Display,
  simd::{
    cmp::{SimdPartialEq, SimdPartialOrd},
    i32x8, u8x8,
  },
};

#[cfg(feature = "local")]
pub const INPUT: &[u8] = include_bytes!("../inputs/day8.txt");

const GRID_SIZE: usize = 50;
const LINE_SIZE: usize = GRID_SIZE + 1;

#[inline(always)]
fn parse_input<const EMPTY: i32, const ANTENNAS_ARE_ANTINODES: bool>(
  positions_by_char: &mut [[(i32, i32); 4]; 123 - 47],
  input: &[u8],
  antinodes: &mut [[bool; 50]; 50],
  antinode_count: &mut usize,
) {
  let mut i = 0usize;
  while i < LINE_SIZE * GRID_SIZE {
    let char_ptr = unsafe { input.as_ptr().add(i) };

    let c = if i < LINE_SIZE * GRID_SIZE - 8 {
      let vector = u8x8::from_slice(unsafe { std::slice::from_raw_parts(char_ptr as *const _, 8) });
      let mask = vector.simd_ge(u8x8::splat('0' as u8));
      match mask.first_set() {
        Some(hit_ix) => {
          i += hit_ix;
        },
        None => {
          i += 8;
          continue;
        },
      };

      unsafe { *input.get_unchecked(i) }
    } else {
      let c = unsafe { std::ptr::read(char_ptr) };
      if c < '0' as u8 {
        i += 1;
        continue;
      }
      c
    };

    let y = i / LINE_SIZE;
    let x = i % LINE_SIZE;

    if ANTENNAS_ARE_ANTINODES {
      unsafe {
        *antinodes.get_unchecked_mut(y).get_unchecked_mut(x) = true;
      }
      *antinode_count += 1;
    }

    let entry_ptr = unsafe { positions_by_char.as_mut_ptr().add(c as usize - 47) };
    let mask = i32x8::splat(EMPTY);
    let entry_simd = unsafe { std::ptr::read_unaligned(entry_ptr as *const _) };
    let eq = mask.simd_eq(entry_simd);
    let first_hit = unsafe { eq.first_set().unwrap_unchecked() };

    unsafe {
      let entry_ptr: *mut (i32, i32) = entry_ptr as *mut _;
      let ix = first_hit / 2;
      *entry_ptr.add(ix) = (x as i32, y as i32);
    }

    i += 1;
  }
}

pub fn part1(input: &[u8]) -> impl Display {
  const EMPTY: i32 = 5000;

  let mut positions_by_char: [[(i32, i32); 4]; 123 - 47] = [[(EMPTY, EMPTY); 4]; 123 - 47];
  let mut antinodes: [[bool; 50]; 50] = unsafe { std::mem::zeroed() };
  let mut antinode_count = 0usize;

  parse_input::<EMPTY, false>(
    &mut positions_by_char,
    input,
    &mut antinodes,
    &mut antinode_count,
  );

  for positions in positions_by_char {
    if positions[0].0 == EMPTY {
      continue;
    }

    for (pos_ix, o_pos_ix) in [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)] {
      let (pos, o_pos) = (positions[pos_ix], positions[o_pos_ix]);
      let antinode_offset = (o_pos.0 - pos.0, o_pos.1 - pos.1);
      let antinode_pos = (o_pos.0 + antinode_offset.0, o_pos.1 + antinode_offset.1);
      if antinode_pos.0 < 0
        || antinode_pos.0 >= GRID_SIZE as i32
        || antinode_pos.1 < 0
        || antinode_pos.1 >= GRID_SIZE as i32
      {
        // skip
      } else {
        let was_antinode = unsafe {
          antinodes
            .get_unchecked(antinode_pos.1 as usize)
            .get_unchecked(antinode_pos.0 as usize)
        };
        if !was_antinode {
          unsafe {
            *antinodes
              .get_unchecked_mut(antinode_pos.1 as usize)
              .get_unchecked_mut(antinode_pos.0 as usize) = true;
          }
          antinode_count += 1;
        }
      }

      let antinode_pos = (pos.0 - antinode_offset.0, pos.1 - antinode_offset.1);
      if antinode_pos.0 < 0
        || antinode_pos.0 >= GRID_SIZE as i32
        || antinode_pos.1 < 0
        || antinode_pos.1 >= GRID_SIZE as i32
      {
        continue;
      }

      let was_antinode = unsafe {
        antinodes
          .get_unchecked(antinode_pos.1 as usize)
          .get_unchecked(antinode_pos.0 as usize)
      };
      if !was_antinode {
        unsafe {
          *antinodes
            .get_unchecked_mut(antinode_pos.1 as usize)
            .get_unchecked_mut(antinode_pos.0 as usize) = true;
        }
        antinode_count += 1;
      }
    }

    // return 0;
  }

  antinode_count
}

pub fn part2(input: &[u8]) -> impl Display {
  const EMPTY: i32 = i32::MIN;

  let mut positions_by_char: [[(i32, i32); 4]; 123 - 47] = [[(EMPTY, EMPTY); 4]; 123 - 47];
  let mut antinodes: [[bool; 50]; 50] = unsafe { std::mem::zeroed() };
  let mut antinode_count = 0usize;

  parse_input::<EMPTY, true>(
    &mut positions_by_char,
    input,
    &mut antinodes,
    &mut antinode_count,
  );

  for positions in positions_by_char {
    if positions[0].0 == EMPTY {
      continue;
    }

    for (pos_ix, o_pos_ix) in [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)] {
      for (pos, o_pos) in [
        (positions[pos_ix], positions[o_pos_ix]),
        (positions[o_pos_ix], positions[pos_ix]),
      ] {
        let antinode_offset = (
          (o_pos.0 as i32).wrapping_sub(pos.0 as i32),
          (o_pos.1 as i32).wrapping_sub(pos.1 as i32),
        );
        let mut antinode_pos = [
          (o_pos.0 as i32).wrapping_add(antinode_offset.0),
          (o_pos.1 as i32).wrapping_add(antinode_offset.1),
        ];

        loop {
          if antinode_pos[0] < 0
            || antinode_pos[0] >= GRID_SIZE as i32
            || antinode_pos[1] < 0
            || antinode_pos[1] >= GRID_SIZE as i32
          {
            break;
          }

          let was_antinode = unsafe {
            *antinodes
              .get_unchecked(antinode_pos[1] as usize)
              .get_unchecked(antinode_pos[0] as usize)
          };
          if !was_antinode {
            unsafe {
              *antinodes
                .get_unchecked_mut(antinode_pos[1] as usize)
                .get_unchecked_mut(antinode_pos[0] as usize) = true;
            }
            antinode_count += 1;
          }

          antinode_pos[0] += antinode_offset.0;
          antinode_pos[1] += antinode_offset.1;
        }
      }
    }
  }

  antinode_count
}

#[cfg(feature = "local")]
pub fn solve() {
  let out = part1(INPUT);
  println!("Part 1: {out}");

  let out = part2(INPUT);
  println!("Part 2: {out}");
}

pub fn run(input: &[u8]) -> impl Display { part2(input) }
