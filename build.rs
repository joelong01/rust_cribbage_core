use serde::{Deserialize, Serialize};
use std::{fs, io::Write};
use toml::de;

/// Reads and deserializes test data from tests/test.toml
/// and generates rust integration test included in
/// tests/generated_tests.rs during the build process
fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let destination = std::path::Path::new(&out_dir).join("test.rs");
    let mut f = std::fs::File::create(&destination).unwrap();

    let test_list = read_tests();

    for test in test_list.tests {
        let test_name = test.name
            .trim()
            .replace(" ", "_")
            .replace(&[',', '.'][..], "");

        write!(
            f,
            "
#[test]
fn {test_method_name}() {{
    let player_hand = parse_toml_hand(\"{player_hand}\");
    let computer_hand = parse_toml_hand(\"{computer_hand}\");
    let crib_hand = parse_toml_hand(\"{crib_hand}\");
    println!(\"Test Name: {{}}\", \"{test_name}\");
    let shared_card = cards::Card::from_string(\"{shared_card}\");
    let player_score = scoring::score_hand(&player_hand, &shared_card, false);
    let expected_player_score = {expected_player_score};
    let crib_score = scoring::score_hand(&crib_hand, &shared_card, true);
    let expected_crib_score = {expected_crib_score};
    let computer_score = scoring::score_hand(&computer_hand, &shared_card, false);
    let expected_computer_score = {expected_computer_score};

    assert_eq!(expected_player_score, player_score);
    println!(
        \"Player Algo Score: {{}} vs. Hand Score: {{}}\",
        player_score, expected_player_score
    );

    assert_eq!(expected_computer_score, computer_score);
    println!(
        \"Computer Algo Score: {{}} vs. Hand Score: {{}}\",
        computer_score, expected_computer_score
    );

    assert_eq!(expected_crib_score, crib_score);
    println!(
        \"Crib Algo Score: {{}} vs. Hand Score: {{}}\",
        crib_score, expected_crib_score
    );
}}",
            test_method_name = test_name,
            player_hand = test.player_hand,
            computer_hand = test.computer_hand,
            crib_hand = test.crib_hand,
            test_name = test.name,
            shared_card = test.shared_card,
            expected_player_score = test.player_score,
            expected_crib_score = test.crib_score,
            expected_computer_score = test.computer_score
        ).unwrap();
    }
}

/// Reads test data from tests/test.toml and deserializes into a TestList
fn read_tests() -> TestList {
    let test_file_path = std::path::Path::new("tests/test.toml");
    let toml_string = fs::read_to_string(test_file_path).unwrap_or_else(|error| {
        panic!("Error Reading file: {}", error);
    });

    println!("Test File {}", toml_string);

    let test_list: TestList = de::from_str(&toml_string).unwrap();

    println!("you have {} tests", test_list.tests.len());

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