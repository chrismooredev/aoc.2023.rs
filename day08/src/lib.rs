#![allow(unused_imports)]
use std::collections::HashMap;
use std::{str::FromStr, collections::BTreeMap};
use std::fmt::Debug;
use itertools::Itertools;
use test_log::test;
use aoch::AoCDay;
#[cfg(test)] #[allow(unused_imports)]
use aoch::{DayPart, run_test, test_runner, daystr};

/// Each mapping is stored within a vector. During parsing, the string-based mapping
/// is converted to use indicies into a vector, where the only source info remaining
/// is if it is a full start/end node (fully 'A' or 'Z'), or just a plain start/end
/// node (ending with 'A' or 'Z'). After traversing the map, the amount of steps needed
/// to reach the end are combined to find the least-common-multiple of them to produce the answer.
#[derive(Debug,Clone,Copy)]
pub struct Day08;

#[derive(Debug,Clone,Copy,PartialEq)]
enum NodeType {
	None,
	Start,
	FullStart,
	End,
	FullEnd,
}

impl NodeType {
	fn classify(s: &str) -> NodeType {
		if s.chars().all(|c| c == 'A') {
			NodeType::FullStart
		} else if s.ends_with('A') {
			NodeType::Start
		} else if s.chars().all(|c| c == 'Z') {
			NodeType::FullEnd
		} else if s.ends_with('Z') {
			NodeType::End
		} else {
			NodeType::None
		}
	}
}

#[derive(Debug,Clone)]
pub struct IndexedMap {
	directions: String,
	mapping: Vec<(NodeType, (usize, usize))>,
}

impl IndexedMap {
	fn run_nodes<FS: FnMut(NodeType) -> bool, FE: FnMut(NodeType) -> bool>(&self, mut start_node_pred: FS, mut end_node_pred: FE) -> usize {
		// Vec<(start_idx, curr_idx)>
		let mut current_nodes = self.mapping.iter()
			.enumerate()
			.filter(|(_i, (nt, _))| start_node_pred(*nt))
			.map(|(i, (_, _))| (i, i))
			.collect_vec();

		// Vec<(start_idx, Vec<(end_idx, step)>)>
		let mut found_cycles: Vec<(usize, Vec<(usize, usize)>)> = Default::default();

		log::info!("running with {} nodes for directions {}", current_nodes.len(), self.directions);

		let mut steps = 1;
		let mut dirs = std::iter::repeat(self.directions.as_str()).map(str::chars).flatten();

		while let Some(dir) = dirs.next() { // will never return None, but prevents an unwrap
			log::trace!("step[{}] => current={:?}, found={:?}", steps, current_nodes, found_cycles);
			current_nodes.retain_mut(|(start, curr)| {
				// update current node
				let next = self.mapping.get(*curr);
				*curr = match (dir, next) {
					(_, None) => panic!("unexpected key: {:?}", curr),
					('L', Some((_, (left, _)))) => *left,
					('R', Some((_, (_, right)))) => *right,
					(l, Some(_)) => panic!("unknown direction: {}", l),
				};

				// check if its an ending node
				if end_node_pred(self.mapping[*curr].0) {

					// dispose if we've seen it before, track the current step
					let ent = match found_cycles.iter_mut().find(|p| p.0 == *curr) {
						Some(ent) => ent,
						None => {
							found_cycles.push((*start, vec![(*curr, steps)]));
							found_cycles.last_mut().unwrap()
						},
					};

					// only retain this entry if this is a new ending
					match ent.1.iter().find(|(end_idx, _)| *end_idx == *curr) {
						Some(_) => false,
						None => {
							ent.1.push((*curr, steps));
							true
						}
					}
				} else {
					true
				}
			});

			steps += 1;
			if steps % 1_000_000 == 0 {
				log::debug!("steps = {}M, found = {:?}, current = {:?}", steps/1_000_000, found_cycles, current_nodes);
			}

			if current_nodes.len() == 0 {
				log::debug!("found = {:?}, current = {:?}", found_cycles, current_nodes);
				break;
			}
		}

		let lcm_steps: usize = found_cycles.iter()
			.map(|(start, ends)| ends.iter().map(move |(end, steps)| (*start, *end, *steps)))
			.flatten()
			.inspect(|(start, end, steps)| log::debug!("{:?} --{}--> {:?}", start, steps, end))
			.map(|(_start, _end, steps)| steps)
			.fold(1, |acc, steps| num::Integer::lcm(&acc, &steps));

		lcm_steps
	}
}

#[derive(Debug,Clone,Default)]
pub struct Map<'s> {
	directions: &'s str,
	mapping: Vec<(&'s str, (&'s str, &'s str))>,
}
impl<'s> Map<'s> {
	fn compile(&mut self) -> IndexedMap {
		// sort it so we can search it better for the new mapping
		self.mapping.sort_by_key(|f| f.0);

		let mapping = self.mapping.iter()
			.map(|(n, (l, r))| {
				let li = self.mapping.binary_search_by_key(l, |n| n.0).unwrap();
				let ri = self.mapping.binary_search_by_key(r, |n| n.0).unwrap();
				(NodeType::classify(n), (li, ri))
			})
			.collect_vec();

		IndexedMap {
			directions: self.directions.to_owned(),
			mapping
		}
	}
}

impl AoCDay for Day08 {
	type Data<'i> = IndexedMap;
	type Answer = usize;

	fn day(&self) -> u8 { 08 }

	fn parse<'i>(&self, input: &'i str) -> Self::Data<'i> {
		let mut lines = input.lines()
			.filter_map(aoch::parsing::trimmed);
		let directions = lines.next().unwrap();
		let mut mapping = lines.map(|line| {
			let (key, dests) = line.split_once(" = ").unwrap();
			let (left, right) = dests.split_once(", ").unwrap();

			(key, (&left[1..], &right[..right.len()-1]))
		})
		.collect_vec();

		mapping.sort();

		Map {
			directions,
			mapping,
		}.compile()
	}
	fn part1(&self, _data: &mut Self::Data<'_>) -> Self::Answer {
		_data.run_nodes(|n| n == NodeType::FullStart, |n| n == NodeType::FullEnd)
	}
	fn part2(&self, _data: &mut Self::Data<'_>) -> Self::Answer {
		_data.run_nodes(
			|n| n == NodeType::Start || n == NodeType::FullStart,
			|n| n == NodeType::End || n == NodeType::FullEnd,
		)
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
