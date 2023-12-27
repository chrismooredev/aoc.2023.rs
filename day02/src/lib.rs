#![allow(unused_imports)]
use std::{str::FromStr, os::raw};
use std::fmt::Debug;
use itertools::Itertools;
use aoch::AoCDay;
#[cfg(test)] #[allow(unused_imports)]
use aoch::{DayPart, run_test, test_runner, daystr};

#[derive(Debug, Clone, Copy)]
pub struct Day02;

#[derive(Debug, Clone)]
pub struct Game {
	id: usize,
	plays: Vec<Rgb>
}
impl FromStr for Game {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (raw_gid, raw_plays) = s.split_once(": ").unwrap();
		let (_pre, gid) = raw_gid.split_once(' ').unwrap();
		let id: usize = gid.parse().unwrap();

		let plays = raw_plays.split("; ")
			.enumerate()
			.map(|(play, f)| {
				let mut r = Option::None;
				let mut g = Option::None;
				let mut b = Option::None;

				f.split(", ")
					.map(|k| k.split_once(' ').unwrap())
					.for_each(|(num, color)| {
						let count = num.parse().unwrap();
						let opt = match color {
							"red" => &mut r,
							"green" => &mut g,
							"blue" => &mut b,
							_ => panic!("unknown color: {:?}", color),
						};
						assert!(opt.is_none(), "game {}, play {} has repeated colors!", id, play);
						*opt = Some(count);
					});

				Rgb {
					red: r.unwrap_or_default(),
					green: g.unwrap_or_default(),
					blue: b.unwrap_or_default(),
				}
			})
			.collect();

		Ok(Game {
			id,
			plays,
		})
	}
}

#[derive(Debug, Clone, Copy)]
struct Rgb {
	red: usize,
	green: usize,
	blue: usize
}
impl Rgb {
	fn is_subset_of(&self, greater: &Rgb) -> bool {
		self.red <= greater.red
		&& self.green <= greater.green
		&& self.blue <= greater.blue
	}
	fn max(&self, other: &Rgb) -> Rgb {
		Rgb {
			red: self.red.max(other.red),
			green: self.green.max(other.green),
			blue: self.blue.max(other.blue),
		}
	}
}

impl AoCDay for Day02 {
	type Data<'i> = Vec<Game>;
	type Answer = usize;

	fn day(&self) -> u8 { 2 }

	fn parse<'i>(&self, input: &'i str) -> Self::Data<'i> {
		aoch::parsing::from_lines(input).unwrap()
	}
	fn part1(&self, _data: &mut Self::Data<'_>) -> Self::Answer {
		const MAX: Rgb = Rgb { red: 12, green: 13, blue: 14 };
		_data.iter()
			// only the plays that fit within our limit above
			.filter(|game| game.plays.iter().all(|p| p.is_subset_of(&MAX)))
			// fetch those game's IDs
			.map(|game| game.id)
			// sum them up for our score
			.sum()
	}
	fn part2(&self, _data: &mut Self::Data<'_>) -> Self::Answer {
		_data.iter()
			.map(|game| {
				game.plays.iter().copied()
					.reduce(|acc, o| acc.max(&o))
					.map(|rgb| rgb.red * rgb.green * rgb.blue)
					.unwrap()
			})
			.sum()
	}
}

#[cfg(test)]
const TEST_INPUT: &'static str = "
Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green
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
		(TEST_INPUT, 8),
		(daystr!("02"), 3099),
	];
	test_runner::<Day02, _>(Day02, DayPart::Part1, &cases);
}

#[test]
fn part2() {
	let cases = [
		(TEST_INPUT, 2286),
		(daystr!("02"), 72970),
	];
	test_runner::<Day02, _>(Day02, DayPart::Part2, &cases);
}
