#![allow(dead_code)]
use std::fmt;

use strum::AsStaticRef;
use strum::IntoEnumIterator;
use strum_macros::AsStaticStr;
use strum_macros::EnumIter;
use strum_macros::EnumString;
//
//  this module has the core abstraction of a card - ordinal, rank, suit, etc.
//

#[derive(EnumIter, Copy, Clone, AsStaticStr, PartialEq, Debug)]
pub(crate) enum Suit {
    Clubs = 1,
    Diamonds = 2,
    Hearts = 3,
    Spades = 4,
}

impl Suit {
    pub fn dump_suits(&self) {
        let iter = Suit::iter();
        iter.for_each(|suit| println!("{:?}", suit));
    }
}

#[derive(Copy, Clone, EnumString, EnumIter, AsStaticStr, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Ordinal {
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
}

fn to_int(ordinal: &Ordinal) -> u8 {
    *ordinal as u8
}

#[derive(Copy, Clone, EnumString, Debug)]
pub(crate) enum Owner {
    Computer = 1,
    Player = 2,
    Shared = 3,
    Unknown = 4,
}

pub(crate) struct Card {
    ordinal: Ordinal,
    rank: i32,  // 1 - 13.  used for runs.
    value: i32, // 1 - 10.  used for counting
    suit: Suit,
    owner: Owner,
    name: String,
}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.ordinal == other.ordinal && self.suit == other.suit
    }
}

impl Eq for Card {}

//
//  when debugging we should see only the name of the card, which is derived below
//  it should be of the form like "AceOfSpades"
impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Card").field("Name", &self.name).finish()
    }
}

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.write_str(&self.name())
    }
}

impl Card {
    pub(crate) fn new(ordinal: Ordinal, suit: Suit) -> Self {
        let name = format!("{}Of{}", ordinal.as_static(), suit.as_static());
        let rank = ordinal as i32;
        let mut value = rank;
        if value > 10 {
            value = 10;
        }
        let owner = Owner::Unknown;
        Self {
            ordinal,
            rank,
            value,
            suit,
            owner,
            name,
        }
    }

    pub fn set_owner( &mut self,  owner: Owner) {
        self.owner = owner;
    }

    pub fn name(&self) -> String {
        let name = format!("{}Of{}", self.ordinal.as_static(), self.suit.as_static());
        name
    }

    pub fn rank(&self) -> i32 {
        self.rank
    }

    pub fn ordinal(&self) -> Ordinal {
        self.ordinal
    }
    pub fn suit(&self) -> Suit {
        self.suit
    }
}
