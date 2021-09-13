#![allow(dead_code)]
#![allow(unused_imports)]

use arrayvec::ArrayVec;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use toml::value::Array;
mod game;
use crate::game::cards::*;
use crate::game::scoring::score_hand;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fmt::Error;
use std::fs::File;
use std::io::Read;
use std::path::*;
use strum::IntoEnumIterator;
use toml::*;
use colored::*;

#[derive(Serialize, Deserialize, Debug)]
struct CribbageTest {
    player_hand: String,
    computer_hand: String,
    shared_card: String,
    crib_hand: String,
    player_score: i32,
    computer_score: i32,
    crib_score: i32,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TestList {
    tests: Vec<CribbageTest>,
}

fn main() {
    let test_file_path = std::path::Path::new("tests/test.toml");
    let mut test_data = match File::open(test_file_path) {
        Ok(f) => f,
        Err(e) => panic!("test file not found: {}", e),
    };

    let mut toml_string = String::new();
    match test_data.read_to_string(&mut &mut toml_string) {
        Ok(s) => s,
        Err(e) => panic!("Error Reading file: {}", e),
    };
    println!("Test File {}", toml_string);

    let test_list: TestList = from_str(&toml_string).unwrap();

    println!("you have {} tests", test_list.tests.len());

    for (_, test) in test_list.tests.iter().enumerate() {
        run_test(test);
    }
}

fn run_test(test: &CribbageTest) -> bool {
    let player_hand = parse_toml_hand(&test.player_hand);
    let computer_hand = parse_toml_hand(&test.computer_hand);
    let crib_hand = parse_toml_hand(&test.crib_hand);
    println!("\nTest Name: {}", test.name);
    let shared_card = Card::from_string(&test.shared_card);
    let player_score = score_hand(&player_hand,Some(&shared_card), false);
    let crib_score = score_hand(&crib_hand, Some(&shared_card), true);
    let computer_score = score_hand(&computer_hand, Some(&shared_card), false);

    if player_score - test.player_score == 0 {
        print!("{}", "PASSED:\t".green());
    } else {
        print!("{}","FAILED:\t".red());
    }

    println!(
        "Player Algo Score: {} vs. Hand Score: {}",
        player_score, test.player_score
    );

    if computer_score - test.computer_score == 0 {
        print!("{}", "PASSED:\t".green());
    } else {
        print!("{}","FAILED:\t".red());
    }
    println!(
        "Computer Algo Score: {} vs. Hand Score: {}",
        computer_score, test.computer_score
    );

    if crib_score - test.crib_score == 0 {
        print!("{}", "PASSED:\t".green());
    } else {
        print!("{}","FAILED:\t".red());
    }

    println!(
        "Crib Algo Score: {} vs. Hand Score: {}",
        crib_score, test.crib_score
    );

    false
}

fn parse_toml_hand(hand_as_string: &str) -> Vec<Card> {
    let mut hand: Vec<Card> = Vec::new();

    for card_name in hand_as_string.split(',') {
        hand.push(Card::from_string(card_name));
    }

    hand
}

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
fn old_main() {
    let mut deck = ArrayVec::<Card, 52>::new();
    for o in Ordinal::iter() {
        for s in Suit::iter() {
            let card = Card::new(o, s);
            deck.push(card);
        }
    }

    shuffle(&mut deck);
    dump_deck(&deck);

    let player_cut = cut();
    let mut computer_cut = cut();
    //
    //  stranger things have happened than getting the same card twice
    while player_cut == computer_cut {
        computer_cut = cut();
    }

    println!(
        "\nComputer Cut: {}\nPlayer Cut: {}\n",
        deck[computer_cut], deck[player_cut]
    );

    println!(" Player Hand\t ComputerHand");
    println!("==============\t==============");
    let mut i = 0;
    let mut player_hand: Vec<Card> = Vec::new();
    let mut computer_hand: Vec<Card> = Vec::new();

    while i < 12 {
        println!("{}\t{}", deck[i], deck[i + 1]);
        player_hand.push(deck[i].clone());
        //  deck[i].set_owner(Owner::Player);
        computer_hand.push(deck[i + 1].clone());
        // deck[i+1].set_owner(Owner::Player);
        i += 2;
    }

    let shared_card = deck[12].clone();

    println!("\n\nSorted Player Hand");
    println!("------------------");

    player_hand.sort_by_key(|a| a.ordinal());
    dump_hand(&player_hand);

    println!("\n\nSorted Computer Hand");
    println!("--------------------");

    computer_hand.sort_by_key(|a| a.ordinal());
    dump_hand(&computer_hand);

    println!("\n\nShared Card");
    println!("------------");
    println!("{}\n", shared_card);

    let json_hand = serialize_hand(&player_hand);

    let new_hand: Vec<Card> = serde_json::from_str(&json_hand).unwrap();
    println!("Deserialized hand");
    println!("===============");
    for card in new_hand {
        print!("{}\t", card);
    }
    println!("\n");

    let player_score = score_hand(&player_hand[0..4], Some(&shared_card), false);
    println!("PlayerScore is: {}", player_score);
}
