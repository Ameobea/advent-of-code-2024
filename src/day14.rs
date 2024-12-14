use std::time::Duration;

use regex::Regex;

pub const INPUT: &'static str = include_str!("../inputs/day14.txt");

#[derive(Debug)]
struct Bot {
  pos: (isize, isize),
  vel: (isize, isize),
}

fn parse_input(input: &str) -> Vec<Bot> {
  let rgx = Regex::new(r#"p=(-?\d+),(-?\d+) v=(-?\d+),(-?\d+)"#).unwrap();

  input
    .lines()
    .map(|l| {
      let caps = rgx.captures(l).unwrap();
      Bot {
        pos: (caps[1].parse().unwrap(), caps[2].parse().unwrap()),
        vel: (caps[3].parse().unwrap(), caps[4].parse().unwrap()),
      }
    })
    .collect()
}

pub fn solve() {
  let mut bots = parse_input(INPUT);

  let width = 101;
  let height = 103;
  let mid_x = 50;
  let mid_y = 51;

  let get_quad = |pos: (isize, isize)| -> usize {
    if pos.0 == mid_x || pos.1 == mid_y {
      return 0;
    }

    let mut quad = 1usize;
    if pos.0 > mid_x {
      quad += 2;
    }
    if pos.1 > mid_y {
      quad += 1;
    }

    quad
  };

  for _ in 0..100 {
    for bot in &mut bots {
      let mut next_pos = (bot.pos.0 + bot.vel.0, bot.pos.1 + bot.vel.1);
      if next_pos.0 >= width {
        next_pos.0 = next_pos.0 % width;
      }
      if next_pos.1 >= height {
        next_pos.1 = next_pos.1 % height;
      }
      if next_pos.0 < 0 {
        next_pos.0 = width - -next_pos.0;
      }
      if next_pos.1 < 0 {
        next_pos.1 = height - -next_pos.1;
      }
      bot.pos = next_pos;
    }
  }

  let mut counts_by_quad = [0, 0, 0, 0, 0];
  for bot in &bots {
    let quad = get_quad(bot.pos);
    counts_by_quad[quad] += 1;
  }

  let out = counts_by_quad[1] * counts_by_quad[2] * counts_by_quad[3] * counts_by_quad[4];

  println!("Part 1: {out}");

  let mut bots = parse_input(INPUT);
  let mut i = 0usize;
  let out = loop {
    i += 1;
    for bot in &mut bots {
      let mut next_pos = (bot.pos.0 + bot.vel.0, bot.pos.1 + bot.vel.1);
      if next_pos.0 >= width {
        next_pos.0 = next_pos.0 % width;
      }
      if next_pos.1 >= height {
        next_pos.1 = next_pos.1 % height;
      }
      if next_pos.0 < 0 {
        next_pos.0 = width - -next_pos.0;
      }
      if next_pos.1 < 0 {
        next_pos.1 = height - -next_pos.1;
      }
      bot.pos = next_pos;
    }

    let mut counts_by_y = vec![0usize; height as usize];
    let mut counts_by_x = vec![0usize; width as usize];
    for bot in &bots {
      counts_by_x[bot.pos.0 as usize] += 1;
      counts_by_y[bot.pos.1 as usize] += 1;
    }

    if counts_by_x.iter().filter(|&&c| c >= 31).count() < 2
      || counts_by_y.iter().filter(|&&c| c >= 30).count() < 2
    {
      continue;
    }

    let mut out = vec![vec![' '; width as usize]; height as usize];
    for bot in &bots {
      out[bot.pos.1 as usize][bot.pos.0 as usize] = 'x';
    }
    // println!("{}", i + 1);
    // for row in out {
    //   let s = row.into_iter().collect::<String>();
    //   println!("{s}");
    // }

    // println!("\n\n");
    // std::thread::sleep(Duration::from_millis(200));
    break i;
  };

  println!("Part 2: {out}");
}
