//! `card` presents an interface for working with standard playing cards.
//! Only the cards required to represent the game cribbage are supported.

#![allow(dead_code)]
// this warning is on by default, but I like the explicit nature of setting the value
#![allow(clippy::redundant_static_lifetimes)]
#![allow(clippy::redundant_field_names)]

use serde::{Deserialize, Serialize};
use std::fmt;
use strum::AsStaticRef;
use strum_macros::AsStaticStr;
use strum_macros::EnumIter;
use strum_macros::EnumString;

/// `Suit` represents the standard playing card suits and an `Unknown` value
/// that is useful for some algorithms over cards.
#[derive(
    Copy,
    Clone,
    EnumString,
    EnumIter,
    AsStaticStr,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
)]
pub enum Suit {
    Clubs = 1,
    Diamonds = 2,
    Hearts = 3,
    Spades = 4,
    Unknown,
}

#[derive(
    Copy,
    Clone,
    EnumString,
    EnumIter,
    AsStaticStr,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
)]
/// `Rank` represents the standard playing card ranks and an `Unknown` value
/// that is useful for some algorithms over cards.
pub enum Rank {
    Ace = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
    Unknown,
}

/// Iterable container of `Suit`s that could be replaced with the Step
/// trait when it is finalized
const SUITS: &'static [Suit] = &[Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];

/// Iterable container of `Rank`s that could be replaced with the Step
/// trait when it is finalized
const RANKS: &'static [Rank] = &[
    Rank::Ace,
    Rank::Two,
    Rank::Three,
    Rank::Four,
    Rank::Five,
    Rank::Six,
    Rank::Seven,
    Rank::Eight,
    Rank::Nine,
    Rank::Ten,
    Rank::Jack,
    Rank::Queen,
    Rank::King,
];

//
#[derive(Serialize, Deserialize, Copy, Clone, Debug, Ord, PartialOrd)]
pub struct Card {
    pub rank: Rank, // 1..13 used for runs
    pub value: i32, // 1 - 10.  used for counting
    pub suit: Suit,
}

/// `Deck` is a convenience type for more fluent code.
pub type Deck = Vec<Card>;

/// `Hand` is a convenience type for more fluent code.
pub type Hand = Vec<Card>;

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.rank == other.rank && self.suit == other.suit
    }
}

impl Eq for Card {}

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.write_str(&self.name())
    }
}

impl Card {
    pub fn name(&self) -> String {
        format!("{}Of{}", self.rank.as_static(), self.suit.as_static())
    }

    pub fn new(rank: Rank, suit: Suit) -> Card {
        Card {
            suit: suit,
            rank: rank,

            value: match rank {
                Rank::Unknown => 0,
                Rank::Ace => 1,
                Rank::Two => 2,
                Rank::Three => 3,
                Rank::Four => 4,
                Rank::Five => 5,
                Rank::Six => 6,
                Rank::Seven => 7,
                Rank::Eight => 8,
                Rank::Nine => 9,
                Rank::Ten | Rank::Jack | Rank::Queen | Rank::King => 10,
            },
        }
    }

    pub fn from_string(card_as_string: &str) -> Self {
        let tokens = card_as_string.split("Of").collect::<Vec<_>>();
        if tokens.len() != 2 {
            panic!(
                "\t\t{:?} is an invalid Card because it couldn't be split by 'Of'",
                card_as_string
            );
        }

        let rank: Rank = tokens[0].parse().unwrap_or_else(|_| {
            panic!(
                "\t\t\tError Parsing ordinal in: {:?}. {:?} is invalid",
                card_as_string, tokens[0]
            );
        });

        let suit: Suit = tokens[1].parse().unwrap_or_else(|_| {
            panic!(
                "\t\t\tError Parsing suit in: {:?}. {:?} is invalid",
                card_as_string, tokens[1]
            );
        });

        Card::new(rank, suit)
    }
}
#[allow(unused_macros)]
macro_rules! card {
    ($ordinal:expr, $suit:expr) => {{
        Card::new($ordinal, $suit)
    }};
}
