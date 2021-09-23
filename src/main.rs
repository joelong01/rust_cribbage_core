#![allow(dead_code)]
#![allow(unused_imports)]

use arrayvec::ArrayVec;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
mod game;
use crate::game::cards::*;
use crate::game::scoring::score_hand;
use std::collections::HashMap;
use std::env;
use strum::IntoEnumIterator;

fn main() {}

fn serialize_hand(hand: &[Card]) -> String {
    let json_hand = serde_json::to_string(&hand).unwrap();
    println!("Serialized to JSON");
    println!("================");
    println!("{:?}", json_hand);
    json_hand
}

//
//  return the index of the cut card
fn cut() -> usize {
    thread_rng().gen_range(0..51)
}

fn shuffle(deck: &mut ArrayVec<Card, 52>) {
    let mut rng = thread_rng();
    deck.shuffle(&mut rng);
}

fn dump_hand(hand: &[Card]) {
    for card in hand {
        println!("{},", card);
    }
}

fn dump_deck(deck: &ArrayVec<Card, 52>) {
    let mut count = 0;
    for card in deck {
        print!("{}\t", card);
        count += 1;
        if count % 4 == 0 {
            println!(" ");
        }
    }
}
