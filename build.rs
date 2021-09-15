use serde::{Deserialize, Serialize};
use std::{fs, io::Write};
use toml::de;

const GENERATED_FILE_PATH: &str = "./tests/generated_toml_tests.rs";
const TEST_FILE_PATH: &str = "./tests/test.toml";

/// Reads and deserializes test data from tests/test.toml
/// and generates rust integration tests included in
/// tests/generated_tests.rs during the build process
fn main() {
    let out_dir = "./tests/";
    let destination = std::path::Path::new(GENERATED_FILE_PATH);
    let mut f = std::fs::File::create(&destination).unwrap();

    let test_list = read_tests();

    write!(
        f,
        "// This is a generated file, do not modify directly

use rstest::rstest;
use rust_cribbage_core::game::cards;
use rust_cribbage_core::game::scoring;

mod generated_common;

/// Run multiple scoring scenarios using rstest
#[rstest]").unwrap_or_else(|error| {
        panic!("Error writing to file: {}, error: {}", GENERATED_FILE_PATH , error);
    });


    for test in test_list.tests {
        let test_name = test.name
            .trim()
            .replace(" ", "_")
            .replace(&[',', '.'][..], "");

        write!(
            f,
            "
#[case::{test_name}(
    generated_common::parse_toml_hand(\"{player_hand}\"),
    generated_common::parse_toml_hand(\"{computer_hand}\"),
    generated_common::parse_toml_hand(\"{crib_hand}\"),
    cards::Card::from_string(\"{shared_card}\"),
    {expected_player_score},
    {expected_computer_score},
    {expected_crib_score})]",
            test_name = test_name,
            player_hand = test.player_hand,
            computer_hand = test.computer_hand,
            crib_hand = test.crib_hand,
            shared_card = test.shared_card,
            expected_player_score = test.player_score,
            expected_computer_score = test.computer_score,
            expected_crib_score = test.crib_score
        ).unwrap_or_else(|error| {
            panic!("Error writing to file: {}, error: {}", GENERATED_FILE_PATH , error);
        });
    }

    write!(
        f,
        "
fn generated_test(
    #[case] player_hand: Vec<cards::Card>,
    #[case] computer_hand: Vec<cards::Card>,
    #[case] crib_hand: Vec<cards::Card>,
    #[case] shared_card: cards::Card,
    #[case] expected_player_score: i32,
    #[case] expected_computer_score: i32,
    #[case] expected_crib_score: i32)  {{
    generated_common::execute_toml_test(
        player_hand,
        computer_hand,
        crib_hand,
        shared_card,
        expected_player_score,
        expected_computer_score,
        expected_crib_score);
}}").unwrap_or_else(|error| {
        panic!("Error writing to file: {}, error: {}", GENERATED_FILE_PATH , error);
    });
}

/// Reads test data from tests/test.toml and deserializes into a TestList
fn read_tests() -> TestList {
    let test_file_path = std::path::Path::new(TEST_FILE_PATH);
    let toml_string = fs::read_to_string(test_file_path).unwrap_or_else(|error| {
        panic!("Error Reading file: {}, error {}", TEST_FILE_PATH, error);
    });

    println!("Test File {}", toml_string);

    let test_list: TestList = de::from_str(&toml_string).unwrap();

    println!("You have {} tests", test_list.tests.len());

    test_list
}

#[derive(Serialize, Deserialize, Debug)]
/// Holds the vector to deserialize the tests into
struct TestList {
    tests: Vec<CribbageTest>,
}

#[derive(Serialize, Deserialize, Debug)]
/// The configuration values for a deserialized cribbage test/scenario
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