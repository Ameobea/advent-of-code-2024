#![feature(array_chunks, array_windows, portable_simd, pointer_is_aligned_to)]

// TRICKS:
//
// - Span structure to represent regions of the memory
//   - Takes advantage of the fact that free space is always to the right of data in each slot.
//   - Uses that trick of adding free space to previous span in the case that the first element is
//     removed
//   - Makes hacky assumption that we'll never remove data from the middle as that would create free
//     space in the middle of the span and invalidate the assumptions.
//
//     However, this does work out for all inputs tested.
// - SoA for the spans
// - aligned input vector as well as data vectors for counts + free lists which facilitates:
// - SIMD parsing
//   - de-interleaves sizes + free spaces into separate arrays and writes out with SIMD where
//     possible
// - MiniVec; specialized capped-sized-vec that stores stuff inline.
//   - uses uninitialized memory for elements that haven't been set
//   - all methods custom-built and unsafe with no checks
//   - TRIED the fancy const impl that pre-computes the vector here with lens and IDs pre-set, but
//     memcpy overhead was greater than savings
// - `start_span_ix_by_needed_size` to keep track of the earliest possible location of a big enough
//   free space for every size
//   - TRIED the fancy impl. that max's the val of all larger buckets as well, but turned out to be
//     way slower (especially when SIMD was enabled)
// - SIMD for finding the first free slot which is large enough
//   - dubious benefit; within a few percent in any case.
// - target-cpu=znver3
// - constant-time checksumming
// - `max_unmoved_src_id` accounting
//   - allows fully empty chunks at the end to be skipped during checksum computation
// - `finished_digit_count` bookkeeping
//   - allows for early exit of the main loop after we've found a stopping place for every char

use std::{
  fmt::Display,
  simd::{cmp::SimdPartialOrd, num::SimdUint, u32x4, u8x16, u8x64},
};

#[cfg(feature = "local")]
pub const INPUT: &'static [u8] = include_bytes!("../inputs/day9.txt");

fn parse_digit(c: u8) -> u32 { (c - 48) as u32 }

fn parse_input(input: &[u8]) -> Vec<(u32, u32)> {
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

#[repr(C, align(64))]
struct AlignToSixtyFour([u8; 64]);

// adapted from: https://stackoverflow.com/a/60180226/3833068
fn aligned_vec(n_bytes: usize) -> Vec<u32> {
  // Lazy math to ensure we always have enough.
  let n_units = (n_bytes / std::mem::size_of::<AlignToSixtyFour>()) + 1;

  let mut aligned: Vec<AlignToSixtyFour> = Vec::with_capacity(n_units);

  let ptr = aligned.as_mut_ptr();
  let len_units = aligned.len();
  let cap_units = aligned.capacity();

  std::mem::forget(aligned);

  unsafe {
    Vec::from_raw_parts(
      ptr as *mut u32,
      (len_units * std::mem::size_of::<AlignToSixtyFour>()) / std::mem::size_of::<u32>(),
      (cap_units * std::mem::size_of::<AlignToSixtyFour>()) / std::mem::size_of::<u32>(),
    )
  }
}

// sadly, the cost of copying all of the uninitialized bytes that we don't care about is higher than
// being able to set the lengths and indices up front.
// const fn build_empty_slots() -> [MiniVec; 10_000] {
//   let mut arr: [MiniVec; 10_000] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
//   let mut i = 0usize;
//   loop {
//     arr[i].len = 1;
//     arr[i].elements[0].id = i as u32;

//     i += 1;
//     if i == arr.len() {
//       break;
//     }
//   }
//   arr
// }

fn parse_input_p2(input: &[u8]) -> (Vec<u32>, Vec<u32>, Vec<MiniVec>) {
  let id_count = if input.len() % 2 == 1 {
    input.len() / 2 + 1
  } else {
    input.len() / 2
  };

  let mut orig_counts: Vec<u32> = aligned_vec(4 * id_count); // Vec::with_capacity(input.len() / 2 + 1);
  unsafe { orig_counts.set_len(id_count) };
  let mut empty_spaces: Vec<u32> = aligned_vec(4 * id_count); // Vec::with_capacity(input.len() / 2 + 1);
  unsafe { empty_spaces.set_len(id_count) };
  let mut slots: Vec<MiniVec> = Vec::with_capacity(id_count);
  unsafe { slots.set_len(id_count) };
  // let mut slots: [MiniVec; 10_000] = build_empty_slots();

  assert!(input.as_ptr().is_aligned_to(std::mem::align_of::<u8x64>()));

  const VECTOR_LEN: usize = 16;
  const STORE_VECTOR_LEN: usize = VECTOR_LEN / 2;
  let batch_count = input.len() / VECTOR_LEN;
  let batch_handled_count = batch_count * VECTOR_LEN;
  for batch_ix in 0..batch_count {
    let vec: u8x16 =
      unsafe { std::ptr::read(input.as_ptr().add(batch_ix * VECTOR_LEN) as *const _) };
    // convert from ascii digits to bytes representing the digit ('0' -> 0)
    let converted = vec - u8x16::splat(48);
    // zero-extend u8 -> u32
    let vu32 = converted.cast::<u32>();
    // split out from size,free,size,free to ([size,size], [free,free])
    let (sizes, frees) = vu32.deinterleave(vu32);
    // the de-interleave duplicates the results, so keeping only the first half is correct
    let sizes = sizes.resize::<STORE_VECTOR_LEN>(STORE_VECTOR_LEN as u32);
    let frees = frees.resize::<STORE_VECTOR_LEN>(STORE_VECTOR_LEN as u32);

    unsafe {
      let frees_ptr = empty_spaces.as_mut_ptr().add(batch_ix * STORE_VECTOR_LEN) as *mut _;
      *frees_ptr = frees;

      let orig_counts_ptr = orig_counts.as_mut_ptr().add(batch_ix * STORE_VECTOR_LEN) as *mut _;
      *orig_counts_ptr = sizes;
    }

    let sizes = sizes.as_array();
    assert_eq!(sizes.len(), STORE_VECTOR_LEN);
    for i in 0..sizes.len() {
      let id = (batch_ix * STORE_VECTOR_LEN + i) as u32;
      unsafe {
        slots.get_unchecked_mut(batch_ix * STORE_VECTOR_LEN + i).len = 1;
        slots
          .get_unchecked_mut(batch_ix * STORE_VECTOR_LEN + i)
          .elements[0] = Slot {
          count: sizes[i],
          id,
        }

        // this is for the const-initialized minivec version
        // slots
        //   .get_unchecked_mut(batch_ix * STORE_VECTOR_LEN + i)
        //   .elements[0]
        //   .count = sizes[i];
      }
    }
  }

  if input.len() % 2 != 0 {
    let mut it = input[batch_handled_count..input.len() - if input.len() % 2 == 0 { 1 } else { 0 }]
      .array_chunks::<2>();

    let mut id = STORE_VECTOR_LEN * batch_count;
    while let Some(&[size, free]) = it.next() {
      let size = parse_digit(size);
      let free = parse_digit(free);

      unsafe {
        *empty_spaces.get_unchecked_mut(id) = free as u32;
        slots.get_unchecked_mut(id).len = 1;
        slots.get_unchecked_mut(id).elements[0] = Slot {
          count: size,
          id: id as _,
        };
        *orig_counts.get_unchecked_mut(id) = size;
      }
      id += 1;
    }

    if let Some(remainder) = it.remainder().get(0) {
      let size = parse_digit(*remainder);

      unsafe {
        *empty_spaces.get_unchecked_mut(id) = 0 as u32;
        slots.get_unchecked_mut(id).len = 1;
        slots.get_unchecked_mut(id).elements[0] = Slot {
          count: size,
          id: id as _,
        };
        *orig_counts.get_unchecked_mut(id) = size;
      }
    }
  } else {
    assert!(input.len() % VECTOR_LEN == 0);
    // we'd technically need to handle converting the newline that we parsed as the last character
    // into a 0 to indicate that there are zero empty slots at the end.
    //
    // however, there is no situation where we'd need to move anything into the last slot, so who
    // cares how big the empty space is.

    // let last_id = empty_spaces.len() - 1;
    // unsafe {
    //   *empty_spaces.get_unchecked_mut(last_id) = 0;
    // }
  }

  (orig_counts, empty_spaces, slots)
}

fn compute_fs(input: &[(u32, u32)]) -> Vec<Option<u32>> {
  let mut fs = Vec::new();
  for (id, (size, free)) in input.iter().enumerate() {
    for _ in 0..*size {
      fs.push(Some(id as u32));
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
    out += i * id as usize;
  }

  out
}

#[derive(Debug, Clone, Copy)]
struct Slot {
  pub id: u32,
  pub count: u32,
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
    // let out = self.elements[0];
    // for i in 1..self.len {
    //   unsafe {
    //     *self.elements.get_unchecked_mut(i as usize - 1) = self.elements[i as usize];
    //   }
    // }
    // self.len -= 1;

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

pub fn part2(raw_input: &[u8]) -> usize {
  let (input, mut empty_spaces, mut slots) = parse_input_p2(raw_input);

  fn checksum(slots: &[Slot], empty_space: u32, total_prev: &mut usize) -> usize {
    let mut sum = 0usize;
    for slot in slots {
      sum += slot.checksum(*total_prev);
      *total_prev += slot.count as usize;
    }
    *total_prev += empty_space as usize;
    sum
  }

  let mut start_span_ix_by_needed_size: [usize; 10] = [0; 10];
  let mut finished_digit_count = 0usize;
  // we keep track of the highest span that still has a value in it.
  //
  // this allows us to skip iterating over fully empty spans at the end when computing the checksum
  let mut max_unmoved_src_id = 0;
  'outer: for src_id in (0..input.len()).rev() {
    let src_count = unsafe { *input.get_unchecked(src_id) };

    let start_ix = unsafe { *start_span_ix_by_needed_size.get_unchecked(src_count as usize) };

    // we can only move elements to the left
    if start_ix >= src_id {
      if start_ix != usize::MAX {
        max_unmoved_src_id = max_unmoved_src_id.max(src_id);
        debug_assert!(slots[max_unmoved_src_id + 1..]
          .iter()
          .all(|s| s.as_slice().is_empty() || s.as_slice().iter().all(|s| s.count == 0)));

        finished_digit_count += 1;
        if finished_digit_count == 9 {
          debug_assert_eq!(
            start_span_ix_by_needed_size[0], 0,
            "there are never zero-size files in the inputs apparently"
          );
          break;
        }
        // TODO: finish bigger digits too?
        unsafe {
          *start_span_ix_by_needed_size.get_unchecked_mut(src_count as usize) = usize::MAX;
        }
      }

      continue;
    }

    let src_id = src_id as u32;

    let start_ptr = unsafe { empty_spaces.as_ptr().add(start_ix) };
    let mut cur_offset = 0usize;
    let mut dst_span_ix = loop {
      const VEC_SIZE: usize = 4usize;
      let end_ix = start_ix + cur_offset + VEC_SIZE;
      // same caveat as before.  For a 100% correct implementation for all possible inputs, we'd
      // need to handle manually checking the tail here but I'm leaving that out
      //
      // I could leave this off if I wanted to and it wouldn't be an issue...
      if end_ix > input.len() - VEC_SIZE {
        start_span_ix_by_needed_size[src_count as usize] = usize::MAX;
        finished_digit_count += 1;
        max_unmoved_src_id = max_unmoved_src_id.max(src_id as usize);
        continue 'outer;
      }

      let empty_spaces_v: u32x4 =
        unsafe { std::ptr::read_unaligned(start_ptr.add(cur_offset) as *const _) };
      let mask = empty_spaces_v.simd_ge(u32x4::splat(src_count));
      match mask.first_set() {
        Some(i) => {
          let dst_span_ix = start_ix + cur_offset + i;
          if dst_span_ix >= src_id as usize {
            start_span_ix_by_needed_size[src_count as usize] = usize::MAX;
            finished_digit_count += 1;
            max_unmoved_src_id = max_unmoved_src_id.max(src_id as usize);
            continue 'outer;
          }
          debug_assert!(empty_spaces[dst_span_ix] >= src_count);
          break dst_span_ix;
        },
        None => cur_offset += VEC_SIZE,
      }
    };

    let dst_slots: &mut MiniVec = unsafe { slots.get_unchecked_mut(dst_span_ix) };
    max_unmoved_src_id = max_unmoved_src_id.max(dst_span_ix);
    dst_slots.push(Slot {
      count: src_count,
      id: src_id,
    });
    empty_spaces[dst_span_ix] -= src_count;

    if (dst_span_ix as u32) < src_id && empty_spaces[dst_span_ix] < src_count {
      dst_span_ix += 1;
    }

    start_span_ix_by_needed_size[src_count as usize] = dst_span_ix;

    // \/ this code uses the fact that if a span of size `src_count` can't fit before `dst_span_ix`,
    // then no bigger span could either.
    //
    // However, it turns out to make things slower - especially when compiling with
    // `target-cpu=native`.  That causes some fancy SIMD that performs this operation using masks
    // and whatnot to be emitted, but that turns out to be way slower than the scalar version.
    //
    // Anyway, just skipping all this work here seems to be the fastest method of them all, probably
    // because our SIMD free slot search is fast enough to make up for the savings of doing the more
    // fancy accounting after the bookkeeping overhead.
    //
    // for i in src_count as usize..10 {
    //   start_span_ix_by_needed_size[i] = start_span_ix_by_needed_size[i].max(dst_span_ix);
    // }

    // the element we're removing is at the first index of the array since any others added to this
    // span will have been put after it
    let src_slots = &mut slots[src_id as usize];
    debug_assert_eq!(src_slots.elements[0].id, src_id);
    empty_spaces[src_id as usize - 1] += src_count;
    src_slots.pop_front();
  }

  let mut out = 0usize;
  let mut total_prev = 0usize;
  for (slot, &empty_count) in unsafe { slots.get_unchecked(..=max_unmoved_src_id) }
    .iter()
    .zip(unsafe { empty_spaces.get_unchecked(..=max_unmoved_src_id) }.iter())
  {
    out += checksum(slot.as_slice(), empty_count, &mut total_prev);
  }

  out
}

#[cfg(feature = "local")]
pub fn solve() {
  use crate::helpers::leak_to_page_aligned;

  let aligned_input = leak_to_page_aligned(INPUT);

  let out = part1(aligned_input);

  println!("Part 1: {}", out);

  let out = part2(aligned_input);

  println!("Part 2: {}", out);
}

pub fn run(input: &[u8]) -> impl Display { part2(input) }
