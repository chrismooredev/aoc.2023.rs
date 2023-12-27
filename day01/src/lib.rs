#![allow(unused_imports)]
use std::str::FromStr;
use std::fmt::Debug;
use itertools::Itertools;
use aoch::AoCDay;
#[cfg(test)] #[allow(unused_imports)]
use aoch::{DayPart, run_test, test_runner, daystr};
use regex::{Regex, Captures};

#[derive(Debug, Clone, Copy)]
pub struct Day01;

lazy_static::lazy_static! {
	static ref ENGLISH_DIGITS: Regex = Regex::new(r#"([1-9])|(one)|(two)|(three)|(four)|(five)|(six)|(seven)|(eight)|(nine)"#).unwrap();
}

impl Day01 {
	fn find_numeric_digits(s: &str) -> Option<(u8, u8)> {
		let mut digits = s.char_indices().enumerate()
			.map(|(ci, (bi, c))| (ci, bi, c))
			.filter_map(|(ci, bi, c)| c.to_digit(10).map(|v| (ci, bi, v as u8)));
		let first = digits.next()?;
		let last = digits.last().unwrap_or(first);
		Some((first.2, last.2))
	}
	fn find_english_digits(s: &str) -> Option<(u8, u8)> {
		let first = ENGLISH_DIGITS.captures_iter(s)
			.map(|capture| Day01::extract_capture_value(&capture))
			.next().unwrap();

		// starting from the end of the string, start scanning until we have a good match
		let last = (0..s.len()).rev()
			.filter_map(|si| {
				ENGLISH_DIGITS.captures_iter(&s[si..])
					.map(|capture| Day01::extract_capture_value(&capture))
					.next()
			})
			.next().unwrap();

		Some((first, last))
	}

	fn extract_capture_value(capture: &Captures<'_>) -> u8 {
		capture.iter()
			.enumerate()
			.skip(1)
			.filter_map(|(mi, om)| om.map(|m| (mi, m)))
			.map(|(mi, m)| {
				// eprintln!("\t\t{}: {:?}", mi, m);
				// get value from capture index as word, or character if in digit capture group
				(match mi {
					1 => m.as_str().chars().next().unwrap().to_digit(10).unwrap() as usize,
					d @ 2..=10 => d-1, // use capture group index
					_ => panic!("unexpected capture group index {} with text {:?}", mi, m.as_str()),
				}) as u8
			})
			// take first (only) match in this capture group
			.next().unwrap()
	}
}

impl AoCDay for Day01 {
	type Data<'i> = Vec<String>;
	type Answer = usize;

	fn day(&self) -> u8 { 1 }

	fn parse<'i>(&self, input: &'i str) -> Self::Data<'i> {
		aoch::parsing::from_lines(input).unwrap()
	}
	fn part1(&self, data: &mut Self::Data<'_>) -> Self::Answer {
		data.iter()
			.enumerate()
			.map(|(_si, s)| {
				let (f, l) = Day01::find_numeric_digits(s).unwrap();
				// eprintln!("{}: {:?} -> {:?}", _si, s, (f, l));
				(f*10 + l) as usize
			})
			.sum()
	}
	fn part2(&self, data: &mut Self::Data<'_>) -> Self::Answer {
		data.iter()
			.enumerate()
			.map(|(_si, s)| {
				let (f, l) = Day01::find_english_digits(s).unwrap();
				// eprintln!("{}: {:?} -> {:?}", _si, s, (f, l));
				(f*10 + l) as usize
			})
			.sum()
	}
}

#[cfg(test)]
const TEST_INPUT_P1: &'static str = "
1abc2
pqr3stu8vwx
a1b2c3d4e5f
treb7uchet
";

#[cfg(test)]
const TEST_INPUT_P2: &'static str = "
two1nine
eightwothree
abcone2threexyz
xtwone3four
4nineeightseven2
zoneight234
7pqrstsixteen
";

#[test]
fn part1() {
	let cases = [
		(TEST_INPUT_P1, 142),
		(daystr!("01"), 55208),
	];
	test_runner::<Day01, _>(Day01, DayPart::Part1, &cases);
}
#[test]
fn part2() {
	let cases = [
		(TEST_INPUT_P2, 281),
		(daystr!("01"), 54578),
	];
	test_runner::<Day01, _>(Day01, DayPart::Part2, &cases);
}
