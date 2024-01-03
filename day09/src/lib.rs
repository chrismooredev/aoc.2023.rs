#![allow(unused_imports)]
use std::collections::VecDeque;
use std::ops::{Add, Sub};
use std::slice;
use std::{str::FromStr, fmt};
use std::fmt::Debug;
use itertools::Itertools;
use test_log::test;
use aoch::AoCDay;
#[cfg(test)] #[allow(unused_imports)]
use aoch::{DayPart, run_test, test_runner, daystr};

#[derive(Debug,Clone,Copy)]
pub struct Day09;

#[derive(Debug,Clone)]
struct Layers<T>(Vec<VecDeque<T>>, T);
impl fmt::Display for Layers<isize> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut layers = self.clone();
		layers.add_base_layer(self.1);
		layers.add_base_layer(0);

		let max_num_len = layers.0.iter()
			.flat_map(|v| v.iter())
			.map(|n| {
				let mut n = *n;
				let mut offset = 1;
				if n.is_negative() {
					offset += 1; // negative sign
					n = -n;
				} else if n == 0 {
					return 1;
				}
				n.ilog10() + offset
			})
			.max().unwrap() as usize;

		for (rowi, row) in layers.0.iter().enumerate() {
			for elem in row {
				write!(f, "{:>max_num_len$}  ", elem)?;
			}
			let ident = (rowi+1)*(max_num_len);
			write!(f, "\n{:>ident$}", "")?;
		}
		Ok(())
    }
}
impl<T> Layers<T> {
	fn add_base_layer(&mut self, v: T) where T: Clone {
		self.0.push((0..self.0.last().unwrap().len()-1)
			.map(|_| v.clone())
			.collect_vec().into());
	}
	fn extrapolate_back(&mut self) where T: Clone + Add<T, Output = T> {
		let mut last = self.1.clone();
		for k in self.0.iter_mut().rev() {
			let new = k.back().unwrap().clone() + last;
			k.push_back(new.clone());
			last = new;
		}
	}
	fn extrapolate_front(&mut self) where T: Clone + Sub<T, Output = T> {
		let mut last = self.1.clone();
		for k in self.0.iter_mut().rev() {
			let new = k.front().unwrap().clone() - last;
			k.push_front(new.clone());
			last = new;
		}
	}
}

#[derive(Debug,Clone)]
pub struct Sequence<T>(Vec<T>);
impl<T: Clone + Copy + PartialEq> Sequence<T> {
	fn find_differences(&self) -> Layers<T> where T: Sub<T, Output = T> {
		let mut layers = Vec::with_capacity(self.0.len()-2);

		let mut upper_layer = self.0.clone();
		while ! upper_layer.iter().all_equal() {
		// while ! upper_layer.iter().all(<T as Zero>::is_zero) {
			let mut next_layer = Vec::with_capacity(self.0.len()-1);
			upper_layer.iter()
				.copied()
				.reduce(|last, n| { next_layer.push(n-last); n })
				.expect("no elements in layer array");

			assert_eq!(upper_layer.len(), next_layer.len()+1, "reduce created unexpected number of elements");
			layers.push(upper_layer.into());
			upper_layer = next_layer;
		}

		let base = *upper_layer.first().unwrap();
		Layers(layers, base)
	}
}

impl AoCDay for Day09 {
	type Data<'i> = Vec<Sequence<isize>>;
	type Answer = isize;

	fn day(&self) -> u8 { 9 }

	fn parse<'i>(&self, input: &'i str) -> Self::Data<'i> {
		input.lines()
			.filter_map(aoch::parsing::trimmed)
			.map(|l| {
				let mut seq = Sequence(l.split(' ').map(|ns| ns.parse().unwrap()).collect_vec());
				seq.0.reserve(2); // preallocate for the actual puzzle
				seq
			})
			.collect_vec()
	}
	fn part1(&self, _data: &mut Self::Data<'_>) -> Self::Answer {
		_data.iter()
			.map(|hist| {
				let mut layers = hist.find_differences();
				layers.extrapolate_back();
				log::debug!("extrapolated history ({:?}):\n{}", hist, layers);
				let n = *layers.0.first().as_ref().unwrap().back().unwrap();
				n
			})
			.sum()
	}
	fn part2(&self, _data: &mut Self::Data<'_>) -> Self::Answer {
		_data.iter()
			.map(|hist| {
				let mut layers = hist.find_differences();
				layers.extrapolate_front();
				log::debug!("extrapolated history ({:?}):\n{}", hist, layers);
				let n = *layers.0.first().as_ref().unwrap().front().unwrap();
				n
			})
			.sum()
	}
}

#[cfg(test)]
const TEST_INPUT: &'static str = "
0 3 6 9 12 15
1 3 6 10 15 21
10 13 16 21 30 45
";

#[cfg(test)]
const TEST_INPUT_3: &'static str = "
10 13 16 21 30 45
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
		(TEST_INPUT, 114),
		(daystr!("09"), 1853145119),
	];
	test_runner::<Day09, _>(Day09, DayPart::Part1, &cases);
}
#[test]
fn part2() {
	let cases = [
		(TEST_INPUT_3, 5),
		(TEST_INPUT, 2),
		(daystr!("09"), 923),
	];
	test_runner::<Day09, _>(Day09, DayPart::Part2, &cases);
}
