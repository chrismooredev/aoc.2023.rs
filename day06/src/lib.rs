#![allow(unused_imports)]
use std::str::FromStr;
use std::fmt::Debug;
use itertools::Itertools;
use test_log::test;
use aoch::AoCDay;
#[cfg(test)] #[allow(unused_imports)]
use aoch::{DayPart, run_test, test_runner, daystr};

#[derive(Debug,Clone,Copy)]
pub struct Day06;

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct Race {
	time: usize,
	dist: usize,
}
impl Race {
	/// Returns "button held down for", and "distance went"
	fn best(&self) -> impl Iterator<Item = (usize, usize)> {
		let Race { time, dist } = *self;
		(0..self.time)
			.map::<(usize, usize), _>(move |held| {
				(held, held*(time - held))
			})
			.filter(move |&(_, d)| d > dist)
	}
}

#[derive(Debug)]
pub struct RaceResults {
	separate: Vec<Race>,
	combined: Race,
}

impl AoCDay for Day06 {
	type Data<'i> = RaceResults;
	type Answer = usize;

	fn day(&self) -> u8 { 6 }

	fn parse<'i>(&self, input: &'i str) -> Self::Data<'i> {
		let (times, dists) = input.trim().split_once('\n').expect("not two lines");

		let itimes = times.split(' ').filter_map(aoch::parsing::trimmed);
		let idists = dists.split(' ').filter_map(aoch::parsing::trimmed);

		let separate = itimes.zip_eq(idists)
			.skip(1) // row title
			// .inspect(|d| eprintln!("{:?}", d))
			.map(|(stime, sdist)| (stime.parse().unwrap(), sdist.parse().unwrap()))
			.map(|(time, dist)| Race { time, dist })
			.collect_vec();

		let combined = separate.iter()
			.copied()
			.reduce(|mut acc, race| {
				// eprintln!("reducing {:?} into {:?}", race, acc);
				acc.time = acc.time*(10usize).pow(race.time.ilog10()+1) + race.time;
				acc.dist = acc.dist*(10usize).pow(race.dist.ilog10()+1) + race.dist;
				acc
			}).unwrap();

		// eprintln!("combined: {:?}", combined);

		RaceResults {
			separate,
			combined,
		}
	}
	fn part1(&self, _data: &mut Self::Data<'_>) -> Self::Answer {
		_data.separate.iter()
			.map(|r| r.best().count())
			.product()
	}
	fn part2(&self, _data: &mut Self::Data<'_>) -> Self::Answer {
		_data.combined.best().count()
	}
}

#[cfg(test)]
const TEST_INPUT: &'static str = "
Time:      7  15   30
Distance:  9  40  200
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
		(TEST_INPUT, 288),
		(daystr!("06"), 771628),
	];
	test_runner::<Day06, _>(Day06, DayPart::Part1, &cases);
}
#[test]
fn part2() {
	let cases = [
		(TEST_INPUT, 71503),
		(daystr!("06"), 27363861),
	];
	test_runner::<Day06, _>(Day06, DayPart::Part2, &cases);
}
