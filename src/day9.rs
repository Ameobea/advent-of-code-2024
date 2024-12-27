#![feature(array_chunks, array_windows, portable_simd)]

use std::fmt::Display;

use smallvec::SmallVec;

#[cfg(feature = "local")]
pub const INPUT: &'static [u8] = include_bytes!("../inputs/day9.txt");

fn parse_digit(c: u8) -> usize { (c - 48) as usize }

fn parse_input(input: &[u8]) -> Vec<(usize, usize)> {
  let mut it = input[..input.len() - if input.len() % 2 == 0 { 1 } else { 0 }].array_chunks::<2>();

  let mut out = Vec::with_capacity(20_002 / 2);
  while let Some(&[size, free]) = it.next() {
    out.push((parse_digit(size), parse_digit(free)));
  }

  if let Some(remainder) = it.remainder().get(0) {
    out.push((parse_digit(*remainder), 0));
  }

  out
}

fn compute_fs(input: &[(usize, usize)]) -> Vec<Option<usize>> {
  let mut fs = Vec::new();
  for (id, (size, free)) in input.iter().enumerate() {
    for _ in 0..*size {
      fs.push(Some(id));
    }
    for _ in 0..*free {
      fs.push(None);
    }
  }

  fs
}

pub fn part1(input: &[u8]) -> usize {
  let input = parse_input(input);
  let mut fs = compute_fs(&input);

  let mut dst_ix = 0usize;
  for src_ix in (0..fs.len()).rev() {
    let Some(id) = fs[src_ix] else { continue };

    if dst_ix >= src_ix {
      break;
    }

    while fs[dst_ix].is_some() {
      dst_ix += 1;
      if dst_ix >= src_ix {
        break;
      }
    }

    fs[dst_ix] = Some(id);
    fs[src_ix] = None;
    dst_ix += 1;
    if dst_ix >= src_ix {
      break;
    }
  }

  let mut out = 0usize;
  for i in 0..fs.len() {
    let Some(id) = fs[i] else {
      continue;
    };
    out += i * id;
  }

  out
}

pub fn part2(input: &[u8]) -> usize {
  let input = parse_input(input);

  let mut data_start_pos = Vec::new();
  let mut cur_start_pos = 0usize;
  for i in 0..input.len() {
    data_start_pos.push(cur_start_pos);
    cur_start_pos += input[i].0 + input[i].1;
  }

  #[derive(Debug)]
  struct Slot {
    pub id: usize,
    pub count: usize,
  }

  impl Slot {
    fn checksum(&self, total_prev: usize) -> usize {
      (0..self.count)
        .map(|i| (total_prev + i) * self.id)
        .sum::<usize>()
    }
  }

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

  let mut slots: Vec<SmallVec<[Slot; 4]>> = Vec::with_capacity(input.len());
  let mut empty_spaces = Vec::with_capacity(input.len());
  for (id, &(count, free)) in input.iter().enumerate() {
    slots.push(smallvec::smallvec![Slot { id, count }]);
    empty_spaces.push(free);
  }

  let mut start_span_ix_by_needed_size: [usize; 10] = [0; 10];
  for src_id in (0..input.len()).rev() {
    let (src_count, _) = input[src_id];

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

    let mut was_first = false;
    {
      let src_span = &mut slots[src_id];

      if src_span.last().unwrap().id == src_id {
        src_span.pop();
      } else if src_span[0].id == src_id {
        src_span.remove(0);
        was_first = true
      }
      empty_spaces[src_id] += src_count;
    }
    if was_first {
      empty_spaces[src_id] -= src_count;
      empty_spaces[src_id - 1] += src_count;
    }
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
  let out = part1(INPUT);

  println!("Part 1: {}", out);

  let out = part2(INPUT);

  println!("Part 2: {}", out);
}

pub fn run(input: &[u8]) -> impl Display { part2(input) }
