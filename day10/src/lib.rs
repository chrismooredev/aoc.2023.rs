#![feature(ascii_char)]

#![allow(unused_imports)]
use std::collections::hash_map::Entry;
use std::str::FromStr;
use std::fmt::Debug;
use arrayvec::ArrayVec;
use itertools::Itertools;
use test_log::test;
use aoch::AoCDay;
#[cfg(test)] #[allow(unused_imports)]
use aoch::{DayPart, run_test, test_runner, daystr};

use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

// BTreeMap does not make it significantly better
type HashMap<K, V> = std::collections::HashMap<K, V>;

type UCOORD = usize;
type SCOORD = isize;

#[derive(Debug,Clone,Copy)]
pub struct Day10;

#[derive(Debug,Clone)]
struct LineSeeker<'s> {
	raw: &'s str,
	line_idx: usize,
	byte_idx: usize,
}

impl<'s> LineSeeker<'s> {
	fn new(s: &str) -> LineSeeker<'_> {
		LineSeeker { raw: s, line_idx: 0, byte_idx: 0 }
	}
	fn next(&'_ mut self) -> Option<(usize, &'s str)> {
		let raw_forward = &self.raw[self.byte_idx..];
		if raw_forward.is_empty() { return None; }
		let line = raw_forward.lines().next()?;
		let line_idx = self.line_idx;
		self.line_idx += 1;
		self.byte_idx += line.as_bytes().len();
		let mut rest = self.raw[self.byte_idx..].chars();
		match (rest.next(), rest.next()) {
			(Some('\r'), Some('\n')) => { self.byte_idx += 2; },
			(Some('\r'), Some(_)) => { self.byte_idx += 1; },
			(Some('\n'), Some(_)) => { self.byte_idx += 1; },
			(Some('\r'), None) => { unreachable!(".lines iterator should add trailing \\r to yielded string") },
			(Some('\n'), None) => { self.byte_idx += 1; /* put iterator at end */ },
			(_, _) => {},
		};
		Some((line_idx, line))
	}
	fn back(&'_ mut self) -> Option<(usize, &'s str)> {
		let raw_previous = &self.raw[..self.byte_idx];
		if raw_previous.is_empty() { return None; }
		let line = raw_previous.lines().next_back()?;
		log::debug!("{:?}.back().line = {:?} (raw_previous = {:?})", self, line, raw_previous);
		let line_idx = self.line_idx;
		self.line_idx -= 1;
		self.byte_idx -= line.as_bytes().len(); // newline handling?
		Some((line_idx-1, line))
	}
}

#[test]
fn line_seeker() {
	const SAMPLE: &str = "a=0\r\nb=1\nc=2\nd=3\r";
	let mut seeker = LineSeeker::new(SAMPLE);
	assert_eq!(seeker.next(), Some((0, "a=0")));
	assert_eq!(seeker.next(), Some((1, "b=1")));
	assert_eq!(seeker.next(), Some((2, "c=2")));
	assert_eq!(seeker.next(), Some((3, "d=3\r")));
	assert_eq!(seeker.next(), None);

	assert_eq!(seeker.back(), Some((3, "d=3\r")));
	assert_eq!(seeker.back(), Some((2, "c=2")));
	assert_eq!(seeker.back(), Some((1, "b=1")));
	assert_eq!(seeker.back(), Some((0, "a=0")));
	assert_eq!(seeker.back(), None);
}

#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
enum LineSide {
	Left,
	Right
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Direction {
	North,
	East,
	South,
	West,
}

impl Direction {
	const ALL: [Direction; 4] = [
		Direction::North,
		Direction::East,
		Direction::South,
		Direction::West,
	];

	#[inline(always)]
	const fn offset(&self) -> (SCOORD, SCOORD) {
		use Direction::*;
		match self {
			North => (0, -1),
			East => (1, 0),
			South => (0, 1),
			West => (-1, 0),
		}
	}

	#[inline(always)]
	fn forward(&self) -> Direction {
		*self
	}
	#[inline(always)]
	fn behind(&self) -> Direction {
		use Direction::*;
		match self {
			North => South,
			East => West,
			South => North,
			West => East,
		}
	}
	#[inline(always)]
	fn left(&self) -> Direction {
		use Direction::*;
		match self {
			North => West,
			East => North,
			South => East,
			West => South,
		}
	}
	#[inline(always)]
	fn right(&self) -> Direction {
		use Direction::*;
		match self {
			North => East,
			East => South,
			South => West,
			West => North,
		}
	}

	/// Returns a tuple of connected directions.
	fn symbol_connections(c: char) -> Option<[Direction; 2]> {
		match c {
			'|' => Some([Direction::North, Direction::South]),
			'-' => Some([Direction::East, Direction::West]),
			'L' => Some([Direction::North, Direction::East]),
			'J' => Some([Direction::North, Direction::West]),
			'7' => Some([Direction::South, Direction::West]),
			'F' => Some([Direction::South, Direction::East]),
			_ => None,
		}
	}
}

impl std::ops::Add<Direction> for (SCOORD, SCOORD) {
    type Output = (SCOORD, SCOORD);

    fn add(self, rhs: Direction) -> Self::Output {
        let (ox, oy) = rhs.offset();
		(self.0 + ox, self.1 + oy)
    }
}

impl std::ops::Add<Direction> for (UCOORD, UCOORD) {
    type Output = Option<(UCOORD, UCOORD)>;

    fn add(self, rhs: Direction) -> Self::Output {
        let (x, y) = (self.0 as SCOORD, self.1 as SCOORD);
		let (ox, oy) = rhs.offset();
		Some((
			(x + ox).try_into().ok()?,
			(y + oy).try_into().ok()?,
		))
    }
}

#[derive(Debug,Clone)]
pub struct Maze<'s>(&'s [std::ascii::Char], usize);

impl<'s> Maze<'s> {
	fn get(&self, x: UCOORD, y: UCOORD) -> Option<char> {
		self.0.get((y as usize)*(self.1) + (x as usize)).map(|c| c.to_char())
	}

	fn get_around(&self, x: UCOORD, y: UCOORD, dir: Direction) -> Option<((UCOORD, UCOORD), char)> {
		let (ox, oy) = ((x, y) + dir)?;
		self.get(ox, oy).map(|c| ((ox, oy), c))
	}

	/// Returns a walk around a tile in NESW order, yielding items of ((x, y), walked direction, dest symbol)
	fn around(&self, x: UCOORD, y: UCOORD) -> impl Iterator<Item = ((UCOORD, UCOORD), Direction, char)> + '_ {
		Direction::ALL.into_iter()
			.flat_map(move |dir| {
				// apply offset w/ lower bounds (type) checking
				let (x, y) = ((x, y) + dir)?;

				// query character w/ upper bounds (index) checking
				self.get(x, y).map(|c| ((x, y), dir, c))
			})
	}

	/// Returns an iterator of valid neighboring tiles that connect back to the provided coordinates
	/// Yields ((neighbor_x, neighbor_y), in_direction, leads_towards_direction)
	fn tile_connections(&self, x: UCOORD, y: UCOORD) -> impl Iterator<Item = ((UCOORD, UCOORD), Direction, Direction)> + '_ {
		self.around(x, y)
			.filter(|(_, _, nc)| *nc != '.')
			.flat_map(move |((nx, ny), ndir, nc)| {
				// what direction does this tile need to connect with start, relative to itself?
				// let towards_self = ndir.opposite();
				log::debug!("Maze::tile_connections({}, {}) visiting neighbor ({}, {}, char: {:?}, dir: {:?})", x, y, nx, ny, nc, ndir);

				let to_center = ndir.behind();
				Direction::symbol_connections(nc).into_iter()
					// Goes back towards origin
					.filter(|arr| arr.contains(&to_center))
					.flatten()
					// Goes away from the origin
					.filter(|d| *d != to_center)
					.map(|d| ((nx, ny), ndir, d))
					.next()
			})
	}
	fn find_start(&self) -> Option<(UCOORD, UCOORD)> {
		self.0.iter()
			.position(|c| c.to_char() == 'S')
			.map(|i| ((i % self.1) as UCOORD, (i/self.1) as UCOORD))
	}
	fn walk_path(&self) -> impl Iterator<Item = ((UCOORD, UCOORD), Direction, char)> + '_ {
		let start = self.find_start().expect("cannot walk a path with no start");

		let (dir1, dir2) = self.tile_connections(start.0, start.1).collect_tuple()
			.expect("Expected two connections to starting tile");

		log::debug!("Found tiles connecting to start at {:?}, in {:?} and {:?} directions", start, dir1, dir2);

		#[derive(Debug)]
		struct PathWalker<'m> {
			maze: &'m Maze<'m>,
			start: (UCOORD, UCOORD),
			position: (UCOORD, UCOORD),
			next_direction: Option<Direction>,
		}
		impl<'m> Iterator for PathWalker<'m> {
			type Item = ((UCOORD, UCOORD), Direction, char);
			fn next(&mut self) -> Option<Self::Item> {
				// eprintln!("PathWalker::next({:?})", self);
				let last_pos = self.position;
				let move_dir = self.next_direction?;
				let Some((next_x, next_y)) = self.position + move_dir else {
					panic!("next_direction targeted tile beyond lower limits of map ({:?})", self);
				};

				if (next_x, next_y) == self.start {
					log::trace!("reached end of walk {:?}", self);
					self.next_direction = None;
					// let rtn = (last_pos, self.maze.get(last_pos.0, last_pos.1).unwrap());
					// eprintln!("\treturnning {:?}", rtn);
					// return Some(rtn);
				} else {
					// Get the next symbols other direction
					let (next_dir,) = self.maze.get(next_x, next_y)
						.map(Direction::symbol_connections)
						.iter().flatten().flatten().copied()
						.filter(|d| d.behind() != move_dir)
						.collect_tuple()
						.unwrap_or_else(|| {
							panic!("found zero or over one targets for next direction from {:?} ({:?})", (next_x, next_y), self);
						});

					log::trace!("\tnext state: {:?} going {:?}", (next_x, next_y), next_dir);
					self.position = (next_x, next_y);
					self.next_direction = Some(next_dir);
				}

				let rtn_char = self.maze.get(last_pos.0, last_pos.1)
					.expect("last PathWalkter iteration stored invalid position coordinates");
				Some((last_pos, move_dir, rtn_char))
			}
		}

		PathWalker {
			maze: self,
			start: start,
			position: start,
			next_direction: Some(dir1.1),
		}
	}
}

impl AoCDay for Day10 {
	type Data<'i> = Maze<'i>;
	type Answer = usize;

	fn day(&self) -> u8 { 10 }

	fn parse<'i>(&self, input: &'i str) -> Self::Data<'i> {
		let raw = input.trim();
		// eprintln!("Debug:\n{:?}", raw);
		// eprintln!("Display:\n{}", raw);
		let ascii = raw.as_ascii().expect("input not ascii");
		let width_eqs = raw.lines()
			.map(|l| l.len())
			// .enumerate()
			// .inspect(|(i, len)| eprintln!("Line {}: {}", i, len))
			// .map(|(_i, len)| len)
			.all_equal();
		if ! width_eqs {
			panic!("not all input lines are of equal length");
		}
		let width = raw.lines().map(|l| l.len()+1).next();

		Maze(ascii, width.expect("no input lines"))
	}
	fn part1(&self, _data: &mut Self::Data<'_>) -> Self::Answer {
		use Direction::{North, East, South, West};

		// eprintln!("Start position: {:?}", _data.find_start());
		_data.walk_path().count().div_ceil(2)
	}
	fn part2(&self, _data: &mut Self::Data<'_>) -> Self::Answer {
		let path_ordered = _data.walk_path()
			// .inspect(|t| eprintln!("path node: {:?}", t))
			.collect_vec();
		let path: HashMap<(UCOORD, UCOORD), (Direction, char)> = path_ordered.iter()
			.map(|&(xy, d, c)| (xy, (d, c)))
			.collect();

		// eprintln!("path:");
		// for (i, p) in path.iter().enumerate() {
		// 	eprintln!("\t{i}: {p:?}");
		// }

		let bounds_x_raw = path_ordered.iter().minmax_by_key(|p| p.0.0).into_option().unwrap();
		let bounds_y_raw = path_ordered.iter().minmax_by_key(|p| p.0.1).into_option().unwrap();
		let (minx, maxx) = (bounds_x_raw.0.0.0, bounds_x_raw.1.0.0);
		let (miny, maxy) = (bounds_y_raw.0.0.1, bounds_y_raw.1.0.1);
		// path.sort();

		let mut next_group_id: u32 = 0;
		let mut others: HashMap<(UCOORD, UCOORD), u32> = HashMap::with_capacity(((maxy-miny)*(maxx-minx)) as usize);

		// walk each coordinate, categorizing each non-path tile into contiguous groups
		for y in miny..=maxy {
			for x in minx..=maxx {
				if path.get(&(x, y)).is_some() { continue; }

				let mut known_neighbors: ArrayVec<_, 4> = _data.around(x, y)
					.filter(|&((ax, ay), d, c)| !path.contains_key(&(ax,ay)))
					.filter_map(|((ax,ay), d, c)| others.get(&(ax,ay)).map(|gid| {
						((ax,ay),d,c,*gid)
					}))
					.collect();

				let same_gid = known_neighbors.iter()
					.map(|(_, _, _, gid)| *gid)
					.all_equal();

				if known_neighbors.is_empty() {
					others.insert((x,y), next_group_id);
					next_group_id += 1;
				} else if same_gid {
					others.insert((x,y), known_neighbors[0].3);
				} else {
					// multiple neighbors with differing group IDs
					// make smaller group ID, consume the larger group IDs
					let smallest_group_id = known_neighbors.iter()
						.map(|(_,_,_,gid)| *gid)
						.min().unwrap();
					for ((ax,ay),d,c,lgid) in known_neighbors.iter_mut() {
						others.iter_mut()
							.filter(|(_,oid)| *oid == lgid)
							.for_each(|(_,oid)| {
								// deallocate greater group ID for reuse
								if next_group_id == *oid+1 {
									next_group_id -= 1;
								}

								*oid = smallest_group_id;
							});
						*lgid = smallest_group_id;
					}

					others.insert((x,y), smallest_group_id);
				}
			}
		}

		// put each group's coordinates into a Vec
		let mut groups = HashMap::new();
		for (coord, gid) in others.iter() {
			groups.entry(*gid)
				.or_insert_with(|| Vec::with_capacity(8))
				.push(*coord);
		}

		let mut group_affinities = HashMap::new();

		macro_rules! check_lineside {
			($sx:expr, $sy:expr, $traveled_dir:expr, $side:expr) => {
				if let Some(((ox, oy), _c)) = _data.get_around($sx, $sy, $traveled_dir) {
					if let Some(gid) = others.get(&(ox, oy)) {
						assert!(!path.contains_key(&(ox, oy)), "tried to assign line affinity to pathful tile");
						let og_side = group_affinities.entry(*gid)
							.or_insert($side);
						if *og_side != $side {
							panic!("line group moved sides!");
						}
					}
				}
			};
			($sx:expr, $sy:expr, $traveled_dir:expr, $side:expr, { $($direction:ident),+ $(,)? }) => {{
				$(
					check_lineside!($sx, $sy, $traveled_dir.$direction(), $side);
				)+
			}};
		}

		// walk the path again, categorizing each group
		let mut path_ordered_iter = path_ordered.iter();
		let mut dir = path_ordered_iter.next().unwrap();
		// walk through, tracking left/right of the path
		for &((sx, sy), traveled_dir, s) in path_ordered_iter {
			match (s, traveled_dir) {

				('J', Direction::North) => check_lineside!(sx, sy, traveled_dir, LineSide::Right, { right, behind, }),
				('L', Direction::North) => check_lineside!(sx, sy, traveled_dir, LineSide::Left, { left, behind, }),

				('L', Direction::East) => check_lineside!(sx, sy, traveled_dir, LineSide::Right, { right, behind, }),
				('F', Direction::East) => check_lineside!(sx, sy, traveled_dir, LineSide::Left, { left, behind, }),

				('F', Direction::South) => check_lineside!(sx, sy, traveled_dir, LineSide::Right, { right, behind, }),
				('7', Direction::South) => check_lineside!(sx, sy, traveled_dir, LineSide::Left, { left, behind, }),

				('7', Direction::West) => check_lineside!(sx, sy, traveled_dir, LineSide::Right, { right, behind, }),
				('J', Direction::West) => check_lineside!(sx, sy, traveled_dir, LineSide::Left, { left, behind, }),

				('|' | '-', _) => {
					check_lineside!(sx, sy, traveled_dir.left(), LineSide::Left);
					check_lineside!(sx, sy, traveled_dir.right(), LineSide::Right);
				},

				(_, _) => {
					// ignore
				}
			}
		}

		let mut affinities = HashMap::<LineSide, usize>::new();
		for (gid, side) in group_affinities.iter() {
			log::debug!("Group {}: {:?}", gid, side);
			*affinities.entry(*side)
				.or_default() += groups.get(&gid).unwrap().len();
		}

		log::debug!("groups affinities: {:?}", group_affinities);
		log::debug!("affinities: {:?}", affinities);

		// walk each tile, to print it to the console

		if log::log_enabled!(log::Level::Info) {
			let mut stdout = StandardStream::stdout(ColorChoice::Always);
			const UNICODE: bool = false;
			for y in miny..=maxy {
				for x in minx..=maxx {
					let (c, opt_grp) = path.get(&(x, y))
						.map(|(_d, c)| (match c {
							'S' => 'S',
							'-' if UNICODE => '─',
							'7' if UNICODE  => '┐',
							'J' if UNICODE => '┘',
							'L' if UNICODE  => '┕',
							'F' if UNICODE  => '┌',
							_ => *c,
						}, None))
						.unwrap_or_else(|| {
							let gid = *others.get(&(x, y)).expect("non-path tile not filled in others");
							const CHARS: &str = "0123456789*@a";
							let c = if (gid as usize) < CHARS.len() {
								CHARS.chars().nth(gid as usize).unwrap()
							} else {
								'•'
							};
							(c, Some(gid))
						});

					let mut spec = ColorSpec::new();
					match opt_grp {
						None => { spec.set_reset(true); },
						Some(gid) => {
							match group_affinities.get(&gid) {
								None => spec.set_fg(Some(Color::Red)),
								Some(LineSide::Left) => spec.set_fg(Some(Color::Magenta)),
								Some(LineSide::Right) => spec.set_fg(Some(Color::Cyan)),
							};
						}
					}

					match (opt_grp, path.get(&(x, y))) {
						(Some(gid), _) => match group_affinities.get(&gid) {
							None => spec.set_fg(Some(Color::Red)),
							Some(LineSide::Left) => spec.set_fg(Some(Color::Magenta)),
							Some(LineSide::Right) => spec.set_fg(Some(Color::Cyan)),
						},
						(_, Some((dir, _))) => {
							spec.set_bg(Some(Color::Rgb(50, 50, 50)));
							match dir {
								Direction::North => spec.set_fg(Some(Color::Blue)),
								Direction::East => spec.set_fg(Some(Color::Cyan)),
								Direction::South => spec.set_fg(Some(Color::Red)),
								Direction::West => spec.set_fg(Some(Color::Magenta)),
							}
						},
						_ => panic!("tile not categorized as path nor group"),
					};

					stdout.set_color(&spec).unwrap();
					write!(&mut stdout, "{}", c).unwrap();
					stdout.set_color(ColorSpec::new().set_reset(true)).unwrap();
					// print!("{}", c);
				}
				writeln!(&mut stdout, "").unwrap();
			}
		}

		*affinities.values()
			.min().unwrap()
	}
}

#[cfg(test)]
const TEST_INPUT_P1_SIMPLE: &'static str = "
.....
.S-7.
.|.|.
.L-J.
.....
";


#[cfg(test)]
const TEST_INPUT_P1_COMPLEX: &'static str = "
..F7.
.FJ|.
SJ.L7
|F--J
LJ...
";

#[cfg(test)]
const TEST_INPUT_P2_SIMPLE: &'static str = "
...........
.S-------7.
.|F-----7|.
.||.....||.
.||.....||.
.|L-7.F-J|.
.|..|.|..|.
.L--J.L--J.
...........
";

#[cfg(test)]
const TEST_INPUT_P2_LARGE: &'static str = "
.F----7F7F7F7F-7....
.|F--7||||||||FJ....
.||.FJ||||||||L7....
FJL7L7LJLJ||LJ.L-7..
L--J.L7...LJS7F-7L7.
....F-J..F7FJ|L7L7L7
....L7.F7||L7|.L7L7|
.....|FJLJ|FJ|F7|.LJ
....FJL-7.||.||||...
....L---J.LJ.LJLJ...
";

#[cfg(test)]
const TEST_INPUT_P2_WITHTRASH: &'static str = "
FF7FSF7F7F7F7F7F---7
L|LJ||||||||||||F--J
FL-7LJLJ||||||LJL-77
F--JF--7||LJLJ7F7FJ-
L---JF-JLJ.||-FJLJJ7
|F|F-JF---7F7-L7L|7|
|FFJF7L7F-JF7|JL---7
7-L-JL7||F7|L7F-7F7|
L.L7LFJ|||||FJL7||LJ
L7JLJL-JLJLJL--JLJ.L
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
		(TEST_INPUT_P1_SIMPLE, 4),
		(TEST_INPUT_P1_COMPLEX, 8),
		(daystr!("10"), 6864),
	];
	test_runner::<Day10, _>(Day10, DayPart::Part1, &cases);
}
#[test]
fn part2() {
	let cases = [
		(TEST_INPUT_P2_SIMPLE, 4),
		(TEST_INPUT_P2_LARGE, 8),
		(TEST_INPUT_P2_WITHTRASH, 10),
		(daystr!("10"), 349),
	];
	test_runner::<Day10, _>(Day10, DayPart::Part2, &cases);
}
