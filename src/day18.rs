use pathfinding::directed::astar;

pub const INPUT: &'static str = include_str!("../inputs/day18.txt");

fn parse_input(input: &str) -> Vec<(usize, usize)> {
  input
    .lines()
    .take_while(|l| l.len() >= 1)
    .map(|l| {
      let (s, e) = l.split_once(',').unwrap();
      (s.parse().unwrap(), e.parse().unwrap())
    })
    .collect()
}

const WIDTH: usize = 71;
const HEIGHT: usize = 71;

fn pathfind(input: &[(usize, usize)], timestep: usize) -> Option<usize> {
  let grid = get_grid(&input, timestep);

  let start_coord = (0, 0);
  let next_coord = |(x, y): (usize, usize)| {
    let mut outs = Vec::new();
    for [o_x, o_y] in [
      [x as isize - 1, y as isize],
      [x as isize + 1, y as isize],
      [x as isize, y as isize - 1],
      [x as isize, y as isize + 1],
    ] {
      if o_x < 0 || o_y < 0 || o_x >= grid[0].len() as isize || o_y >= grid.len() as isize {
        continue;
      }

      if grid[o_y as usize][o_x as usize] {
        continue;
      }

      let next_coord = (o_x as usize, o_y as usize);
      outs.push((next_coord, 1));
    }

    outs.into_iter()
  };

  let end_coord = (WIDTH - 1, HEIGHT - 1);
  astar::astar(&start_coord, |c| next_coord(*c), |_| 1, |c| *c == end_coord).map(|p| p.0.len())
}

fn get_grid(inputs: &[(usize, usize)], timestep: usize) -> Vec<Vec<bool>> {
  let mut grid = vec![vec![false; WIDTH]; HEIGHT];

  for i in 0..timestep {
    if i >= inputs.len() {
      break;
    }
    grid[inputs[i].1][inputs[i].0] = true;
  }

  grid
}

pub fn solve() {
  let input = parse_input(INPUT);

  let part1 = pathfind(&input, 1024).unwrap();
  println!("Part 1: {part1}");

  for i in 0..input.len() {
    if pathfind(&input, i).is_some() {
      continue;
    }

    println!("Part 2: {},{}", input[i].0, input[i].1);
    break;
  }
}
