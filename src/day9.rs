pub const INPUT: &'static str = include_str!("../inputs/day9.txt");

fn parse_input(input: &str) -> (Vec<(usize, usize)>, Vec<Option<usize>>) {
  let input = if input.len() % 2 == 1 {
    let mut input = input.to_owned();
    input.push('0');
    input
  } else {
    input.to_owned()
  };
  let input: Vec<_> = input
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
    .collect();

  let mut fs = Vec::new();
  for (id, (size, free)) in input.iter().enumerate() {
    for _ in 0..*size {
      fs.push(Some(id));
    }
    for _ in 0..*free {
      fs.push(None);
    }
  }

  (input, fs)
}

#[cfg(feature = "local")]
pub fn solve() {
  let (_input, mut fs) = parse_input(INPUT);

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

  let (input, mut fs) = parse_input(INPUT);

  let mut data_start_pos = Vec::new();
  let mut cur_start_pos = 0usize;
  for i in 0..input.len() {
    data_start_pos.push(cur_start_pos);
    cur_start_pos += input[i].0 + input[i].1;
  }

  //   enum Span {
  //     Empty {
  //       total_prev: usize,
  //       count: usize,
  //     },
  //     Filled {
  //       total_prev: usize,
  //       count: usize,
  //       id: usize,
  //     },
  //     Split {
  //       total_prev: usize,
  //       count: usize,
  //       inner: Box<(Span, Span)>,
  //     },
  //   }

  //   let mut spans = Vec::with_capacity(input.len() * 2);
  //   let mut total_prev = 0usize;
  //   for (id, &(count, free)) in input.iter().enumerate() {
  //     spans.push(Span::Filled {
  //       total_prev,
  //       count,
  //       id,
  //     });
  //     total_prev += count;
  //     spans.push(Span::Empty {
  //       total_prev,
  //       count: free,
  //     });
  //     total_prev += count;
  //   }

  for src_id in (0..input.len()).rev() {
    let (count, _) = input[src_id];
    let mut dst_ix = None;
    for i in 0..fs.len() {
      if i > data_start_pos[src_id] {
        break;
      }
      match fs.get(i..i + count) {
        None => continue,
        Some(slice) =>
          if slice.iter().any(|s| s.is_some()) {
            continue;
          } else {
            dst_ix = Some(i);
            break;
          },
      };
    }
    let Some(dst_ix) = dst_ix else {
      continue;
    };

    let data_span = data_start_pos[src_id]..data_start_pos[src_id] + count;
    for i in data_span {
      assert_eq!(fs[i], Some(src_id));
      fs[i] = None;
    }

    for i in dst_ix..dst_ix + count {
      assert!(fs[i].is_none());
      fs[i] = Some(src_id);
    }
  }

  let mut out = 0usize;
  for i in 0..fs.len() {
    let Some(id) = fs[i] else {
      continue;
    };
    out += i * id;
  }

  println!("Part 2: {}", out);
}
