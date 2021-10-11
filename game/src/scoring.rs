//! `score` provides a functional approach to tallying the score for a
//! cribbage hand.

use crate::{
    cards::{Card, Hand, Rank, Suit},
    combinator::all_combinations_of_min_size,
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Cribbage has five basic hand scoring combinations.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CombinationKind {
    Nob,
    Fifteen,
    RankMatch,
    Run,
    SuitMatch,
    ThirtyOne,
}

/// Some cribbage scoring combinations have specific names
/// depending on how many cards are involved.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CombinationName {
    Nob,
    Fifteen,
    Pair,
    RunOfThree,
    RunOfFour,
    FlushOfFour,
    RunOfFive,
    FlushOfFive,
    RoyalPair,
    DoubleRoyalPair,
    RunOfSix,
    RunOfSeven,
    ThirtyOne,
}

/// `Combination` is a record of a single scoring combination of cards.
#[derive(Clone, Debug, Serialize)]
pub struct Combination {
    kind: CombinationKind,
    pub name: CombinationName,
    pub cards: Vec<Card>,
    rank_info: Rank, // convenience field for pairs
    suit_info: Suit, // convenience field for flushes
    pub points: u32,
}

/// `Score` holds a collection of scoring combinations. `score.points()`
/// returns the sume of the points of the `combinations`.
#[derive(Clone, Debug, Serialize)]
pub struct Score {
    pub combinations: Vec<Combination>,
    pub total_score: u32, // convinient sum of all combinations.score
}

/// Calculates the score for a `hand` of four cribbage cards and the `starter`.
///
/// # Assumptions
///
/// * `hand` has four unique and valid cards
/// * `starter` is a valid `Card` that is unique when combined with `hand`
/// * This is also used when picking a hand, in which case 4 cards are passed in without a starter
//  *
pub fn score_hand(hand: Hand, starter: Option<Card>, is_crib: bool) -> Score {
    let mut vector = hand.clone();
    let mut s: Score = Score::new();
    if let Some(starter) = starter {
        vector.push(starter);
        s = nob_score(hand, starter);
    }
    vector.sort(); // ordered by rank rirst

    all_combinations_of_min_size(vector, 2)
        .filter_map(|cards| score(cards, is_crib))
        .fold(&mut s, |score, combis| score.tally(combis))
        .clone()
}

/// A Nob is scored if a Jack in the `hand` matches the suit of the `starter`.
///
/// # Returns
///
/// An empty `Score` or a `Score` that contains a single `Nob` `Combination`
pub fn nob_score(hand: Hand, starter: Card) -> Score {
    let mut score = Score::new();
    for c in hand {
        if c.rank == Rank::Jack && c.suit == starter.suit {
            score.add_combination(Combination::new(CombinationKind::Nob, vec![c, starter]));
            return score;
        };
    }
    score
}

/// `score` evaluates `cards` against each of the cribbage hand scoring
/// combinations and returns all scoring combinations that are
/// identified.
pub fn score(cards: Vec<Card>, is_crib: bool) -> Option<Vec<Combination>> {
    let mut combis = Vec::new();

    if let Some(c) = score_fifteen(cards.clone()) {
        combis.push(c)
    }
    if let Some(c) = score_pair(cards.clone()) {
        combis.push(c)
    }
    if let Some(c) = score_run(cards.clone()) {
        combis.push(c)
    }
    if let Some(c) = score_flush(cards, is_crib) {
        combis.push(c)
    }
    match combis.len() {
        0 => None,
        _ => Some(combis),
    }
}

/// If the values of the `cards` sum to 15, returns a `Fifteen` `Combination`
fn score_fifteen(cards: Vec<Card>) -> Option<Combination> {
    match cards.iter().fold(0, |s, c| s + c.value) {
        15 => Some(Combination::new(CombinationKind::Fifteen, cards)),
        _ => None,
    }
}

/// If all of the `cards` share the same `Rank`, returns a `Pair`
/// `Combination`
fn score_pair(cards: Vec<Card>) -> Option<Combination> {
    let len = cards.len();
    if len < 2 {
        None
    } else {
        let rank = cards[0].rank;
        match cards.iter().all(|c| c.rank == rank) {
            true => Some(Combination::new(CombinationKind::RankMatch, cards)),
            false => None,
        }
    }
}

/// If `cards` has at least three cards, and they form a contiguous run of
/// sequential `Rank`s, returns a `Run` `Combination`
pub fn score_run(cards: Vec<Card>) -> Option<Combination> {
    if cards.len() < 3 {
        None
    } else {
        let first_rank = cards[0].rank as usize;
        match cards
            .iter()
            .map(|c| c.rank as usize - first_rank)
            .eq(0..cards.len())
        {
            true => Some(Combination::new(CombinationKind::Run, cards)),
            false => None,
        }
    }
}

/// If `cards` has at least four cards and they all share a `Suit`, returns
/// a `Flush` `Combination`.  a crib has to have 5 cards of the same suit,
/// a regular hand can have only 4
fn score_flush(cards: Vec<Card>, is_crib: bool) -> Option<Combination> {
    let len = cards.len();

    match len {
        1..=3 => {
            return None;
        }
        4 => {
            if is_crib {
                return None;
            }
        }
        _ => {}
    }

    let suit = cards[0].suit;
    match cards.iter().all(|c| c.suit == suit) {
        true => Some(Combination::new(CombinationKind::SuitMatch, cards)),
        false => None,
    }
}
impl Default for Score {
    fn default() -> Self {
        Self::new()
    }
}
impl Score {
    /// Returns an empty score with no `Combination`s
    pub fn new() -> Score {
        Score {
            combinations: Vec::new(),
            total_score: 0,
        }
    }

    /// Returns the sum of the points associated with the `combinations`
    pub fn points(self) -> u32 {
        self.combinations.iter().fold(0, |p, c| p + c.points)
    }

    /// `tally` adds the `Combination`s in `combis` to `score` one at a time,
    /// so that combinations that are subsumed by other combinations are
    /// handled correctly.
    pub fn tally(&mut self, combis: Vec<Combination>) -> &mut Self {
        for c in combis {
            self.add_combination(c);
        }
        self.total_score = self.combinations.iter().map(|combi| combi.points).sum();

        self
    }

    /// `add_combination` adds `combi` to the score, unless it would be
    /// subsumed by a `Combination` already in `combinations`. If `combi`
    /// subsumes a `Combination` that is already in `combinations`, `combi`
    /// replaces it.
    fn add_combination(&mut self, combi: Combination) {
        match combi.kind {
            CombinationKind::Nob | CombinationKind::Fifteen => self.combinations.push(combi),
            _ => {
                let mut subsumed = false;
                self.combinations = self
                    .combinations
                    .iter()
                    .filter_map(|c| match c.kind == combi.kind {
                        false => Some(c.clone()),
                        true => match c.kind {
                            CombinationKind::RankMatch => {
                                if c.rank_info == combi.rank_info {
                                    if c.points >= combi.points {
                                        subsumed = true;
                                        Some(c.clone())
                                    } else {
                                        None
                                    }
                                } else {
                                    Some(c.clone())
                                }
                            }
                            CombinationKind::Run => match c.points.cmp(&combi.points) {
                                Ordering::Greater => {
                                    subsumed = true;
                                    Some(c.clone())
                                }
                                Ordering::Equal => Some(c.clone()),
                                Ordering::Less => None,
                            },
                            CombinationKind::SuitMatch => {
                                if c.points > combi.points {
                                    subsumed = true;
                                    Some(c.clone())
                                } else {
                                    None
                                }
                            }
                            _ => Some(c.clone()),
                        },
                    })
                    .collect::<Vec<Combination>>();
                if !subsumed {
                    self.combinations.push(combi);
                }
            }
        }
    }
}

impl Combination {
    pub fn new(kind: CombinationKind, cards: Vec<Card>) -> Combination {
        let name = Combination::name(kind, cards.len());
        let points = Combination::points(name);
        let rank = match kind {
            CombinationKind::RankMatch => cards[0].rank,
            _ => Rank::Unknown,
        };
        let suit = match kind {
            CombinationKind::SuitMatch => cards[0].suit,
            _ => Suit::Unknown,
        };
        Combination {
            kind,
            name,
            cards,
            rank_info: rank,
            suit_info: suit,
            points,
        }
    }

    /// This is the one place in which `CombinationKind`s and card counts are
    /// mapped to `CombinationName`s.
    fn name(kind: CombinationKind, count: usize) -> CombinationName {
        match kind {
            CombinationKind::Nob => CombinationName::Nob,
            CombinationKind::Fifteen => CombinationName::Fifteen,
            CombinationKind::RankMatch => match count {
                2 => CombinationName::Pair,
                3 => CombinationName::RoyalPair,
                4 => CombinationName::DoubleRoyalPair,
                _ => panic!("How did you get here?"),
            },
            CombinationKind::Run => match count {
                3 => CombinationName::RunOfThree,
                4 => CombinationName::RunOfFour,
                5 => CombinationName::RunOfFive,
                6 => CombinationName::RunOfSix,
                7 => CombinationName::RunOfSeven,
                _ => panic!("How did you get here?"),
            },
            CombinationKind::SuitMatch => match count {
                4 => CombinationName::FlushOfFour,
                5 => CombinationName::FlushOfFive,
                _ => panic!("How did you get here?"),
            },
            CombinationKind::ThirtyOne => CombinationName::ThirtyOne,
        }
    }

    /// This is the one place in which particular combinations are mapped to
    /// points.
    fn points(name: CombinationName) -> u32 {
        match name {
            CombinationName::Nob => 1,
            CombinationName::Fifteen => 2,
            CombinationName::Pair => 2,
            CombinationName::RunOfThree => 3,
            CombinationName::RunOfFour => 4,
            CombinationName::FlushOfFour => 4,
            CombinationName::RunOfFive => 5,
            CombinationName::FlushOfFive => 5,
            CombinationName::RoyalPair => 6,
            CombinationName::DoubleRoyalPair => 12,
            CombinationName::RunOfSix => 6,
            CombinationName::RunOfSeven => 7,
            CombinationName::ThirtyOne => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cards::{Card, Rank::*, Suit as Of};
    use crate::new_card as c;

    macro_rules! test_case {
        (
        $name:ident,
        $player_hand:expr,
        $computer_hand:expr,
        $crib_hand:expr,
        $shared_card:expr,
        $expected_player_score:literal,
        $expected_computer_score:literal,
        $expected_crib_score:literal
     ) => {
            #[test]
            fn $name() {
                let player_score =
                    super::score_hand((&$player_hand).to_vec(), Some($shared_card), false);
                let crib_score =
                    super::score_hand((&$crib_hand).to_vec(), Some($shared_card), true);
                let computer_score =
                    super::score_hand((&$computer_hand).to_vec(), Some($shared_card), false);

                assert_eq!(
                    $expected_player_score, player_score.total_score,
                    "Player Algo Score: {} vs. Hand Score: {}",
                    player_score.total_score, $expected_player_score
                );
                assert_eq!(
                    $expected_computer_score, computer_score.total_score,
                    "Computer Algo Score: {} vs. Hand Score: {}",
                    computer_score.total_score, $expected_computer_score
                );
                assert_eq!(
                    $expected_crib_score, crib_score.total_score,
                    "Crib Algo Score: {} vs. Hand Score: {}",
                    crib_score.total_score, $expected_crib_score
                );
            }
        };
    }

    test_case!(
        player_3_fives_and_a_jack_cut_the_5,
        [
            c!(Five, Of::Hearts),
            c!(Five, Of::Clubs),
            c!(Five, Of::Spades),
            c!(Jack, Of::Diamonds)
        ],
        [
            c!(Ace, Of::Spades),
            c!(Four, Of::Diamonds),
            c!(Six, Of::Spades),
            c!(Jack, Of::Hearts)
        ],
        [
            c!(Ace, Of::Clubs),
            c!(Two, Of::Clubs),
            c!(Three, Of::Clubs),
            c!(Four, Of::Clubs)
        ],
        c!(Five, Of::Diamonds),
        29,
        9,
        7
    );

    test_case!(
        two_pair_many_15s_double_run_of_4,
        [
            c!(Five, Of::Hearts),
            c!(Five, Of::Clubs),
            c!(Six, Of::Hearts),
            c!(Six, Of::Clubs)
        ],
        [
            c!(Ace, Of::Spades),
            c!(Four, Of::Diamonds),
            c!(Six, Of::Spades),
            c!(Jack, Of::Diamonds)
        ],
        [
            c!(Ace, Of::Clubs),
            c!(Two, Of::Clubs),
            c!(Three, Of::Clubs),
            c!(Four, Of::Clubs)
        ],
        c!(Four, Of::Hearts),
        24,
        8,
        10
    );

    test_case!(
        flush_nibs_15_double_run_no_flush_in_crib,
        [
            c!(Five, Of::Hearts),
            c!(Six, Of::Clubs),
            c!(Seven, Of::Hearts),
            c!(Jack, Of::Hearts)
        ],
        [
            c!(Ace, Of::Spades),
            c!(Four, Of::Diamonds),
            c!(Six, Of::Spades),
            c!(Jack, Of::Diamonds)
        ],
        [
            c!(Ace, Of::Clubs),
            c!(Two, Of::Clubs),
            c!(Three, Of::Clubs),
            c!(Four, Of::Clubs)
        ],
        c!(King, Of::Hearts),
        12,
        4,
        8
    );

    test_case!(
        two_pair,
        [
            c!(Five, Of::Hearts),
            c!(Five, Of::Clubs),
            c!(Six, Of::Hearts),
            c!(Six, Of::Clubs)
        ],
        [
            c!(Ace, Of::Spades),
            c!(Four, Of::Diamonds),
            c!(Six, Of::Spades),
            c!(Jack, Of::Diamonds)
        ],
        [
            c!(Ace, Of::Clubs),
            c!(Two, Of::Clubs),
            c!(Three, Of::Clubs),
            c!(Four, Of::Clubs)
        ],
        c!(King, Of::Diamonds),
        8,
        5,
        8
    );

    test_case!(
        player_3_fives_and_his_nibs,
        [
            c!(Five, Of::Hearts),
            c!(Five, Of::Clubs),
            c!(Four, Of::Spades),
            c!(Jack, Of::Diamonds)
        ],
        [
            c!(Ace, Of::Spades),
            c!(Four, Of::Diamonds),
            c!(Six, Of::Spades),
            c!(Jack, Of::Hearts)
        ],
        [
            c!(Ace, Of::Clubs),
            c!(Two, Of::Clubs),
            c!(Three, Of::Clubs),
            c!(Four, Of::Clubs)
        ],
        c!(Five, Of::Diamonds),
        15,
        9,
        7
    );

    test_case!(
        four_twos_and_a_nine,
        [
            c!(Two, Of::Hearts),
            c!(Two, Of::Clubs),
            c!(Two, Of::Spades),
            c!(Two, Of::Diamonds)
        ],
        [
            c!(Ace, Of::Spades),
            c!(Four, Of::Diamonds),
            c!(Six, Of::Spades),
            c!(Jack, Of::Hearts)
        ],
        [
            c!(Ace, Of::Clubs),
            c!(Seven, Of::Clubs),
            c!(Three, Of::Clubs),
            c!(Four, Of::Clubs)
        ],
        c!(Nine, Of::Diamonds),
        20,
        4,
        2
    );

    test_case!(
        four_threes_and_a_nine,
        [
            c!(Three, Of::Hearts),
            c!(Three, Of::Clubs),
            c!(Three, Of::Spades),
            c!(Three, Of::Diamonds)
        ],
        [
            c!(Ace, Of::Spades),
            c!(Four, Of::Diamonds),
            c!(Six, Of::Spades),
            c!(Jack, Of::Hearts)
        ],
        [
            c!(Ace, Of::Clubs),
            c!(Seven, Of::Clubs),
            c!(Three, Of::Clubs),
            c!(Four, Of::Clubs)
        ],
        c!(Nine, Of::Diamonds),
        24,
        4,
        2
    );

    test_case!(
        fours_fives_and_sixes,
        [
            c!(Four, Of::Hearts),
            c!(Five, Of::Clubs),
            c!(Five, Of::Spades),
            c!(Six, Of::Diamonds)
        ],
        [
            c!(Two, Of::Spades),
            c!(Four, Of::Diamonds),
            c!(Jack, Of::Spades),
            c!(Eight, Of::Hearts)
        ],
        [
            c!(Ace, Of::Clubs),
            c!(Seven, Of::Clubs),
            c!(Three, Of::Clubs),
            c!(Four, Of::Clubs)
        ],
        c!(Six, Of::Hearts),
        24,
        0,
        2
    );

    test_case!(
        highest_five_point_flush,
        [
            c!(Five, Of::Hearts),
            c!(Ten, Of::Hearts),
            c!(Jack, Of::Hearts),
            c!(Queen, Of::Hearts)
        ],
        [
            c!(Two, Of::Spades),
            c!(Four, Of::Diamonds),
            c!(Jack, Of::Spades),
            c!(Eight, Of::Hearts)
        ],
        [
            c!(Ace, Of::Clubs),
            c!(Seven, Of::Clubs),
            c!(Three, Of::Clubs),
            c!(Four, Of::Clubs)
        ],
        c!(King, Of::Hearts),
        18,
        0,
        4
    );
}
