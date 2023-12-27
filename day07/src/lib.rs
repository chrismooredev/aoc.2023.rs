#![allow(unused_imports)]
use std::cmp::Reverse;
use std::{str::FromStr, cmp::Ordering};
use std::fmt::Debug;
use itertools::Itertools;
use num_traits::{FromPrimitive, ToPrimitive};
use num_derive::{FromPrimitive, ToPrimitive};
use test_log::test;
use arrayvec::ArrayVec;
use aoch::AoCDay;
#[cfg(test)] #[allow(unused_imports)]
use aoch::{DayPart, run_test, test_runner, daystr};

#[derive(Debug,Clone,PartialEq,Eq,PartialOrd,Ord,FromPrimitive,ToPrimitive)]
enum Card {
	N2 = 2, N3, N4, N5, N6, N7, N8, N9,
	T, J, Q, K, A,
}
impl Card {
	fn rank(&self, wildcard_joker: bool) -> u32 {
		match (wildcard_joker, self) {
			(true, Card::J) => 1,
			_ => ToPrimitive::to_u32(self).unwrap(),
		}
	}
}

#[derive(Debug,Clone,PartialEq,Eq,PartialOrd,Ord)]
enum HandType {
	FiveOfAKind,
	FourOfAKind,
	FullHouse,
	ThreeOfAKind,
	TwoPair,
	OnePair,
	HighCard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Play {
	cards: [Card; 5],
	bid: usize,
}

impl FromStr for Play {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use Card::*;

		let (cards, bid) = s.trim().split_once(' ').unwrap();
		let bid = bid.parse().unwrap();

		let (c1, c2, c3, c4, c5) = cards.chars()
			.map(|c| match c {
				'2'..='9' => FromPrimitive::from_u32((c as u32) - ('0' as u32)).unwrap(),
				'T' => T,
				'J' => J,
				'Q' => Q,
				'K' => K,
				'A' => A,
				_ => panic!("unexpected card character: {:?}", c),
			})
			.collect_tuple().unwrap();

		Ok(Play { cards: [c1, c2, c3, c4, c5], bid })
	}
}

impl Play {
	fn hand_type(&self, wildcard_joker: bool) -> HandType {
		let mut cards = self.cards.clone();
		cards.sort(); // sort by card rank
		let mut partitions: ArrayVec<(usize, Card), 5> = cards.iter()
			.cloned()
			.dedup_with_count()
			.collect();
		partitions.sort_by_key(|p| Reverse(p.clone())); // sort by group length

		log::trace!("hand_type[{:?}], partitions = {:?}", self, partitions);

		if wildcard_joker {
			let opt_joker = partitions.iter()
				.cloned()
				.find_position(|(i, c)| *i < 5 && *c == Card::J)
				.clone();
			if let Some((i, pair @ (count, _))) = opt_joker {
				let removed = partitions.remove(i);
				assert_eq!(pair, removed);
				partitions[0].0 += count;
			}
		}

		match partitions.as_slice() {
			[(5, _)] => HandType::FiveOfAKind, // all five are eq
			[(4, _), (1, _)] => HandType::FourOfAKind,
			[(3, _), (2, _)] => HandType::FullHouse,
			[(3, _), (1, _), (1, _)] => HandType::ThreeOfAKind,
			[(2, _), (2, _), (1, _)] => HandType::TwoPair,
			[(2, _), (1, _), (1, _), (1, _)] => HandType::OnePair,
			[(1, _), (1, _), (1, _), (1, _), (1, _)] => HandType::HighCard,
			_ => panic!("unexpected hand partitions: {:?}", partitions),
		}
	}
	fn cmp(&self, other: &Self) -> Ordering {
		self._cmp::<false>(other)
	}
	fn cmp_joker(&self, other: &Self) -> Ordering {
		self._cmp::<true>(other)
	}
	fn _cmp<const WILDCARD_JOKER: bool>(&self, other: &Self) -> Ordering {
		match self.hand_type(WILDCARD_JOKER).cmp(&other.hand_type(WILDCARD_JOKER)) {
			Ordering::Equal => {
				// emit highest rank first, not lowest
				self.cards.iter().map(|c| c.rank(WILDCARD_JOKER)).cmp(
					other.cards.iter().map(|c| c.rank(WILDCARD_JOKER))
				).reverse()
			},
			o => o,
		}
	}
}

#[derive(Debug,Clone,Copy)]
pub struct Day07;

impl AoCDay for Day07 {
	type Data<'i> = Vec<Play>;
	type Answer = usize;

	fn day(&self) -> u8 { 7 }

	fn parse<'i>(&self, input: &'i str) -> Self::Data<'i> {
		aoch::parsing::from_lines(input).unwrap()
	}
	fn part1(&self, _data: &mut Self::Data<'_>) -> Self::Answer {
		_data.sort_by(|a, b| a.cmp(b).reverse());
		_data.iter()
			.enumerate()
			.map(|(rank, play)| (rank+1, play))
			.inspect(|(rank, play)| log::debug!("rank {}: {:?} w/ {:?}", rank, play, play.hand_type(false)))
			.map(|(rank, play)| rank * play.bid)
			.sum::<usize>()
	}
	fn part2(&self, _data: &mut Self::Data<'_>) -> Self::Answer {
		_data.sort_by(|a, b| a.cmp_joker(b).reverse());
		_data.iter()
			.enumerate()
			.map(|(rank, play)| (rank+1, play))
			.inspect(|(rank, play)| log::debug!("rank {}: {:?} w/ {:?}", rank, play, play.hand_type(true)))
			.map(|(rank, play)| rank * play.bid)
			.sum::<usize>()
	}
}

#[cfg(test)]
const TEST_INPUT: &'static str = "
32T3K 765
T55J5 684
KK677 28
KTJJT 220
QQQJA 483
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
		(TEST_INPUT, 6440),
		(daystr!("07"), 254024898),
	];
	test_runner::<Day07, _>(Day07, DayPart::Part1, &cases);
}
#[test]
fn part2() {
	let cases = [
		(TEST_INPUT, 5905),
		(daystr!("07"), 254115617),
	];
	test_runner::<Day07, _>(Day07, DayPart::Part2, &cases);
}
