pub const INPUT: &'static str = include_str!("../inputs/day15.txt");

#[derive(PartialEq, Debug, Clone, Copy)]
enum Slot {
  Empty,
  Wall,
  Box,
  BoxStart,
  BoxEnd,
}

impl Slot {
  fn to_c(&self) -> char {
    match self {
      Slot::Empty => '.',
      Slot::Wall => '#',
      Slot::Box => 'O',
      Slot::BoxStart => '[',
      Slot::BoxEnd => ']',
    }
  }
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum Move {
  U,
  D,
  L,
  R,
}

impl Move {
  fn get_target(&self, p: (usize, usize)) -> (usize, usize) {
    match self {
      Move::U => (p.0, p.1 - 1),
      Move::D => (p.0, p.1 + 1),
      Move::L => (p.0 - 1, p.1),
      Move::R => (p.0 + 1, p.1),
    }
  }
}

impl From<char> for Move {
  fn from(value: char) -> Self {
    match value {
      '<' => Self::L,
      '>' => Self::R,
      '^' => Self::U,
      'v' => Self::D,
      _ => unreachable!(),
    }
  }
}

fn gps_coord(coord: (usize, usize)) -> usize { 100 * coord.1 + coord.0 }

fn parse_input(input: &str) -> (Vec<Vec<Slot>>, Vec<Move>, (usize, usize)) {
  let (fir, sec) = input.split_once("\n\n").unwrap();

  let mut grid = Vec::new();
  let mut moves = Vec::new();
  let mut bot_pos = (0, 0);

  for line in fir.lines() {
    let mut out = Vec::new();
    for c in line.chars() {
      match c {
        '#' => out.push(Slot::Wall),
        '.' => out.push(Slot::Empty),
        'O' => out.push(Slot::Box),
        '@' => {
          bot_pos = (out.len(), grid.len());
          out.push(Slot::Empty)
        },
        _ => panic!("{c}"),
      }
    }
    grid.push(out);
  }

  for c in sec.chars() {
    if c == '\n' {
      continue;
    }

    moves.push(Move::from(c))
  }

  (grid, moves, bot_pos)
}

fn print_grid(grid: &[Vec<Slot>], bot_pos: (usize, usize)) {
  // std::thread::sleep(Duration::from_millis(10));
  // print!("{esc}[2J{esc}[1;1H", esc = 27 as char);

  for (y, l) in grid.iter().enumerate() {
    let mut l = l.iter().map(|s| s.to_c()).collect::<String>();
    if y == bot_pos.1 {
      l = l
        .chars()
        .enumerate()
        .map(|(c_ix, c)| if c_ix == bot_pos.0 { '@' } else { c })
        .collect();
    }
    println!("{l}");
  }
  println!("\n");
}

fn parse_input_p2(input: &str) -> (Vec<Vec<Slot>>, Vec<Move>, (usize, usize)) {
  let (fir, sec) = input.split_once("\n\n").unwrap();

  let mut grid = Vec::new();
  let mut moves = Vec::new();
  let mut bot_pos = (0, 0);

  for line in fir.lines() {
    let mut out = Vec::new();
    for c in line.chars() {
      match c {
        '#' => {
          out.push(Slot::Wall);
          out.push(Slot::Wall);
        },
        '.' => {
          out.push(Slot::Empty);
          out.push(Slot::Empty);
        },
        'O' => {
          out.push(Slot::BoxStart);
          out.push(Slot::BoxEnd);
        },
        '@' => {
          bot_pos = (out.len(), grid.len());
          out.push(Slot::Empty);
          out.push(Slot::Empty)
        },
        _ => panic!("{c}"),
      }
    }
    grid.push(out);
  }

  for c in sec.chars() {
    if c == '\n' {
      continue;
    }

    moves.push(Move::from(c))
  }

  (grid, moves, bot_pos)
}

pub fn solve() {
  let (mut grid, moves, mut bot_pos) = parse_input(INPUT);

  'outer: for m in moves {
    let start_target = m.get_target(bot_pos);
    if grid[start_target.1][start_target.0] == Slot::Empty {
      bot_pos = start_target;
      continue;
    } else if grid[start_target.1][start_target.0] == Slot::Wall {
      continue;
    }

    let mut cur_target = start_target;
    let mut box_tiles = Vec::new();
    while grid[cur_target.1][cur_target.0] == Slot::Box {
      box_tiles.push(cur_target);
      cur_target = m.get_target(cur_target);
      if grid[cur_target.1][cur_target.0] == Slot::Wall {
        continue 'outer;
      }
    }

    assert_eq!(grid[cur_target.1][cur_target.0], Slot::Empty);

    grid[cur_target.1][cur_target.0] = Slot::Box;
    for &coord in &box_tiles[1..] {
      grid[coord.1][coord.0] = Slot::Box;
    }
    grid[box_tiles[0].1][box_tiles[0].0] = Slot::Empty;
    bot_pos = start_target;
  }

  print_grid(&grid, bot_pos);

  let mut out = 0usize;
  for y in 0..grid.len() {
    for x in 0..grid[0].len() {
      if grid[y][x] == Slot::Box {
        out += gps_coord((x, y));
      }
    }
  }

  println!("Part 1: {out}");

  let (mut grid, moves, mut bot_pos) = parse_input_p2(INPUT);

  'outer: for m in moves {
    // print_grid(&grid, bot_pos);

    for y in 0..grid.len() {
      for x in 0..grid[0].len() {
        if grid[y][x] == Slot::BoxStart {
          assert_eq!(grid[y][x + 1], Slot::BoxEnd);
          assert_ne!((x, y), bot_pos);
        } else if grid[y][x] == Slot::BoxEnd {
          assert_eq!(grid[y][x - 1], Slot::BoxStart);
          assert_ne!((x, y), bot_pos);
        }
      }
    }

    let start_target = m.get_target(bot_pos);
    if grid[start_target.1][start_target.0] == Slot::Empty {
      bot_pos = start_target;
      continue;
    } else if grid[start_target.1][start_target.0] == Slot::Wall {
      continue;
    }

    if m == Move::L || m == Move::R {
      let mut cur_target = start_target;
      let mut box_count = 0usize;
      while grid[cur_target.1][cur_target.0] == Slot::BoxStart
        || grid[cur_target.1][cur_target.0] == Slot::BoxEnd
      {
        box_count += 1;
        cur_target = m.get_target(cur_target);
        if grid[cur_target.1][cur_target.0] == Slot::Wall {
          continue 'outer;
        }
      }

      assert_eq!(grid[cur_target.1][cur_target.0], Slot::Empty);

      if m == Move::L {
        let from_slice =
          grid[start_target.1][start_target.0 - box_count + 1..start_target.0 + 1].to_vec();
        assert_eq!(from_slice[0], Slot::BoxStart);
        grid[start_target.1][start_target.0 - box_count..start_target.0]
          .copy_from_slice(from_slice.as_slice());
      } else {
        let from_slice = grid[start_target.1][start_target.0..start_target.0 + box_count].to_vec();
        assert_eq!(from_slice[0], Slot::BoxStart);
        grid[start_target.1][start_target.0 + 1..start_target.0 + 1 + box_count]
          .copy_from_slice(from_slice.as_slice());
      };

      grid[start_target.1][start_target.0] = Slot::Empty;
      bot_pos = start_target;

      continue 'outer;
    }

    let mut frontier_box_coords = Vec::new();
    if grid[start_target.1][start_target.0] == Slot::BoxStart
      || grid[start_target.1][start_target.0] == Slot::BoxEnd
    {
      frontier_box_coords.push(if grid[start_target.1][start_target.0] == Slot::BoxStart {
        (start_target, (start_target.0 + 1, start_target.1))
      } else {
        ((start_target.0 - 1, start_target.1), start_target)
      });
    }

    let mut valid_box_coords = Vec::new();
    // let mut invalid_box_coords = Vec::new();
    // let mut pushers_by_pushee: FxHashMap<(usize, usize), (usize, usize)> = FxHashMap::default();

    let mut next_cur_box_top_coords = Vec::new();
    let next_y = |y: usize| if m == Move::D { y + 1 } else { y - 1 };
    while !frontier_box_coords.is_empty() {
      for bt_coords in frontier_box_coords {
        for bt_coord in [bt_coords.0, bt_coords.1] {
          assert!(
            grid[bt_coord.1][bt_coord.0] == Slot::BoxStart
              || grid[bt_coord.1][bt_coord.0] == Slot::BoxEnd
          );

          let next = (bt_coord.0, next_y(bt_coord.1));
          if grid[next.1][next.0] == Slot::Wall {
            continue 'outer;
          } else if grid[next.1][next.0] == Slot::Empty {
            continue;
          } else if grid[next.1][next.0] == Slot::BoxStart || grid[next.1][next.0] == Slot::BoxEnd {
            if grid[next.1][next.0] == Slot::BoxStart {
              next_cur_box_top_coords.push((next, (next.0 + 1, next.1)));
            } else {
              next_cur_box_top_coords.push((next, (next.0 - 1, next.1)));
            }
          } else {
            unreachable!()
          }
        }

        valid_box_coords.push(bt_coords.0);
        valid_box_coords.push(bt_coords.1);
      }

      frontier_box_coords = next_cur_box_top_coords.clone();
      next_cur_box_top_coords = Vec::new();
    }

    assert!(!valid_box_coords.is_empty());

    let mut min_y = usize::MAX;
    let mut max_y = usize::MIN;
    let vals = valid_box_coords
      .iter()
      .map(|c| {
        min_y = min_y.min(c.1);
        max_y = max_y.max(c.1);
        grid[c.1][c.0]
      })
      .collect::<Vec<_>>();

    if m == Move::D {
      for src_y in (min_y..=max_y).rev() {
        let to_move = valid_box_coords
          .iter()
          .enumerate()
          .filter(|(_i, c)| c.1 == src_y)
          .map(|(i, c)| (c, vals[i]));
        for (coord, val) in to_move {
          grid[coord.1 + 1][coord.0] = val;
          grid[coord.1][coord.0] = Slot::Empty;
        }
      }
    } else if m == Move::U {
      for src_y in min_y..=max_y {
        let to_move = valid_box_coords
          .iter()
          .enumerate()
          .filter(|(_i, c)| c.1 == src_y)
          .map(|(i, c)| (c, vals[i]));
        for (coord, val) in to_move {
          grid[coord.1 - 1][coord.0] = val;
          grid[coord.1][coord.0] = Slot::Empty;
        }
      }
    } else {
      unreachable!()
    }

    bot_pos = start_target;
  }

  print_grid(&grid, bot_pos);

  let mut out = 0usize;
  for y in 0..grid.len() {
    for x in 0..grid[0].len() {
      if grid[y][x] == Slot::BoxStart {
        out += gps_coord((x, y));
      }
    }
  }

  println!("Part 2: {out}");
}
