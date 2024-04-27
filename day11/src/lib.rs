#![allow(unused_imports)]
use std::{fmt, str::FromStr};
use std::fmt::Debug;
use itertools::Itertools;
use test_log::test;
use aoch::AoCDay;
#[cfg(test)] #[allow(unused_imports)]
use aoch::{DayPart, run_test, test_runner, daystr};

type Num = usize;

#[derive(Debug,Clone,Copy)]
pub struct Day11;

#[derive(Clone)]
pub struct Coordinates {
	raw: Vec<(Num, Num)>,
}

impl Coordinates {
	pub fn bounds_with_min(&self) -> ((Num, Num), (Num, Num)) {
		let mut iter = self.raw.iter().copied();
		let mut min = iter.next().expect("no elements in raw map");
		let mut max = min;
		for (x, y) in iter {
			if x < min.0 { min.0 = x; }
			if x > max.0 { max.0 = x; }
			if y < min.1 { min.1 = y; }
			if y > max.1 { max.1 = y; }
		}
		(min, max)
	}
	pub fn bounds_max(&self) -> (Num, Num) {
		self.raw.iter().copied()
			.reduce(|(ax, ay), (ex, ey)| (ax.max(ex), ay.max(ey)))
			.unwrap()
	}
	pub fn expand(&mut self, gap_scale: usize) {
		let gap_add = gap_scale - 1;
		let ((nx, ny), (mut xx, mut xy)) = self.bounds_with_min();
		eprintln!("expanding {} nodes with scale {} -- x", self.raw.len(), gap_add);

		let mut x = nx+1;
		while x <= xx {
			let empty = ! self.raw.iter().any(|(ix, _)| *ix == x);
			if empty {
				// col is empty, expand it
				self.raw.iter_mut()
					.for_each(|(ix, _)| {
						if *ix > x {
							*ix += gap_add;
						}
					});
				xx += gap_add;
			}

			x += 1;
			if empty {
				x += gap_add;
			}

			// eprintln!("[{}] x = {} (nx, xx) = {:?} (was empty = {:?})", self.raw.len(), x, (nx, xx), empty);
		}

		eprintln!("expanding {} nodes with scale {} -- y", self.raw.len(), gap_add);

		let mut y = ny+1;
		while y <= xy {
			let empty = ! self.raw.iter().any(|(_, iy)| *iy == y);
			if empty {
				// col is empty, expand it
				self.raw.iter_mut()
					.for_each(|(_, iy)| {
						if *iy > y {
							*iy += gap_add;
						}
					});
				xy += gap_add;
			}

			y += 1;
			if empty {
				y += gap_add;
			}

			// eprintln!("[{}] y = {} (ny, xy) = {:?} (was empty = {:?})", self.raw.len(), y, (ny, xy), empty);
		}

		eprintln!("expanding {} nodes with scale {} -- done", self.raw.len(), gap_add);
	}

	pub fn pair_dist_sum(&self) -> usize {
		let mut sum = 0;
		let count = (self.raw.len()*(self.raw.len()-1))/2;
		eprintln!("Pairs: {count:}");
		for i in 0..self.raw.len()-1 {
			for j in i+1..self.raw.len() {
				let (ax, ay) = self.raw[i];
				let (bx, by) = self.raw[j];
				// eprintln!("[{}, {}] = ({:?}, {:?})", i, j, (ax, ay), (bx, by));
				let len = ax.abs_diff(bx) + ay.abs_diff(by);
				// eprintln!("[{}, {}] = ({:?}, {:?}) = {}", i, j, (ax, ay), (bx, by), len);
				sum += len;
			}
		}
		sum
	}
}

impl fmt::Debug for Coordinates {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let (xx, xy) = self.bounds_max();
		write!(f, "\n")?;
		for y in 0..=xy {
			for x in 0..=xx {
				let exists = self.raw.iter().find(|&&(ex, ey)| ex == x && ey == y).is_some();
				if exists {
					write!(f, "#")?;
				} else {
					write!(f, ".")?;
				}
			}
			write!(f, "\n")?;
		}
		Ok(())
	}
}

impl AoCDay for Day11 {
	type Data<'i> = Coordinates;
	type Answer = usize;

	fn day(&self) -> u8 { 11 }

	fn parse<'i>(&self, input: &'i str) -> Self::Data<'i> {
		let coords = input.lines()
			.filter_map(aoch::parsing::trimmed)
			.enumerate()
			.flat_map(|(y, l)| {
				l.chars().enumerate()
					.filter(|(_, c)| *c == '#')
					.map(move |(x, _)| (x, y))
			})
			.collect_vec();
		eprintln!("Coordinates: {} total", coords.len());
		Coordinates { raw: coords }
	}
	fn part1(&self, _data: &mut Self::Data<'_>) -> Self::Answer {
		let mut p1 = _data.clone();
		p1.expand(2);
		p1.pair_dist_sum()
	}
	fn part2(&self, _data: &mut Self::Data<'_>) -> Self::Answer {
		let mut p1 = _data.clone();
		p1.expand(1000000);
		p1.pair_dist_sum()
	}
}

#[cfg(test)]
const TEST_INPUT: &'static str = "
...#......
.......#..
#.........
..........
......#...
.#........
.........#
..........
.......#..
#...#.....
";

#[test]
fn scaled_stars() {
	let cases = [
		(2, 374),
		(10, 1030),
		(100, 8410),
	];
	run_test(|scale| {
		let mut d11 = Day11.parse(TEST_INPUT);
		d11.expand(*scale);
		d11.pair_dist_sum()
	}, &cases);
}

#[test]
fn part1() {
	let cases = [
		(TEST_INPUT, 374),
		(daystr!("11"), 9545480),
	];
	test_runner::<Day11, _>(Day11, DayPart::Part1, &cases);
}

#[test]
fn part2() {
	let cases = [
		// (TEST_INPUT, 0),
		(daystr!("11"), 0),
	];
	test_runner::<Day11, _>(Day11, DayPart::Part2, &cases);
}
