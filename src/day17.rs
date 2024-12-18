use itertools::Itertools;
use regex::Regex;
use z3::{
  ast::{Ast, Bool, Int, BV},
  Solver,
};

pub const INPUT: &'static str = include_str!("../inputs/day17.txt");

#[derive(Clone)]
struct Cpu {
  ip: usize,
  a: usize,
  b: usize,
  c: usize,
  prog: Vec<usize>,
}

impl Cpu {
  pub fn tick(&mut self, out: &mut Vec<usize>) -> bool {
    let Some(&op) = self.prog.get(self.ip) else {
      return false;
    };
    let op = Op::from_usize(op);
    let operand = self.prog[self.ip + 1];
    op.apply(self, operand, out);

    true
  }
}

#[allow(dead_code)]
#[repr(usize)]
enum Op {
  Adv = 0,
  Bxl = 1,
  Bst = 2,
  Jnz = 3,
  Bxc = 4,
  Out = 5,
  Bdv = 6,
  Cdv = 7,
}

impl Op {
  fn from_usize(val: usize) -> Self {
    if val > 7 {
      panic!();
    }

    unsafe { std::mem::transmute(val) }
  }

  fn get_combo_operand(cpu: &Cpu, val: usize) -> usize {
    match val {
      lit @ (0 | 1 | 2 | 3) => lit,
      4 => cpu.a,
      5 => cpu.b,
      6 => cpu.c,
      7 => unreachable!(),
      _ => unreachable!(),
    }
  }

  pub fn apply(&self, cpu: &mut Cpu, val: usize, out: &mut Vec<usize>) {
    match self {
      Op::Adv => cpu.a = cpu.a / 2usize.pow(Self::get_combo_operand(cpu, val) as u32),
      Op::Bxl => cpu.b = cpu.b ^ val,
      Op::Bst => cpu.b = Self::get_combo_operand(cpu, val) % 8,
      Op::Jnz =>
        if cpu.a == 0 {
          cpu.ip += 2;
          return;
        } else {
          cpu.ip = val;
          return;
        },
      Op::Bxc => cpu.b = cpu.b ^ cpu.c,
      Op::Out => out.push(Self::get_combo_operand(cpu, val) % 8),
      Op::Bdv => cpu.b = cpu.a / 2usize.pow(Self::get_combo_operand(cpu, val) as u32),
      Op::Cdv => {
        cpu.c = cpu.a / 2usize.pow(Self::get_combo_operand(cpu, val) as u32);
      },
    }

    cpu.ip += 2;
  }
}

struct Outs<'a> {
  a: BV<'a>,
  out: BV<'a>,
  exit: Bool<'a>,
}

fn one_iter<'a>(ctx: &'a z3::Context, a: BV<'a>) -> Outs<'a> {
  // 2,4
  // b = a % 8
  let b = a.bvurem(&BV::from_u64(&ctx, 8, 64));

  // 1,1
  // b = b ^ 1
  let b = b.bvxor(&BV::from_u64(&ctx, 1, 64));

  // 7,5
  // c = (a / 2^b).trunc()
  let c = a.bvudiv(&BV::from_u64(&ctx, 1, 64).bvshl(&b));

  // 1,5
  // b = b ^ 5
  let b = b.bvxor(&BV::from_u64(&ctx, 5, 64));

  // 4,3
  // b = b ^ c
  let b = b.bvxor(&c);

  // 0,3
  // a = a / 2^3
  let a = a.bvudiv(&BV::from_u64(&ctx, 8, 64));

  // 5,5
  // out = b % 8
  let out = b.bvurem(&BV::from_u64(&ctx, 8, 64));

  // exit if a == 0
  let exit = a._eq(&BV::from_u64(&ctx, 0, 64));

  Outs { a, out, exit }
}

fn check_z3() {
  let z3_conf = z3::Config::new();
  let ctx = z3::Context::new(&z3_conf);
  let solver = Solver::new(&ctx);

  let a = BV::from_u64(&ctx, 46323429, 64);

  let outs = one_iter(&ctx, a);

  solver.assert(&outs.out._eq(&BV::from_u64(&ctx, 7, 64)));
  solver.assert(&outs.a._eq(&BV::from_u64(&ctx, 5790428, 64)));
  assert_eq!(solver.check(), z3::SatResult::Sat);
}

fn do_z3(outputs: &[usize], lt: Option<usize>) -> Option<usize> {
  let z3_conf = z3::Config::new();
  let ctx = z3::Context::new(&z3_conf);
  let solver = Solver::new(&ctx);

  let zero = BV::from_u64(&ctx, 0, 64);

  let mut a = BV::new_const(&ctx, "a", 64);
  let orig_a = a.clone();
  solver.assert(&orig_a.bvugt(&zero));

  if let Some(lt) = lt {
    solver.assert(&orig_a.bvult(&BV::from_u64(&ctx, lt as u64, 64)));
  }

  for i in 0..outputs.len() {
    let last_a = a.clone();
    let out = one_iter(&ctx, a);
    solver.assert(&out.a.bvult(&last_a));
    a = out.a;

    solver.assert(&out.exit._eq(&Bool::from_bool(&ctx, i == outputs.len() - 1)));

    solver.assert(&out.out._eq(&BV::from_u64(&ctx, outputs[i] as u64, 64)));
  }

  solver.check();
  let model = solver.get_model()?;
  let out = model.eval(&Int::from_bv(&orig_a, false), false)?;
  out.as_u64().map(|u| u as usize)
}

fn parse_input(input: &str) -> Cpu {
  let rgx =
    Regex::new(r#"Register A: (\d+)\nRegister B: (\d+)\nRegister C: (\d+)\n\nProgram: (.+)"#)
      .unwrap();

  let caps = rgx.captures(input).unwrap();
  let a = caps[1].parse::<usize>().unwrap();
  let b = caps[2].parse::<usize>().unwrap();
  let c = caps[3].parse::<usize>().unwrap();
  let prog = caps[4]
    .split(',')
    .map(|i| i.parse::<usize>().unwrap())
    .collect();

  Cpu {
    ip: 0,
    a,
    b,
    c,
    prog,
  }
}

pub fn solve() {
  let orig_cpu = parse_input(INPUT);
  let mut cpu = orig_cpu.clone();

  let mut out = Vec::new();
  while cpu.tick(&mut out) {}

  let out = out.iter().copied().join(",");

  println!("Part 1: {out}");

  check_z3();

  let mut lt = None;
  loop {
    match do_z3(&cpu.prog, lt) {
      Some(res) => {
        lt = Some(res);
      },
      None => {
        println!("Part 2: {}", lt.unwrap());
        return;
      },
    }
  }
}
