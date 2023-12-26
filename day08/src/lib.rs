#![feature(extract_if)]

#![allow(unused_imports)]
use std::collections::HashMap;
use std::{str::FromStr, collections::BTreeMap};
use std::fmt::Debug;
use itertools::Itertools;
use test_log::test;
use aoch::AoCDay;
#[cfg(test)] #[allow(unused_imports)]
use aoch::{DayPart, run_test, test_runner, daystr};

//TODO: Replace the strings with integer keys

#[derive(Debug,Clone,Copy)]
pub struct Day08;

#[derive(Debug,Clone,Default)]
pub struct Map<'s> {
	directions: &'s str,
	mapping: BTreeMap<&'s str, (&'s str, &'s str)>,
}

impl AoCDay for Day08 {
	type Data<'i> = Map<'i>;
	type Answer = usize;

	fn day(&self) -> u8 { 08 }

	fn parse<'i>(&self, input: &'i str) -> Self::Data<'i> {
		let mut lines = input.lines()
			.filter_map(aoch::parsing::trimmed);
		let directions = lines.next().unwrap();
		let mapping = lines.map(|line| {
			let (key, dests) = line.split_once(" = ").unwrap();
			let (left, right) = dests.split_once(", ").unwrap();
			(key, (&left[1..], &right[..right.len()-1]))
		})
		.collect();

		Map {
			directions,
			mapping,
		}
	}
	fn part1(&self, _data: &mut Self::Data<'_>) -> Self::Answer {
		const SRC: &str = "AAA";
		const DST: &str = "ZZZ";
		let mut current = SRC;
		let mut steps = 0;
		let mut dirs = std::iter::repeat(_data.directions).map(|s| s.chars()).flatten();

		while let Some(dir) = dirs.next() { // will never return None, but prevents an unwrap
			match (dir, _data.mapping.get(current)) {
				(_, None) => panic!("unexpected key: {:?}", current),
				('L', Some((left, right))) => {
					current = left;
				},
				('R', Some((left, right))) => {
					current = right;
				},
				(l, Some(_)) => panic!("unknown direction: {}", l),
			}

			steps += 1;

			if current == DST {
				break;
			}
		}

		steps
	}
	fn part2(&self, _data: &mut Self::Data<'_>) -> Self::Answer {
		let mut current_nodes = _data.mapping.keys()
			.filter(|n| n.ends_with('A'))
			.copied()
			.map(|n| (n, n))
			.collect_vec();
		let mut found_cycles: HashMap<&str, HashMap<&str, usize>> = Default::default();
		// let mut found_cycles: Vec<(&str, usize)> = Vec::with_capacity(current_nodes.len());

		eprintln!("running with {} nodes", current_nodes.len());

		let mut steps = 1;
		let mut dirs = std::iter::repeat(_data.directions).map(|s| s.chars()).flatten();

		while let Some(dir) = dirs.next() { // will never return None, but prevents an unwrap
			current_nodes.retain_mut(|(start, node)| {
				*node = match (dir, _data.mapping.get(*node)) {
					(_, None) => panic!("unexpected key: {:?}", *node),
					('L', Some((left, _))) => left,
					('R', Some((_, right))) => right,
					(l, Some(_)) => panic!("unknown direction: {}", l),
				};

				// *node = new;

				if start == node {
					false
				} else if node.ends_with('Z') {
					let orig_step = found_cycles.entry(start)
						.or_default()
						.entry(node)
						.or_insert(steps);

					// only retain if this is a new ending
					*orig_step == steps
				} else {
					true
				}
			});

			steps += 1;
			if steps % 1_000_000 == 0 {
				eprintln!("steps = {}M, found = {:?}, current = {:?}", steps/1_000_000, found_cycles, current_nodes);
			}

			if current_nodes.len() == 0 {
				eprintln!("found = {:?}, current = {:?}", found_cycles, current_nodes);
				break;
			}
		}

		let lcm_steps: usize = found_cycles.iter()
			.map(|(start, ends)| ends.iter().map(move |(end, steps)| (start, end, steps)))
			.flatten()
			.inspect(|(start, end, steps)| eprintln!("{:?} --{}--> {:?}", start, steps, end))
			.map(|(_start, _end, steps)| *steps)
			.fold(1, |acc, steps| num::Integer::lcm(&acc, &steps));

		lcm_steps
	}
}

#[cfg(test)]
const TEST_INPUT_RL: &'static str = "
RL

AAA = (BBB, CCC)
BBB = (DDD, EEE)
CCC = (ZZZ, GGG)
DDD = (DDD, DDD)
EEE = (EEE, EEE)
GGG = (GGG, GGG)
ZZZ = (ZZZ, ZZZ)
";

#[cfg(test)]
const TEST_INPUT_LLR: &'static str = "
LLR

AAA = (BBB, BBB)
BBB = (AAA, ZZZ)
ZZZ = (ZZZ, ZZZ)
";

#[cfg(test)]
const TEST_INPUT_LR_PAR: &'static str = "
LR

11A = (11B, XXX)
11B = (XXX, 11Z)
11Z = (11B, XXX)
22A = (22B, XXX)
22B = (22C, 22C)
22C = (22Z, 22Z)
22Z = (22B, 22B)
XXX = (XXX, XXX)
";

/*
#[test]
fn fuel_calc() {
	let cases = [
		(100756, 33583),
	];
	run_test(|n| DayMe::calc_fuel(*n), &cases);
}
*/

#[test]
fn part1() {
	let cases = [
		(TEST_INPUT_RL, 2),
		(TEST_INPUT_LLR, 6),
		(daystr!("08"), 22411),
	];
	test_runner::<Day08, _>(Day08, DayPart::Part1, &cases);
}
#[test]
fn part2() {
	let cases = [
		(TEST_INPUT_LR_PAR, 6),
		(daystr!("08"), 11188774513823),
	];
	test_runner::<Day08, _>(Day08, DayPart::Part2, &cases);
}
