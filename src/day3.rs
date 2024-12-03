use std::simd::{cmp::SimdPartialEq, u8x16};

use smallvec::SmallVec;

const INPUT: &'static [u8] = include_bytes!("../inputs/day3.txt");

type Nums = SmallVec<[usize; 2]>;
type Cur = SmallVec<[u8; 3]>;

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

fn consume_num(nums: &mut Nums, cur: Cur) {
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
    _ => unreachable!(),
  };
  nums.push(num);
}

impl ParseState {
  pub fn next<const ENABLE_DO_STATE: bool>(
    self,
    sum: &mut usize,
    do_state: &mut bool,
    c: u8,
  ) -> Self {
    match self {
      ParseState::Fresh =>
        if c == 'm' as u8 {
          Self::M
        } else if c == 'd' as u8 {
          Self::D
        } else {
          Self::Fresh
        },
      ParseState::M =>
        if c == 'u' as u8 {
          Self::U
        } else {
          Self::Fresh
        },
      ParseState::U =>
        if c == 'l' as u8 {
          Self::L
        } else {
          Self::Fresh
        },
      ParseState::L =>
        if c == '(' as u8 {
          Self::Nums {
            nums: SmallVec::new(),
            cur: SmallVec::new(),
          }
        } else {
          Self::Fresh
        },
      ParseState::Nums { mut nums, mut cur } =>
      // if c.is_numeric() {
        if c >= '0' as u8 && c <= '9' as u8 {
          cur.push(c as u8);
          ParseState::Nums { nums, cur }
        } else if c == ',' as u8 {
          if cur.is_empty() {
            Self::Fresh
          } else {
            consume_num(&mut nums, cur);
            ParseState::Nums {
              nums,
              cur: SmallVec::new(),
            }
          }
        } else if c == ')' as u8 {
          if cur.is_empty() {
            Self::Fresh
          } else {
            consume_num(&mut nums, cur);
            if *do_state {
              // *sum += nums[0] * nums[1];
              *sum += unsafe { nums.get_unchecked(0) * nums.get_unchecked(1) };
            }
            Self::Fresh
          }
        } else {
          Self::Fresh
        },
      ParseState::D =>
        if c == 'o' as u8 {
          Self::O
        } else {
          Self::Fresh
        },
      ParseState::O =>
        if *do_state && c == 'n' as u8 {
          Self::N
        } else if !*do_state && c == '(' as u8 {
          Self::DoOpenParen
        } else {
          Self::Fresh
        },
      ParseState::DoOpenParen => {
        if c == ')' as u8 && ENABLE_DO_STATE {
          *do_state = true
        }
        Self::Fresh
      },
      ParseState::N =>
        if c == '\'' as u8 {
          Self::Apostrophe
        } else {
          Self::Fresh
        },
      ParseState::Apostrophe =>
        if c == 't' as u8 {
          Self::T
        } else {
          Self::Fresh
        },
      ParseState::T =>
        if c == '(' as u8 {
          Self::DontOpenParen
        } else {
          Self::Fresh
        },
      ParseState::DontOpenParen => {
        if c == ')' as u8 && ENABLE_DO_STATE {
          *do_state = false;
        }
        Self::Fresh
      },
    }
  }
}

pub fn parse_and_compute<const ENABLE_DO_STATE: bool>() -> usize {
  let mut sum = 0usize;
  let mut cur = ParseState::Fresh;
  let mut do_state = true;
  let mut char_ix = 0usize;

  while char_ix < INPUT.len() {
    let mut c = unsafe { *INPUT.get_unchecked(char_ix) };
    if matches!(cur, ParseState::Fresh) {
      // if we're not currently in the middle of parsing anything, scan ahead 16 chars and check to
      // see if they're all garbage characters that can be skipped.
      //
      // scanning 16 seems to be faster than 32, probably because the `mul()`s are so dense in the
      // actual inputs that it usually fails
      while char_ix < INPUT.len() - (16 + 1) {
        let vector =
          unsafe { u8x16::from_slice(std::slice::from_raw_parts(INPUT.as_ptr().add(char_ix), 16)) };
        let m_mask = vector.simd_eq(u8x16::splat('m' as u8));
        let d_mask = vector.simd_eq(u8x16::splat('d' as u8));
        let has_hit = (m_mask | d_mask).any();
        if has_hit {
          break;
        }
        char_ix += 16;
        c = unsafe { *INPUT.get_unchecked(char_ix) };
      }

      // scan through any remainder garbage ahead of time as a fast-path
      while c != 'd' as u8 && c != 'm' as u8 {
        char_ix += 1;
        if char_ix == INPUT.len() {
          return sum;
        }
        c = unsafe { *INPUT.get_unchecked(char_ix) };
      }
    }

    cur = cur.next::<ENABLE_DO_STATE>(&mut sum, &mut do_state, c);
    char_ix += 1;
  }

  sum
}

pub fn solve() {
  let out = parse_and_compute::<false>();
  println!("Part 1: {out}");

  let out = parse_and_compute::<true>();
  println!("Part 2: {out}");
}
