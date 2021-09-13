use rust_cribbage_core::game::cards;
use rust_cribbage_core::game::scoring;

/// Parses toml to produce a vector of cards
fn parse_toml_hand(hand_as_string: &str) -> Vec<cards::Card> {
    let mut hand: Vec<cards::Card> = Vec::new();

    for card_name in hand_as_string.split(',') {
        hand.push(cards::Card::from_string(card_name));
    }

    hand
}

// Includes the tests generated from the test.toml file
include!(concat!(env!("OUT_DIR"), "/test.rs"));
