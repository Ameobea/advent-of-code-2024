const INPUT: &'static str = include_str!("../inputs/day5.txt");

use std::collections::{HashMap, HashSet};

use pathfinding::directed::topological_sort::topological_sort;

fn parse_input(input: &str) -> (Vec<(usize, usize)>, Vec<Vec<usize>>) {
  let spl = input.split_once("\n\n").unwrap();

  let deps: Vec<(usize, usize)> = spl
    .0
    .lines()
    .map(|s| {
      let (a, b) = s.split_once('|').unwrap();
      (a.parse().unwrap(), b.parse().unwrap())
    })
    .collect();

  let pages: Vec<Vec<usize>> = spl
    .1
    .lines()
    .map(|s| s.split(',').map(|s| s.parse().unwrap()).collect())
    .collect();

  (deps, pages)
}

pub fn solve() {
  let (deps, pages) = parse_input(INPUT);

  let roots: HashSet<usize> = deps.iter().flat_map(|(a, b)| [*a, *b]).collect();
  let mut successors_by_key: HashMap<usize, Vec<usize>> = HashMap::default();
  for (a, b) in &deps {
    successors_by_key.entry(*a).or_default().push(*b);
  }

  let (sorted_pages, unsorted_pages): (Vec<_>, Vec<_>) = pages
    .iter()
    .map(|pages| {
      let sorted = topological_sort(
        &roots
          .iter()
          .filter(|r| pages.contains(*r))
          .copied()
          .collect::<Vec<_>>(),
        |n| {
          successors_by_key[n]
            .iter()
            .copied()
            .filter(|s| pages.contains(s))
        },
      )
      .unwrap();

      (sorted, pages)
    })
    .partition(|(sorted, pages)| {
      pages.is_sorted_by_key(|p| sorted.iter().position(|an| an == p).unwrap())
    });

  let sum = sorted_pages
    .iter()
    .map(|(_sorted, pages)| {
      let middle_page_ix = pages.len() / 2;
      pages[middle_page_ix]
    })
    .sum::<usize>();

  println!("Part 1: {sum}");

  let mut sum = 0;
  for (sorted, pages) in unsorted_pages {
    let middle_page_ix = pages.len() / 2;
    sum += sorted[middle_page_ix];
  }

  println!("Part 2: {sum}");
}
