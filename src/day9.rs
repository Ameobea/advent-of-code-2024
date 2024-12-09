pub const INPUT: &'static str = include_str!("../inputs/day9.txt");

fn parse_input(input: &str) -> Vec<(usize, usize)> {
  let input = if input.len() % 2 == 1 {
    let mut input = input.to_owned();
    input.push('0');
    input
  } else {
    input.to_owned()
  };
  input
    .chars()
    .take_while(|c| c.is_numeric())
    .collect::<Vec<_>>()
    .array_chunks::<2>()
    .map(|[size, free]| {
      (
        size.to_string().parse().unwrap(),
        free.to_string().parse().unwrap(),
      )
    })
    .collect()
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

pub fn part2(input: &str) -> usize {
  let input = parse_input(input);

  let mut data_start_pos = Vec::new();
  let mut cur_start_pos = 0usize;
  for i in 0..input.len() {
    data_start_pos.push(cur_start_pos);
    cur_start_pos += input[i].0 + input[i].1;
  }

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
      inner: Box<(Span, Span)>,
    },
  }

  impl Span {
    fn checksum(&self) -> usize {
      match self {
        Span::Empty { .. } => 0,
        Span::Filled {
          total_prev,
          count,
          id,
        } => (0..*count).map(|i| (*total_prev + i) * *id).sum::<usize>(),
        Span::Split { inner, .. } => inner.0.checksum() + inner.1.checksum(),
      }
    }

    fn total_prev(&self) -> usize {
      match self {
        Span::Empty { total_prev, .. } => *total_prev,
        Span::Filled { total_prev, .. } => *total_prev,
        Span::Split { total_prev, .. } => *total_prev,
      }
    }
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

    fn span_is_valid(span: &mut Span, needed_space: usize) -> Option<&mut Span> {
      match span {
        Span::Empty {
          total_prev: _,
          count,
        } =>
          if *count >= needed_space {
            Some(span)
          } else {
            None
          },
        Span::Filled { .. } => None,
        Span::Split {
          total_prev: _,
          count,
          inner,
        } => {
          if *count < needed_space {
            return None;
          }
          if let Some(span) = span_is_valid(&mut inner.0, needed_space) {
            return Some(span);
          }
          if let Some(span) = span_is_valid(&mut inner.1, needed_space) {
            return Some(span);
          }
          None
        },
      }
    }

    if start_span_ix_by_needed_size[src_count] >= src_span_ix {
      continue;
    }
    let dst_span = spans[start_span_ix_by_needed_size[src_count]..src_span_ix]
      .iter_mut()
      .enumerate()
      .filter(|(_ix, span)| span.total_prev() < src_total_prev)
      .find_map(|(base_ix, span)| match span {
        Span::Empty {
          total_prev: _,
          count,
        } =>
          if *count >= src_count {
            Some((span, base_ix + start_span_ix_by_needed_size[src_count]))
          } else {
            None
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
          if let Some(span) = span_is_valid(&mut inner.0, src_count) {
            return Some((span, base_ix + start_span_ix_by_needed_size[src_count]));
          }
          if let Some(span) = span_is_valid(&mut inner.1, src_count) {
            return Some((span, base_ix + start_span_ix_by_needed_size[src_count]));
          }
          None
        },
      });
    let Some((dst_span, dst_span_ix)) = dst_span else {
      continue;
    };
    start_span_ix_by_needed_size[src_count] = dst_span_ix;

    let (free_space, total_prev) = match dst_span {
      Span::Empty {
        total_prev, count, ..
      } => (*count, *total_prev),
      _ => unreachable!(),
    };
    assert!(free_space >= src_count);
    if free_space == src_count {
      *dst_span = Span::Filled {
        count: src_count,
        id: src_id,
        total_prev,
      }
    } else {
      *dst_span = Span::Split {
        count: free_space,
        total_prev,
        inner: Box::new((
          Span::Filled {
            count: src_count,
            id: src_id,
            total_prev,
          },
          Span::Empty {
            total_prev: total_prev + src_count,
            count: free_space - src_count,
          },
        )),
      }
    }

    let src_span = &mut spans[src_id * 2];
    *src_span = Span::Empty {
      count: src_count,
      total_prev: src_total_prev,
    };
  }

  let mut out = 0usize;
  for span in spans {
    out += span.checksum();
  }

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
