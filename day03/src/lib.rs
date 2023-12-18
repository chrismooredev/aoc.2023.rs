#![allow(unused_imports)]
use core::num;
use std::str::FromStr;
use std::fmt::Debug;
use itertools::Itertools;
use aoch::AoCDay;
#[cfg(test)] #[allow(unused_imports)]
use aoch::{DayPart, run_test, test_runner, daystr};

#[derive(Debug,Clone,Copy)]
pub struct Day03;

type Symbol = (char, usize, usize);

#[derive(Debug,Clone)]
pub struct GameBoard {
	numbers: Vec<PartNbr>,
	symbols: Vec<Symbol>,
	line_lens: Vec<usize>,
}

impl GameBoard {
	fn number_adjacent_to(&self, nbr: &PartNbr, sym: &Symbol) -> bool {
		let (c, r) = (sym.1, sym.2);

		let bound_left = nbr.left.saturating_sub(1);
		let bound_right = nbr.right.unwrap_or(self.line_lens[nbr.row]+1)+1;
		let range = bound_left..=bound_right;

		if ! range.contains(&c) { return false; }

		if r == nbr.row {
			// eprintln!("[{:?}] symbol adjacent to {:?}", sym, nbr);
			return true;
		}
		if r+1 == nbr.row {
			// eprintln!("[{:?}] symbol above {:?}", sym, nbr);
			return true;
		}
		if r == nbr.row+1 {
			// eprintln!("[{:?}] symbol below {:?}", sym, nbr);
			return true;
		}
		false
	}
}

/// A description of a part number. The parsed value, the row, and the inclusive left and right bounds of the string.
#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
struct PartNbr {
	value: usize,
	row: usize,
	left: usize,
	right: Option<usize>,
}

impl AoCDay for Day03 {
	type Data<'i> = GameBoard;
	type Answer = usize;

	fn day(&self) -> u8 { 03 }

	fn parse<'i>(&self, input: &'i str) -> Self::Data<'i> {
		assert!(input.is_ascii(), "input string not ascii");
		// let mut numbers: Vec<PartNbr> = Vec::new();
		// indicates if we are actively within a number
		// let mut num_start = Option::None;
		let (line_lens, mut numbers, symbols, num_start) = input.lines()
			.filter_map(aoch::parsing::trimmed)
			.enumerate()
			// create variables that can be used for an effectively flattened iterator
			.fold((Vec::new(), Vec::new(), Vec::new(), Option::None),
				|(mut line_lens, mut nums, mut syms, mut num_start), (row, raw)| {
					raw.char_indices()
						.for_each(|(col, c)| {
							let digit = c.is_digit(10);
							// eprintln!("{:?} => {:?}", (c, col, row), (digit, num_start));
							match (digit, num_start) {
								(true, None) => { // start a numebr
									num_start = Some((raw, col, row));
								},
								(true, Some((l, c, r))) => { // continue this number

								},
								(false, Some((numstr, nc, nr))) if nr == row => { // finish this number
									num_start = None;
									assert!(numstr == raw, "same line string using different string for source number");
									let txt = &numstr[nc..col];
									let value = txt.parse::<usize>().expect(&format!("unable for parse string at ({:?}) as number : {:?}", (nc, nr), txt));
									nums.push(PartNbr { value, row: nr, left: nc, right: Some(col-1) });
									syms.push((c, col, row));
								},
								(false, Some((numstr, nc, nr))) => { // finish last number
									num_start = None;
									let txt = &numstr[nc..];
									let value = txt.parse::<usize>().unwrap();
									nums.push(PartNbr { value, row: nr, left: nc, right: None });
									syms.push((c, col, row));
								},
								(false, None) => {
									syms.push((c, col, row));
								}
							};
						});
					line_lens.push(raw.len());
					(line_lens, nums, syms, num_start)
			});
		if let Some((numstr, nc, nr)) = num_start {
			// finish last number
			let txt = &numstr[nc..];
			let value = txt.parse::<usize>().unwrap();
			numbers.push(PartNbr { value, row: nr, left: nc, right: None });
		}

		// for (num, v) in numbers.iter().enumerate() {
		// 	eprintln!("N {}: {:?}", num, v);
		// }
		// for (num, v) in symbols.iter().enumerate().filter(|(_, (c, _, _))| *c != '.') {
		// 	eprintln!("S {}: {:?}", num, v);
		// }
		eprintln!("max checksum: {}", numbers.iter().map(|nbr| nbr.value).sum::<usize>());

		GameBoard { line_lens, numbers, symbols }
	}
	fn part1(&self, _data: &mut Self::Data<'_>) -> Self::Answer {
		_data.numbers.iter()
			.filter(|nbr| {
				_data.symbols.iter()
					.filter(|sym| sym.0 != '.')
					.any(|sym| _data.number_adjacent_to(nbr, sym))
			})
			.map(|nbr| nbr.value)
			.sum()
	}
	fn part2(&self, _data: &mut Self::Data<'_>) -> Self::Answer {
		_data.symbols.iter()
			.filter(|sym| sym.0 == '*')
			.filter_map(|sym| {
				let (num1, num2) = _data.numbers.iter()
					.filter(|nbr| _data.number_adjacent_to(nbr, sym))
					.copied()
					.collect_tuple()?;

				// only yield these elements if this gear has two siblings
				Some(num1.value * num2.value)
			})
			.sum()
	}
}

#[cfg(test)]
const TEST_INPUT: &'static str = "
467..114..
...*......
..35..633.
......#...
617*......
.....+.58.
..592.....
......755.
...$.*....
.664.598..
";

#[cfg(test)]
const TEST_INPUT_L: &'static str = "
467..114..
...*......
..35..633.
......#...
617*......
.....+*58.
..592.....
......755.
...$.*....
.664.598..
";

#[cfg(test)]
const TEST_INPUT_TL: &'static str = "
467..114..
...*......
..35..633.
......#...
617*..*...
.....+.58.
..592.....
......755.
...$.*....
.664.598..
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
		(TEST_INPUT, 4361),
		(TEST_INPUT_L, 4361+58),
		(TEST_INPUT_TL, 4361+58),
		(daystr!("03"), 553825),
	];
	test_runner::<Day03, _>(Day03, DayPart::Part1, &cases);
}
#[test]
fn part2() {
	let cases = [
		(TEST_INPUT, 467835),
		(daystr!("03"), 93994191),
	];
	test_runner::<Day03, _>(Day03, DayPart::Part2, &cases);
}
