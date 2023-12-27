#![feature(iterator_try_reduce)]

#![allow(unused_imports)]
use std::{str::FromStr, fmt};
use std::fmt::Debug;
use itertools::Itertools;
use test_log::test;
use aoch::AoCDay;
#[cfg(test)] #[allow(unused_imports)]
use aoch::{DayPart, run_test, test_runner, daystr};

#[derive(Debug,Clone,Copy)]
pub struct Day04;

fn iter_set_bits(num: u128) -> impl Iterator<Item = usize> {
	(0..u128::BITS)
		.filter(move |i| num & (1u128 << i) != 0)
		.map(|n| n as usize)
}
fn collect_into_bitset(s: &str) -> Result<u128, std::num::ParseIntError> {
	s.split(' ')
		.map(|s| s.trim())
		.filter(|s| !s.is_empty())
		.try_fold(0u128,|acc, s| {
			let n: u32 = s.parse()?;
			assert!(n < u128::BITS, "attempt to register scratchcard number over 128: {}", n);
			Ok(acc | (1u128 << n))
		})
}

#[derive(Clone,Copy,PartialEq,Eq,Hash)]
pub struct ScratchCard {
	index: usize,
	winning: u128,
	results: u128,
}
impl ScratchCard {
	fn points(&self) -> usize {
		let masked = self.winning & self.results;
		let count = masked.count_ones();
		if count == 0 { return 0; }
		1 << (count - 1)
	}
}
impl FromStr for ScratchCard {
	type Err = std::num::ParseIntError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (ridx, rnums) = s.split_once(": ").unwrap();
		log::debug!("card#: {:?}", ridx);
		let index: usize = ridx[4..].trim().parse().unwrap();
		let (winning, results) = rnums.split_once(" | ").unwrap();
		log::debug!("(winning,results) = {:?}", (winning,results));
		let winning = collect_into_bitset(winning)?;
		let results = collect_into_bitset(results)?;
		Ok(ScratchCard { index, winning, results })
	}
}
impl fmt::Debug for ScratchCard {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let winning = iter_set_bits(self.winning).collect_vec();
		let results = iter_set_bits(self.results).collect_vec();
		f.debug_struct("ScratchCard")
			.field("index", &self.index)
			.field("winning", &winning)
			.field("results", &results)
			.field("points", &self.points())
			.finish()
	}
}

impl AoCDay for Day04 {
	type Data<'i> = Vec<ScratchCard>;
	type Answer = usize;

	fn day(&self) -> u8 { 4 }

	fn parse<'i>(&self, input: &'i str) -> Self::Data<'i> {
		aoch::parsing::from_lines(input).unwrap()
	}
	fn part1(&self, _data: &mut Self::Data<'_>) -> Self::Answer {
		_data.iter().enumerate()
			.inspect(|t| log::debug!("{:?}", t))
			.map(|(_, sc)| sc.points())
			.sum()
	}
	fn part2(&self, _data: &mut Self::Data<'_>) -> Self::Answer {
		let mut hits = vec![1; _data.len()];
		for (ci, card) in _data.iter().enumerate() {
			assert!(ci+1 == card.index, "card index not matching array index");
			let matching = (card.winning & card.results).count_ones();
			let card_count = hits[ci];
			for i in 0..matching {
				let dest_card = card.index + (i as usize);
				if let Some(oc) = hits.get_mut(dest_card) {
					*oc += card_count;
				}
			}
		}
		hits.iter().sum()
	}
}

#[cfg(test)]
const TEST_INPUT: &'static str = "
Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83
Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11
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
		(TEST_INPUT, 13),
		(daystr!("04"), 21158),
	];
	test_runner::<Day04, _>(Day04, DayPart::Part1, &cases);
}
#[test]
fn part2() {
	let cases = [
		(TEST_INPUT, 30),
		(daystr!("04"), 6050769),
	];
	test_runner::<Day04, _>(Day04, DayPart::Part2, &cases);
}
