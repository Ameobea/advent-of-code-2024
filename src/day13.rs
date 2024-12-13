use regex::Regex;
use z3::ast::{Ast, Int};

#[cfg(feature = "local")]
pub const INPUT: &'static str = include_str!("../inputs/day13.txt");

#[derive(Debug)]
struct Puz {
  a_move_size_x: usize,
  a_move_size_y: usize,
  b_move_size_x: usize,
  b_move_size_y: usize,
  prize_x: usize,
  prize_y: usize,
}

fn parse_input(input: &str) -> Vec<Puz> {
  let mut out = Vec::new();

  for input in input.split("\n\n") {
    const INPUT_RGX_STR: &str =
      r#"Button A: X\+(\d+), Y\+(\d+)\nButton B: X\+(\d+), Y\+(\d+)\nPrize: X=(\d+), Y=(\d+)"#;
    let rgx = Regex::new(INPUT_RGX_STR).unwrap();
    let caps = rgx.captures(input).unwrap();

    let a_x = caps[1].parse().unwrap();
    let a_y = caps[2].parse().unwrap();
    let b_x = caps[3].parse().unwrap();
    let b_y = caps[4].parse().unwrap();
    let prize_x = caps[5].parse().unwrap();
    let prize_y = caps[6].parse().unwrap();

    out.push(Puz {
      a_move_size_x: a_x,
      a_move_size_y: a_y,
      b_move_size_x: b_x,
      b_move_size_y: b_y,
      prize_x,
      prize_y,
    })
  }

  out
}

/// Returns `Some(cost)`` if there is a solution, and `None` if no combination of moves could solve
/// the puzzle
fn solve_puzzle<const IS_PART_1: bool>(puz: &Puz) -> Option<usize> {
  let z3_conf = z3::Config::new();
  let ctx = z3::Context::new(&z3_conf);

  let optim = z3::Optimize::new(&ctx);

  // free variables. The "const" naming is misleading; these are the ones that Z3 is free to set
  // values to in order to satisfy + optimize the problem
  let a_presses = Int::new_const(&ctx, "a_presses");
  let b_presses = Int::new_const(&ctx, "b_presses");

  let a_button_cost = Int::from_i64(&ctx, 3);
  let b_button_cost = Int::from_i64(&ctx, 1);

  let a_move_size_x = Int::from_i64(&ctx, puz.a_move_size_x as _);
  let a_move_size_y = Int::from_i64(&ctx, puz.a_move_size_y as _);
  let b_move_size_x = Int::from_i64(&ctx, puz.b_move_size_x as _);
  let b_move_size_y = Int::from_i64(&ctx, puz.b_move_size_y as _);

  let prize_x = Int::from_i64(
    &ctx,
    puz.prize_x as i64 + if IS_PART_1 { 0 } else { 10000000000000 },
  );
  let prize_y = Int::from_i64(
    &ctx,
    puz.prize_y as i64 + if IS_PART_1 { 0 } else { 10000000000000 },
  );

  let constraints = &[
    Int::add(&ctx, &[
      &Int::mul(&ctx, &[&a_move_size_x, &a_presses]),
      &Int::mul(&ctx, &[&b_move_size_x, &b_presses]),
    ])
    ._eq(&prize_x),
    Int::add(&ctx, &[
      &Int::mul(&ctx, &[&a_move_size_y, &a_presses]),
      &Int::mul(&ctx, &[&b_move_size_y, &b_presses]),
    ])
    ._eq(&prize_y),
  ];

  let tokens = Int::add(&ctx, &[
    &Int::mul(&ctx, &[&a_presses, &a_button_cost]),
    &Int::mul(&ctx, &[&b_presses, &b_button_cost]),
  ]);

  optim.minimize(&tokens);
  optim.check(constraints);

  let Some(model) = optim.get_model() else {
    return None;
  };
  let res = model.eval(&tokens, true).unwrap().as_i64().unwrap();
  Some(res as usize)
}

pub fn solve() {
  let puzzles = parse_input(INPUT);

  let mut total_cost = 0usize;
  for puz in &puzzles {
    if let Some(cost) = solve_puzzle::<true>(puz) {
      total_cost += cost;
    }
  }

  println!("Part 1: {total_cost}");

  let mut total_cost = 0usize;
  for puz in &puzzles {
    if let Some(cost) = solve_puzzle::<false>(puz) {
      total_cost += cost;
    }
  }

  println!("Part 2: {total_cost}");
}
