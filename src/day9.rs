#![feature(array_chunks, array_windows, portable_simd)]

use std::fmt::Display;

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

pub fn part2(input: &[u8]) -> usize {
  let input = parse_input(input);

  let mut data_start_pos = Vec::new();
  let mut cur_start_pos = 0usize;
  for i in 0..input.len() {
    data_start_pos.push(cur_start_pos);
    cur_start_pos += input[i].0 + input[i].1;
  }

  let mut leaf_spans = Vec::with_capacity(input.len());

  #[derive(Debug)]
  enum Span {
    Empty {
      total_prev: usize,
      count: usize,
    },
    Filled {
      total_prev: usize,
      count: usize,
      id: usize,
    },
    Split {
      total_prev: usize,
      count: usize,
      inner: (usize, usize),
    },
  }

  impl Span {
    fn checksum_recursive(&self, leaf_spans: &[Span]) -> usize {
      match self {
        Span::Empty { .. } => 0,
        Span::Filled {
          total_prev,
          count,
          id,
        } => (0..*count).map(|i| (*total_prev + i) * *id).sum::<usize>(),
        Span::Split { inner, .. } =>
          leaf_spans[inner.0].checksum_recursive(leaf_spans)
            + leaf_spans[inner.1].checksum_recursive(leaf_spans),
      }
    }

    fn checksum(&self, leaf_spans: &[Span]) -> usize {
      match self {
        Span::Empty { .. } => 0,
        Span::Filled {
          total_prev,
          count,
          id,
        } => (0..*count).map(|i| (*total_prev + i) * *id).sum::<usize>(),
        Span::Split { inner, .. } =>
          leaf_spans[inner.0].checksum_recursive(leaf_spans)
            + leaf_spans[inner.1].checksum_recursive(leaf_spans),
      }
    }

    fn total_prev(&self) -> usize {
      match self {
        Span::Empty { total_prev, .. } => *total_prev,
        Span::Filled { total_prev, .. } => *total_prev,
        Span::Split { total_prev, .. } => *total_prev,
      }
    }

    fn to_vec(&self, leaf_spans: &[Span]) -> Vec<isize> {
      match self {
        Span::Empty { count, .. } => vec![-1; *count],
        Span::Filled { count, id, .. } => vec![*id as _; *count],
        Span::Split { inner, .. } => {
          let mut out = Vec::new();
          out.extend(leaf_spans[inner.0].to_vec(leaf_spans));
          out.extend(leaf_spans[inner.1].to_vec(leaf_spans));
          out
        },
      }
    }
  }

  #[derive(Debug)]
  enum SpanIx {
    Root(usize),
    Leaf(usize),
  }

  let mut spans = Vec::with_capacity(input.len() * 2);
  let mut total_prev = 0usize;
  for (id, &(count, free)) in input.iter().enumerate() {
    spans.push(Span::Filled {
      total_prev,
      count,
      id,
    });
    total_prev += count;
    spans.push(Span::Empty {
      total_prev,
      count: free,
    });
    total_prev += free;
  }

  let mut start_span_ix_by_needed_size: [usize; 10] = [0; 10];
  for src_id in (0..input.len()).rev() {
    let (src_count, _) = input[src_id];

    let src_span_ix = src_id * 2;
    let src_total_prev = spans[src_span_ix].total_prev();

    fn span_is_valid<'a>(
      leaf_spans: &[Span],
      leaf_span_ix: usize,
      needed_space: usize,
    ) -> Option<usize> {
      match &leaf_spans[leaf_span_ix] {
        Span::Empty {
          total_prev: _,
          count,
        } =>
          if *count >= needed_space {
            return Some(leaf_span_ix);
          } else {
            return None;
          },
        Span::Filled { .. } => return None,
        Span::Split {
          total_prev: _,
          count,
          inner,
        } => {
          if *count < needed_space {
            return None;
          }

          if let Some(span) = span_is_valid(leaf_spans, inner.0, needed_space) {
            return Some(span);
          }
          if let Some(span) = span_is_valid(leaf_spans, inner.1, needed_space) {
            return Some(span);
          }
        },
      };

      None
    }

    if start_span_ix_by_needed_size[src_count] >= src_span_ix {
      continue;
    }
    let dst_span = spans[start_span_ix_by_needed_size[src_count]..src_span_ix]
      .iter_mut()
      .enumerate()
      .find_map(|(base_ix, span)| match span {
        Span::Empty {
          total_prev: _,
          count,
        } => {
          let root_span_ix = base_ix + start_span_ix_by_needed_size[src_count];
          if *count >= src_count {
            Some((SpanIx::Root(root_span_ix), root_span_ix))
          } else {
            None
          }
        },
        Span::Filled { .. } => None,
        Span::Split {
          total_prev: _,
          count,
          inner,
        } => {
          if *count < src_count {
            return None;
          }
          // \/ we can skip this check since we only ever store empty slots on the right side of the
          // tree   if let Some(leaf_span_ix) = span_is_valid(&mut leaf_spans, inner.0,
          // src_count) {     return Some((
          //       SpanIx::Leaf(leaf_span_ix),
          //       base_ix + start_span_ix_by_needed_size[src_count],
          //     ));
          //   }
          if let Some(leaf_span_ix) = span_is_valid(&mut leaf_spans, inner.1, src_count) {
            let root_span_ix = base_ix + start_span_ix_by_needed_size[src_count];
            return Some((SpanIx::Leaf(leaf_span_ix), root_span_ix));
          }

          None
        },
      });
    let Some((dst_span_ix, mut dst_root_span_ix)) = dst_span else {
      continue;
    };

    let dst_span = match dst_span_ix {
      SpanIx::Root(ix) => &mut spans[ix],
      SpanIx::Leaf(ix) => &mut leaf_spans[ix],
    };
    let (free_space, total_prev) = match dst_span {
      Span::Empty {
        total_prev, count, ..
      } => (*count, *total_prev),
      _ => unreachable!(),
    };
    if free_space == src_count {
      *dst_span = Span::Filled {
        count: src_count,
        id: src_id,
        total_prev,
      };

      dst_root_span_ix += 1;
    } else {
      *dst_span = Span::Split {
        count: free_space,
        total_prev,
        inner: (0, 0),
      };

      leaf_spans.push(Span::Filled {
        count: src_count,
        id: src_id,
        total_prev,
      });
      let new_free_space = free_space - src_count;
      leaf_spans.push(Span::Empty {
        total_prev: total_prev + src_count,
        count: new_free_space,
      });
      let new_inner = (leaf_spans.len() - 2, leaf_spans.len() - 1);

      let dst_span = match dst_span_ix {
        SpanIx::Root(ix) => &mut spans[ix],
        SpanIx::Leaf(ix) => &mut leaf_spans[ix],
      };

      match dst_span {
        Span::Split { inner, .. } => *inner = new_inner,
        _ => unreachable!(),
      }

      if new_free_space < src_count {
        dst_root_span_ix += 1;
      }
    }

    for i in src_count..10 {
      start_span_ix_by_needed_size[i] = start_span_ix_by_needed_size[i].max(dst_root_span_ix);
    }

    let src_span = &mut spans[src_id * 2];
    *src_span = Span::Empty {
      count: src_count,
      total_prev: src_total_prev,
    };
  }

  let mut out = 0usize;
  for span in &spans {
    out += span.checksum(&leaf_spans);
  }

  //   let mut vals = Vec::new();
  //   for span in &spans {
  //     vals.extend(span.to_vec(&leaf_spans));
  //   }
  //   println!("{:?}", vals);

  out
}

#[cfg(feature = "local")]
pub fn solve() {
  let input = parse_input(INPUT);
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

  println!("Part 1: {}", out);

  let out = part2(INPUT);

  println!("Part 2: {}", out);
}

pub fn run(input: &[u8]) -> impl Display { part2(input) }
