#![feature(assert_matches)]
#![allow(unused_imports)]
use core::fmt;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::marker::PhantomData;
use std::ops::Range;
use std::{str::FromStr, collections::HashMap};
use std::fmt::Debug;
use itertools::Itertools;
use test_log::test;
use aoch::AoCDay;
#[cfg(test)] #[allow(unused_imports)]
use aoch::{DayPart, run_test, test_runner, daystr};

#[derive(Debug,Clone,Copy)]
pub struct Day05;

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Almanac {
	seeds: Vec<usize>,
	mappings: Vec<Layer>,
	cached: BTreeMap<usize, usize>,
}

impl Almanac {
	fn process(&self, seed: usize) -> AlmanacLookup<'_> {
		AlmanacLookup::new(&self.mappings, seed)
	}
	fn by_divisible_range(&self, range: Range<usize>) -> Vec<Vec<(isize, Range<usize>)>> {
		fn inner<F: FnMut(&[(isize, Range<usize>)])>(mappings: &[Layer], stack: &mut Vec<(isize, Range<usize>)>, search: (isize, Range<usize>), visit: &mut F) {
			use Ordering::{Less, Equal, Greater};
			let (last_offset, search) = search.clone();
			let depthstr: String = (1..=stack.len()).map(|_| '\t').collect();

			let root_seed = stack.first().cloned().unwrap_or((last_offset, search.clone())).1;
			log::debug!("seed[_, {root_seed:?}] {depthstr:}searching for match to {}[{search:?}]->{}",
				mappings.first().map(|s| s.src_type.as_str()).unwrap_or("location"),
				mappings.first().map(|s| s.dst_type.as_str()).unwrap_or("<end>"),
			);

			stack.push((last_offset, search.clone()));

			match mappings.split_first() {
				None => {
					/* nothing left */
					eprintln!("seed[_, {:?}] {depthstr:}found -> {:?}",
						stack.first().unwrap_or(&(last_offset, search.clone())).1,
						search,
					);
					visit(stack.as_slice());
				},
				Some((map, rest)) => {
					for result in map.search_segments(search.clone()) {
						inner(rest, stack, result, visit);
					}
				}
			}

			let popped = stack.pop();
			assert_eq!(popped.expect("stack unexpectedly emptied"), (last_offset, search), "stack not appropriately kept");
		}

		let mut scratch = Vec::with_capacity(self.mappings.len());
		let mut results = Vec::new();
		inner(&self.mappings, &mut scratch, (0, range.clone()), &mut |stack| {
			// eprintln!("seed range[{:?}] result[{}] = {:?}", range, results.len(), stack);
			results.push(stack.to_vec());
		});
		assert_eq!(scratch.len(), 0, "stack not emptied when done");
		results
	}
}

struct AlmanacLookup<'a> {
	// alamanac: &'a Almanac,
	layers: &'a [Layer],
	last_type: &'a str,
	last_value: usize,
}
impl<'a> AlmanacLookup<'a> {
	fn new(mappings: &[Layer], seed: usize) -> AlmanacLookup {
		AlmanacLookup {
			layers: mappings,
			last_type: "seed",
			last_value: seed
		}
	}
}
impl<'a> Iterator for AlmanacLookup<'a> {
	type Item = (&'a str, usize);
	fn next(&mut self) -> Option<Self::Item> {
		// terminate if there isn't a next type
		// let map = self.alamanac.mappings.iter()
		let map = self.layers.iter()
			.find(|p| p.src_type == self.last_type)?;
		let mapped = map.map(self.last_value);
		self.last_value = mapped;
		self.last_type = map.dst_type.as_str();

		Some((self.last_type, self.last_value))
	}
}

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
struct Layer {
	src_type: String,
	dst_type: String,
	ranges: Vec<Segment>
}
impl Layer {
	fn find(&self, value: usize) -> Option<usize> {
		let m = self.ranges.iter()
			.find(|m| m.src <= value && value < m.src + m.len)?;
		Some(value - m.src + m.dst)
	}
	fn map(&self, value: usize) -> usize {
		self.find(value).unwrap_or(value)
	}
	fn check_overlapping(&mut self) -> bool {
		self.ranges.sort_by_key(|m| m.src);
		if self.ranges.is_empty() { return false; }
		let first_end = { let k = self.ranges[0]; k.src+k.len };
		let (overlapping, _) = self.ranges.iter()
			.skip(1)
			.fold((false, first_end), |acc, r| {
				let (overlapping, last_end) = acc;
				if !overlapping && r.src < last_end {
					(true, 0)
				} else {
					(false, r.src+r.len)
				}
			});
		overlapping
	}
	fn search_segments(&self, search: Range<usize>) -> SegmentMapper {
		SegmentMapper { search: Some(search), segments: self.ranges.as_slice() }
	}
}

#[derive(Debug, Clone)]
struct SegmentMapper<'r> {
	search: Option<Range<usize>>,
	segments: &'r [Segment],
}
impl Iterator for SegmentMapper<'_> {
	type Item = (isize, Range<usize>);
	fn next(&mut self) -> Option<Self::Item> {
		use std::cmp::Ordering::{Greater, Equal, Less};
		// assumes the ranges are ordered
		while let Some((mr, rest)) = self.segments.split_first() {
			let Range { start: ss, end: se } = self.search.as_mut()?;
			let Range { start: rs, end: re } = mr.range();

			assert!(ss <= se);
			assert!(rs <= re);

			let ordering = ((*ss).cmp(&rs), (*se).cmp(&rs), (*ss).cmp(&re), (*se).cmp(&re));

			return Some(match ordering {
				// action 1 | starts before segment, does not enter
				// action 2 | starts before segment, terminates inside or goes past
				// action 3 | starts with segment, ends before or at segment end
				// action 4 | starts with segment, goes past segment end
				// action 5 | starting after segment end

				(Less, Less|Equal, Less|Equal, Less|Equal) => { // 1: starts before segment, does not enter
				// (_/* Less|Equal */, Less|Equal, _ /* Less */, _ /* Less */) => { // starts before segment, does not enter
					let rtn = *ss..*se; // return it as is, before sorted segments
					self.search = None; // stop operations
					self.segments = rest;
					(0, rtn) // no offset, first segment
				},
				(Less, Greater, Less|Equal, Less|Equal|Greater) => { // 2: starts before segment, terminates inside or goes past
					let rtn = *ss..rs; // save first segment
					*ss = rs; // search start for next loop
					(0, rtn) // no offset, first segment
				},
				(Equal|Greater, Greater|Equal, Less|Equal, Less|Equal) => { // 3: starts with segment, ends before or at segment end
					// let rtn = *ss..*se; // return as-is
					let rtn = mr.rangemap_opt(*ss..*se).unwrap(); // save mapped
					self.search = None; // clear search, we're done
					self.segments = &[];
					(mr.offset(), rtn) // yield mapped
				},
				(Equal|Greater, Greater|Equal, Less|Equal, Greater) => { // 4: starts with segment, goes past end
					let rtn = mr.rangemap_opt(*ss..re).unwrap();
					*ss = re; // update search
					self.segments = rest; // update segments
					(mr.offset(), rtn) // yield mapped
				},
				(Greater, Greater, Greater, Greater) => { // 5: starts after segment end
					self.segments = rest;
					continue;
				},

				_ => {
					unreachable!("Found unhandled search/segment ordering: {:?}", ordering);
				}
			});
		}

		// Handle case when the search is after all ranges
		self.search.take().map(|r| (0, r))
	}
}

#[test]
fn mapped_ranges() {
	use std::assert_matches::assert_matches;

	let almanac = Day05.parse(TEST_INPUT);

	let seed2soil = &almanac.mappings[0];
	assert_eq!(seed2soil.src_type, "seed", "test data changed");
	assert_eq!(seed2soil.dst_type, "soil", "test data changed");
	assert_eq!(seed2soil.ranges, vec![
		Segment { dst: 52, src: 50, len: 48 },
		Segment { dst: 50, src: 98, len: 2 }
	], "test data changed");
	assert_eq!(seed2soil.ranges[0].range(), 50..98, "test data changed");
	assert_eq!(seed2soil.ranges[0].offset(), 2, "test data changed");
	assert_eq!(seed2soil.ranges[1].range(), 98..100, "test data changed");
	assert_eq!(seed2soil.ranges[1].offset(), -48, "test data changed");

	macro_rules! check_ranges {
		($mapping:expr, $src:expr, $msg:literal, (), &[ $(($o:literal, $s:literal .. $e:literal),)* ]) => {
			assert_matches!(
				$mapping.search_segments($src)
					.enumerate()
					.inspect(|(i, r)| eprintln!("[{} / {:?}][{}] = {:?}", $msg, $src, i, r))
					.map(|(_, r)| r)
					.collect_vec()
					.as_slice(),
				&[
				$(($o, Range { start: $s, end: $e })),*
			], $msg);
		}
	}

	check_ranges!(seed2soil, 0..40, "search before ranges", (), &[(0, 0..40),]);
	check_ranges!(seed2soil, 300..400, "search after ranges", (), &[(0, 300..400),]);
	check_ranges!(seed2soil, 25..75, "search across range start", (), &[
		(0, 25..50),
		(2, 52..77),
	]);
	check_ranges!(seed2soil, 25..98, "search until consecutive segment starts", (), &[
		(0, 25..50),
		(2, 52..100), // 50..98
	]);
	check_ranges!(seed2soil, 25..99, "search until one-past consecutive segment starts", (), &[
		(0, 25..50),
		(2, 52..100), // 50..98
		(-48, 50..51), // 98..99
	]);
	check_ranges!(seed2soil, 50..100, "search starts at segment, terminates in next segment", (), &[
		(2, 52..100), // 50..98
		(-48, 50..52), // 98..100
	]);
	check_ranges!(seed2soil, 50..200, "search starts at segment, passes through another, and terminates outside", (), &[
		(2, 52..100), // 50..98
		(-48, 50..52), // 98..100
		(0, 100..200), // 100..200
	]);

	let with_hole = Layer { src_type: "a".into(), dst_type: "b".into(), ranges: vec![
		Segment { src: 25, len: 5, dst: 100 },
		Segment { src: 40, len: 10, dst: 200 },
	] };
	check_ranges!(with_hole, 0..100, "search starts before segments, goes through fragmented segments, and terminates outside", (), &[
		(0, 0..25), // 0..25
		(75, 100..105), // 25..30
		(0, 30..40), // 30..40
		(160, 200..210), // 40..50
		(0, 50..100), // 50..100
	]);

}

#[test]
fn overlapping_maps() {
	let mut overlapping = Layer { src_type: "src".into(), dst_type: "dst".into(), ranges: vec![
		Segment { src: 0, len: 16, dst: 64 },
		Segment { src: 8, len: 16, dst: 72 },
	]};

	let mut contiguous = Layer { src_type: "src".into(), dst_type: "dst".into(), ranges: vec![
		Segment { src: 0, len: 16, dst: 64 },
		Segment { src: 16, len: 16, dst: 64+16 },
	]};

	let mut separate = Layer { src_type: "src".into(), dst_type: "dst".into(), ranges: vec![
		Segment { src: 0, len: 16, dst: 64 },
		Segment { src: 32, len: 16, dst: 64+32 },
	]};

	assert_eq!(overlapping.check_overlapping(), true);
	assert_eq!(contiguous.check_overlapping(), false);
	assert_eq!(separate.check_overlapping(), false);
}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
struct Segment {
	dst: usize,
	src: usize,
	len: usize,
}
impl FromStr for Segment {
	type Err = std::num::ParseIntError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (dst, src, len) = s.trim()
			.split(' ')
			.map(|s| s.parse())
			.collect_tuple()
			.unwrap();
		Ok(Segment { dst: dst?, src: src?, len: len? })
	}
}
impl Segment {
	/// Attempts to convert a range using this map. If the range only partially overlaps, then
	/// this returns None. If the range is outside then it is returned as is. If fully contained, then it is mapped
	/// according to this range.
	fn rangemap_opt(&self, orange: Range<usize>) -> Option<Range<usize>> {
		let srange = self.src..self.src+self.len;

		let overlaps_start = srange.start <= orange.start && orange.start < srange.end;
		let overlaps_end = srange.start < orange.end && orange.end <= srange.end;

		match (overlaps_start, overlaps_end) {
			(true, true) => Some(Range {
				start: orange.start + self.dst - self.src,
				end: orange.end + self.dst - self.src,
			}),
			(false, false) => Some(orange),
			_ => None
		}
	}
	fn range(&self) -> Range<usize> {
		self.src..self.src+self.len
	}
	fn offset(&self) -> isize {
		(self.dst as isize) - (self.src as isize)
	}
}
impl fmt::Display for Segment {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}:{:+}", self.range(), self.offset())
	}
}

struct MapIndexed<I, T, U, F>(I, F, PhantomData<(T, U)>);

impl<I, T, U, F> Iterator for MapIndexed<I, T, U, F>
where
	I: Iterator<Item = (usize, T)>,
	F: FnMut(T) -> U,
{
	type Item = (usize, U);
	fn next(&mut self) -> Option<Self::Item> {
		let (i, old) = self.0.next()?;
		let new = self.1(old);
		Some((i, new))
	}
}

trait LocalIterExt: Iterator {
	fn map_enum<T, U, F: FnMut(T) -> U>(self, map: F) -> MapIndexed<Self, T, U, F>
	where
		Self: Sized + Iterator<Item=(usize, T)>
	{
		MapIndexed(self, map, PhantomData)
	}
}
impl<I: Iterator> LocalIterExt for I {}

impl AoCDay for Day05 {
	type Data<'i> = Almanac;
	type Answer = usize;

	fn day(&self) -> u8 { 5 }

	fn parse<'i>(&self, input: &'i str) -> Self::Data<'i> {
		let mut lines = input.lines().filter_map(aoch::parsing::trimmed);
		let raw_seeds = lines.next().expect("no seed string");
		assert!(raw_seeds.starts_with("seeds: "));
		let seeds = raw_seeds[7..].split(' ')
			.map(|s| s.parse::<usize>().unwrap())
			.collect_vec();

		// let mut mappings = HashMap::default();
		let mut mappings = Vec::default();
		let mut group: (String, String, Vec<Segment>) = Default::default();

		for l in lines {
			if l.ends_with("map:") {
				if !group.2.is_empty() {
					let (src, dst, maps) = group;
					mappings.push(Layer { src_type: src, dst_type: dst, ranges: maps });
					// mappings.insert((src, dst), maps);
				}

				let (raw_desc, _map) = l.split_once(' ').unwrap();
				let (src, _, dst) = raw_desc.split('-').collect_tuple().unwrap();
				group = (src.to_owned(), dst.to_owned(), Vec::new());
			} else {
				let (dst, src, len) = l.split(' ')
					.map(|s| s.parse::<usize>().unwrap())
					.collect_tuple().unwrap();

				group.2.push(Segment { dst, src, len })
			}
		}
		if !group.2.is_empty() {
			let (src, dst, maps) = group;
			mappings.push(Layer { src_type: src, dst_type: dst, ranges: maps });
		}

		mappings.iter_mut()
			.for_each(|m| {
				let overlaps = m.check_overlapping();
				log::debug!("Range {}-{} overlaps: {} ({} ranges)", m.src_type, m.dst_type, overlaps, m.ranges.len());
				assert!(!overlaps, "range {}-{} overlaps", m.src_type, m.dst_type);
			});

		Almanac { seeds, mappings, cached: BTreeMap::default() }
	}
	fn part1(&self, _data: &mut Self::Data<'_>) -> Self::Answer {
		_data.seeds.iter()
			.map(|seed| {
				let (t, v) = _data.process(*seed).last().unwrap();
				assert_eq!(t, "location");
				v
			})
			.min().unwrap()
	}

	fn part2(&self, _data: &mut Self::Data<'_>) -> Self::Answer {
		// have to mutably borrow the almanac so just copy the otherwise read data
		let ranges = _data.seeds.chunks_exact(2)
			.map(<[_; 2]>::try_from)
			.map(Result::unwrap)
			.collect_vec();

		let results = ranges.iter()
			.enumerate()
			.map_enum(|&[start, len]| start..start+len)
			// .inspect(|(i, seed)| eprintln!("seed[{}, {:?}] starting search", i, seed))
			.map_enum(|range| (range.clone(), _data.by_divisible_range(range)))
			.flat_map(|(si, (seed, derivations))| {
				derivations.into_iter()
					.enumerate()
					// .inspect(move |(path_idx, path)| eprintln!("seed[{}, {:?}][{}].raw = {:?}", si, sr1, path_idx, path))
					.map_enum(|path| {
						path.last().unwrap().1.clone() // interested in the last location value
					})
					// .inspect(move |(ii, s)| eprintln!("seed[{}, {:?}][{}] =-> {:?}", si, sr1.clone(), ii, s))
					.map(move |(li, loc)| (si, seed.clone(), li, loc))
			})
			// .inspect(|(si, sr, li, lr)| eprintln!("seed[{}, {:?}][{}].result = {:?}", si, sr, li, lr))
			.collect_vec();

		let best = results.iter()
			.min_by_key(|(_si, _sr, _li, lr)| lr.start)
			.expect("no results found");

		eprintln!("total of {} results found", results.len());

		best.3.start
	}
}

#[cfg(test)]
const TEST_INPUT: &'static str = "
seeds: 79 14 55 13

seed-to-soil map:
50 98 2
52 50 48

soil-to-fertilizer map:
0 15 37
37 52 2
39 0 15

fertilizer-to-water map:
49 53 8
0 11 42
42 0 7
57 7 4

water-to-light map:
88 18 7
18 25 70

light-to-temperature map:
45 77 23
81 45 19
68 64 13

temperature-to-humidity map:
0 69 1
1 0 69

humidity-to-location map:
60 56 37
56 93 4
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
		(TEST_INPUT, 35),
		(daystr!("05"), 278755257),
	];
	test_runner::<Day05, _>(Day05, DayPart::Part1, &cases);
}

#[test]
fn part2() {
	let cases = [
		(TEST_INPUT, 46),
		(daystr!("05"), 26829166),
	];
	test_runner::<Day05, _>(Day05, DayPart::Part2, &cases);
}
