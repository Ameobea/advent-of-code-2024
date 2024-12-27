#![feature(array_chunks, array_windows, portable_simd)]

use std::fmt::Display;

#[cfg(feature = "local")]
pub const INPUT: &'static [u8] = include_bytes!("../inputs/day9.txt");

fn parse_digit(c: u8) -> usize { (c.saturating_sub(48)) as usize }

fn parse_input(input: &[u8]) -> (Vec<usize>, Vec<MiniVec>, Vec<usize>) {
  let mut it = input.array_chunks::<2>();

  let mut orig_counts = Vec::with_capacity(input.len() / 2);
  let mut slots: Vec<MiniVec> = Vec::with_capacity(input.len() / 2);
  unsafe { slots.set_len(input.len() / 2) };
  let mut empty_spaces = Vec::with_capacity(input.len());

  let mut id = 0usize;
  while let Some(&[size, free]) = it.next() {
    let size = parse_digit(size);
    let free = parse_digit(free);
    orig_counts.push(size as usize);
    slots[id].len = 1;
    slots[id].elements[0] = Slot {
      count: size as _,
      id,
    };
    empty_spaces.push(free as usize);
    id += 1;
  }
  empty_spaces.push(0);

  (orig_counts, slots, empty_spaces)
}

#[derive(Clone, Copy, Debug)]
struct Slot {
  pub id: usize,
  pub count: usize,
}

impl Slot {
  fn checksum(&self, total_prev: usize) -> usize {
    // naive impl:
    // (0..self.count)
    //   .map(|i| (total_prev + i as usize) * self.id as usize)
    //   .sum::<usize>()
    // So, this condenses down to a sum of the following:
    //
    // (total_prev + 0) * id
    // (total_prev + 1) * id
    // (total_prev + 2) * id
    // ...
    // (total_prev + (count - 1)) * id
    //
    // the `total_prev` part can be split out:
    // total_prev * self.count * id
    //
    // leaving that base plus a sum of the following:
    //
    // 0 * id
    // 1 * id
    // 2 * id
    // ...
    // (count - 1) * id
    //
    // this reduces to (0 + 1 + 2 + ... + (count - 1)) * id
    //
    // and since count is always [0,9], we can use a tiny LUT for this which makes this whole
    // checksum essentially constant time
    total_prev * self.count as usize * self.id as usize
      + unsafe { *ADD_FACTORIAL_LUT.get_unchecked(self.count as usize) } * self.id as usize
  }
}

#[derive(Clone, Debug)]
struct MiniVec {
  pub len: u32,
  pub elements: [Slot; 6],
}

impl MiniVec {
  fn push(&mut self, item: Slot) {
    unsafe {
      *self.elements.get_unchecked_mut(self.len as usize) = item;
    }
    self.len += 1;
    debug_assert!(self.len as usize <= self.elements.len());
  }

  fn pop_front(&mut self) {
    for i in 1..self.len {
      unsafe {
        *self.elements.get_unchecked_mut(i as usize - 1) = self.elements[i as usize];
      }
    }
    self.len -= 1;
    return;
    // we should only ever mutate the vector once
    debug_assert!(self.elements[0].count != 0);
    // this is a nice trick I came up with to accomplish the equivalent
    self.elements[0].count = 0;
  }

  fn as_slice(&self) -> &[Slot] { unsafe { self.elements.get_unchecked(..self.len as usize) } }
}

const ADD_FACTORIAL_LUT: [usize; 11] = [
  0,
  0,
  1,
  2 + 1,
  3 + 2 + 1,
  4 + 3 + 2 + 1,
  5 + 4 + 3 + 2 + 1,
  6 + 5 + 4 + 3 + 2 + 1,
  7 + 6 + 5 + 4 + 3 + 2 + 1,
  8 + 7 + 6 + 5 + 4 + 3 + 2 + 1,
  9 + 8 + 7 + 6 + 5 + 4 + 3 + 2 + 1,
];

pub fn part2(input: &[u8]) -> usize {
  let (input, mut slots, mut empty_spaces) = parse_input(input);

  fn checksum(total_prev: &mut usize, slots: &[Slot], empty_space: usize) -> usize {
    debug_assert!(slots.len() <= 4);

    let mut sum = 0usize;
    for slot in slots {
      sum += slot.checksum(*total_prev);
      *total_prev += slot.count;
    }
    *total_prev += empty_space;
    sum
  }

  let mut start_span_ix_by_needed_size: [usize; 10] = [0; 10];
  for src_id in (0..input.len()).rev() {
    let src_count = input[src_id];

    let start_ix = start_span_ix_by_needed_size[src_count];
    if start_ix >= src_id {
      continue;
    }
    let dst_span_ix = empty_spaces[start_ix..src_id]
      .iter_mut()
      .enumerate()
      .find_map(|(i, &mut empty_space)| {
        if empty_space >= src_count {
          Some(start_ix + i)
        } else {
          None
        }
      });
    let Some(mut dst_span_ix) = dst_span_ix else {
      continue;
    };
    let dst_span = &mut slots[dst_span_ix];

    dst_span.push(Slot {
      count: src_count,
      id: src_id,
    });
    empty_spaces[dst_span_ix] -= src_count;

    while empty_spaces[dst_span_ix] < src_count {
      dst_span_ix += 1;
    }

    for i in src_count..10 {
      start_span_ix_by_needed_size[i] = start_span_ix_by_needed_size[i].max(dst_span_ix);
    }

    {
      let src_span = &mut slots[src_id];

      src_span.pop_front();

      empty_spaces[src_id] += src_count;
    }
    empty_spaces[src_id] -= src_count;
    empty_spaces[src_id - 1] += src_count;
  }

  let mut out = 0usize;
  let mut total_prev = 0usize;
  for (i, slot) in slots.iter().enumerate() {
    out += checksum(&mut total_prev, slot.as_slice(), empty_spaces[i]);
  }

  out
}

#[cfg(feature = "local")]
pub fn solve() {
  // let out = part1(INPUT);

  // println!("Part 1: {}", out);

  let out = part2(INPUT);

  println!("Part 2: {}", out);
}

pub fn run(input: &[u8]) -> impl Display { part2(input) }
