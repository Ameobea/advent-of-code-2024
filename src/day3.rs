use std::simd::{cmp::SimdPartialEq, u8x32};

use smallvec::SmallVec;

pub const INPUT: &'static [u8] = include_bytes!("../inputs/day3.txt");

type Nums = SmallVec<[usize; 2]>;
type Cur = SmallVec<[u8; 3]>;

#[derive(Debug)]
enum ParseState {
  Fresh,
  M,
  U,
  L,
  Nums { nums: Nums, cur: Cur },
  D,
  O,
  DoOpenParen,
  N,
  Apostrophe,
  T,
  DontOpenParen,
}

fn parse_char(c: u8) -> usize { (c - 48) as usize }

fn consume_num(nums: &mut Nums, cur: &Cur) {
  let num = match cur.len() {
    0 => unreachable!(),
    1 => parse_char(unsafe { *cur.get_unchecked(0) }),
    2 =>
      10 * parse_char(unsafe { *cur.get_unchecked(0) })
        + parse_char(unsafe { *cur.get_unchecked(1) }),
    3 =>
      100 * parse_char(unsafe { *cur.get_unchecked(0) })
        + 10 * parse_char(unsafe { *cur.get_unchecked(1) })
        + parse_char(unsafe { *cur.get_unchecked(2) }),
    other => unreachable!("Unexpected digit count: {other}"),
  };
  nums.push(num);
}

const MUL: [u8; 4] = ['m' as u8, 'u' as u8, 'l' as u8, '(' as u8];
const DONT: [u8; 7] = [
  'd' as u8, 'o' as u8, 'n' as u8, '\'' as u8, 't' as u8, '(' as u8, ')' as u8,
];
const DO: [u8; 4] = ['d' as u8, 'o' as u8, '(' as u8, ')' as u8];

pub fn parse_and_compute<const ENABLE_DO_STATE: bool>(input: &[u8]) -> usize {
  let mut sum = 0usize;
  let mut cur = ParseState::Fresh;
  let mut do_state = true;
  let mut char_ix = 0usize;

  loop {
    let mut c = unsafe { *input.get_unchecked(char_ix) };
    if matches!(cur, ParseState::Fresh) {
      // if we're not currently in the middle of parsing anything, the only thing we can start with
      // is `d` or `m`.
      //
      // So, scan ahead through the input and find the first instance of either of those characters,
      // then jump to it.
      if char_ix < input.len() - (32 + 1) {
        let vector =
          unsafe { u8x32::from_slice(std::slice::from_raw_parts(input.as_ptr().add(char_ix), 32)) };

        let m_mask = vector.simd_eq(u8x32::splat('m' as u8));
        let combined_mask = if ENABLE_DO_STATE {
          let d_mask = vector.simd_eq(u8x32::splat('d' as u8));
          m_mask | d_mask
        } else {
          m_mask
        };
        let hit_ix = match combined_mask.first_set() {
          Some(hit_ix) => hit_ix,
          None => {
            char_ix += 32;
            continue;
          },
        };

        char_ix += hit_ix;
        c = unsafe { *input.get_unchecked(char_ix) };
      } else {
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
          if !do_state {
            continue;
          }

          c = unsafe { *input.get_unchecked(char_ix) };

          // try to fastpath consume nums
          let mut nums = Nums::default();
          let mut cur = Cur::default();

          'consume_nums: loop {
            if c >= '0' as u8 && c <= '9' as u8 {
              cur.push(c);
            } else if c == ',' as u8 {
              if cur.is_empty() {
                break 'consume_nums;
              }
              consume_num(&mut nums, &cur);
              cur = Default::default();
            } else if c == ')' as u8 {
              if cur.is_empty() {
                break 'consume_nums;
              } else {
                consume_num(&mut nums, &cur);
                debug_assert_eq!(nums.len(), 2);
                sum += unsafe { nums.get_unchecked(0) * nums.get_unchecked(1) };

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

    if char_ix >= input.len() {
      break;
    }

    dbg!(&cur, char_ix);
    char_ix += 1;

    if char_ix >= input.len() {
      break;
    }
  }

  sum
}

pub fn solve() {
  let out = parse_and_compute::<false>(INPUT);
  println!("Part 1: {out}");

  let out = parse_and_compute::<true>(INPUT);
  println!("Part 2: {out}");
}
