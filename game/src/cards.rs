//! `card` presents an interface for working with standard playing cards.
//! Only the cards required to represent the game cribbage are supported.

#![allow(dead_code)]
// this warning is on by default, but I like the explicit nature of setting the value
#![allow(clippy::redundant_static_lifetimes)]
#![allow(clippy::redundant_field_names)]

use crate::cribbage_errors::{CribbageError, CribbageErrorKind};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::error::Error;
use strum::AsStaticRef;
use strum::IntoEnumIterator;
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
            suit,
            rank,
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

    pub fn default() -> Card {
        Card {
            rank: Rank::Unknown,
            value: 0,
            suit: Suit::Unknown,
        }
    }

    //
    //  given an index into a deck, return the card.
    //  assume a form of the deck where index / 13 where the whole number is the suit and the remainder is the rank
    pub fn from_index(index: usize) -> Card {
        let div = index / 13usize;
        let suit = match Suit::iter().nth(div) {
            Some(suit) => suit,
            None => {
                panic!("Bad index passed into from_index");
            }
        };
        let remainder = index % 13;
        let rank = match Rank::iter().nth(remainder) {
            Some(rank) => rank,
            None => {
                panic!("Bad index passed into from_index");
            }
        };

        let value: i32 = if remainder >= 9 { 10 } else { remainder + 1 } as i32; // the remainder is 0 based and the value is 1 based

        Card { rank, value, suit }
    }

    pub fn from_string(card_as_string: &str) -> Result<Self, CribbageError> {
        let tokens = card_as_string.split("Of").collect::<Vec<_>>();
        if tokens.len() != 2 {
            let msg = format!(
                "{:?} is an invalid Card because it couldn't be split by 'Of'",
                card_as_string
            );
            return Err(CribbageError::new(CribbageErrorKind::ParseError, msg));
        };

        let rank: Rank = match tokens[0].parse() {
            Ok(rank) => rank,
            Err(_) => {
                return Err(CribbageError::new(
                    CribbageErrorKind::ParseError,
                    format!(
                        "Error Parsing ordinal in: {:?}. {:?} is invalid",
                        card_as_string, tokens[0]
                    )
                    .into(),
                ));
            }
        };

        let suit: Suit = match tokens[1].parse() {
            Ok(suit) => suit,
            Err(_) => {
                return Err(CribbageError::new(
                    CribbageErrorKind::ParseError,
                    format!(
                        "Error Parsing suit in: {:?}. {:?} is invalid",
                        card_as_string, tokens[1]
                    )
                    .into(),
                ));
            }
        };

        Ok(Card::new(rank, suit))
    }

    pub fn random_card() -> String {
        let mut rng = rand::thread_rng();
        let suit: Suit = SUITS[rng.gen_range(0..3)];
        let rank: Rank = RANKS[rng.gen_range(0..12)];
        format!("{:?}Of{:?}", rank, suit)
    }
}
#[allow(unused_macros)]
macro_rules! card {
    ($rank:expr, $suit:expr) => {{
        Card::new($rank, $suit)
    }};
}
