#![feature(array_chunks, array_windows, duration_constructors, portable_simd)]

use std::{
  fmt::Display,
  simd::{cmp::SimdPartialEq, u8x32},
};

pub const INPUT: &'static [u8] = include_bytes!("../inputs/day3.txt");

fn parse_char(c: u8) -> usize { (c - 48) as usize }

fn add_num(digits: &[usize]) -> usize {
  match digits.len() {
    0 => unreachable!(),
    1 => unsafe { *digits.get_unchecked(0) },
    2 => 10 * unsafe { *digits.get_unchecked(0) } + unsafe { *digits.get_unchecked(1) },
    3 =>
      100 * unsafe { *digits.get_unchecked(0) }
        + 10 * unsafe { *digits.get_unchecked(1) }
        + unsafe { *digits.get_unchecked(2) },
    _ => unreachable!(),
  }
}

const MUL: [u8; 4] = ['m' as u8, 'u' as u8, 'l' as u8, '(' as u8];
const DONT: [u8; 7] = [
  'd' as u8, 'o' as u8, 'n' as u8, '\'' as u8, 't' as u8, '(' as u8, ')' as u8,
];
const DO: [u8; 4] = ['d' as u8, 'o' as u8, '(' as u8, ')' as u8];

pub fn parse_and_compute<const ENABLE_DO_STATE: bool>(input: &[u8]) -> usize {
  let mut sum = 0usize;
  let mut do_state = true;
  let mut char_ix = 0usize;

  loop {
    let mut c;

    if char_ix < input.len() - (32 + 1) {
      let vector =
        unsafe { u8x32::from_slice(std::slice::from_raw_parts(input.as_ptr().add(char_ix), 32)) };

      let combined_mask = if ENABLE_DO_STATE {
        let d_mask = vector.simd_eq(u8x32::splat('d' as u8));
        if !do_state {
          d_mask
        } else {
          let m_mask = vector.simd_eq(u8x32::splat('m' as u8));
          m_mask | d_mask
        }
      } else {
        vector.simd_eq(u8x32::splat('m' as u8))
      };
      let hit_ix = match combined_mask.first_set() {
        Some(hit_ix) => hit_ix,
        None => {
          char_ix += 32;
          continue;
        },
      };

      char_ix += hit_ix;
      c = if ENABLE_DO_STATE {
        unsafe { *input.get_unchecked(char_ix) }
      } else {
        'm' as u8
      };
    } else {
      c = unsafe { *input.get_unchecked(char_ix) };
      while c != 'm' as u8 && (!ENABLE_DO_STATE || c != 'd' as u8) {
        char_ix += 1;
        if char_ix >= input.len() {
          return sum;
        }
        c = unsafe { *input.get_unchecked(char_ix) };
      }
    }

    if c == 'm' as u8 {
      if input.get(char_ix..char_ix + MUL.len()) == Some(&MUL) {
        char_ix += MUL.len();

        // don't bother parsing out this mul if we're not doing
        if ENABLE_DO_STATE && !do_state {
          continue;
        }

        c = unsafe { *input.get_unchecked(char_ix) };

        // try to fastpath consume nums
        let mut first_num = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        let mut cur: [usize; 3] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        let mut digit_ix = 0usize;

        'consume_nums: loop {
          if c >= '0' as u8 && c <= '9' as u8 {
            unsafe {
              *cur.get_unchecked_mut(digit_ix) = parse_char(c);
            }
            digit_ix += 1;
          } else if c == ',' as u8 {
            if digit_ix == 0 {
              break 'consume_nums;
            }
            first_num = add_num(unsafe { cur.get_unchecked(..digit_ix) });
            digit_ix = 0;
          } else if c == ')' as u8 {
            if digit_ix == 0 {
              break 'consume_nums;
            } else {
              let num = add_num(unsafe { cur.get_unchecked(..digit_ix) });
              sum += first_num * num;

              char_ix += 1;
              if char_ix >= input.len() {
                return sum;
              }
              break 'consume_nums;
            }
          } else {
            break;
          }

          char_ix += 1;
          if char_ix >= input.len() {
            return sum;
          }
          c = unsafe { *input.get_unchecked(char_ix) };
        }

        continue;
      } else {
        char_ix += 1;
        continue;
      }
    } else if ENABLE_DO_STATE && c == 'd' as u8 {
      if do_state && input.get(char_ix + 1..char_ix + DONT.len()) == Some(&DONT[1..]) {
        do_state = false;
        char_ix += DONT.len();
        continue;
      } else if !do_state && input.get(char_ix..char_ix + DO.len()) == Some(&DO) {
        do_state = true;
        char_ix += DO.len();
        continue;
      } else {
        char_ix += 1;
        continue;
      }
    }

    break;
  }

  sum
}

pub fn solve() {
  let out = parse_and_compute::<false>(INPUT);
  println!("Part 1: {out}");

  let out = parse_and_compute::<true>(INPUT);
  println!("Part 2: {out}");
}

pub fn run(input: &[u8]) -> impl Display { parse_and_compute::<false>(input) }
