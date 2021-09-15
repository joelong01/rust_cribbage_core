// This is a generated file, do not modify directly

use rstest::rstest;
use rust_cribbage_core::game::cards;
use rust_cribbage_core::game::scoring;

mod generated_common;

/// Run multiple scoring scenarios using rstest
#[rstest]
#[case::player_3_fives_and_a_jack_cut_the_5(
    generated_common::parse_toml_hand("FiveOfHearts,FiveOfClubs,FiveOfSpades,JackOfDiamonds"),
    generated_common::parse_toml_hand("AceOfSpades,FourOfDiamonds,SixOfSpades,JackOfHearts"),
    generated_common::parse_toml_hand("AceOfClubs,TwoOfClubs,ThreeOfClubs,FourOfClubs"),
    cards::Card::from_string("FiveOfDiamonds"),
    29,
    9,
    7)]
#[case::two_pair_many_15s_double_run_of_4(
    generated_common::parse_toml_hand("FiveOfHearts,FiveOfClubs,SixOfHearts,SixOfClubs"),
    generated_common::parse_toml_hand("AceOfSpades,FourOfDiamonds,SixOfSpades,JackOfDiamonds"),
    generated_common::parse_toml_hand("AceOfClubs,TwoOfClubs,ThreeOfClubs,FourOfClubs"),
    cards::Card::from_string("FourOfHearts"),
    24,
    6,
    10)]
#[case::flush_nibs_15_double_run_no_flush_in_crib(
    generated_common::parse_toml_hand("FiveOfHearts,SixOfClubs,SevenOfHearts,JackOfHearts"),
    generated_common::parse_toml_hand("AceOfSpades,FourOfDiamonds,SixOfSpades,JackOfDiamonds"),
    generated_common::parse_toml_hand("AceOfClubs,TwoOfClubs,ThreeOfClubs,FourOfClubs"),
    cards::Card::from_string("KingOfHearts"),
    12,
    4,
    8)]
#[case::two_pair(
    generated_common::parse_toml_hand("FiveOfHearts,FiveOfClubs,SixOfHearts,SixOfClubs"),
    generated_common::parse_toml_hand("AceOfSpades,FourOfDiamonds,SixOfSpades,JackOfDiamonds"),
    generated_common::parse_toml_hand("AceOfClubs,TwoOfClubs,ThreeOfClubs,FourOfClubs"),
    cards::Card::from_string("KingOfDiamonds"),
    8,
    5,
    8)]
#[case::player_3_fives_and_his_nibs(
    generated_common::parse_toml_hand("FiveOfHearts,FiveOfClubs,FourOfSpades,JackOfDiamonds"),
    generated_common::parse_toml_hand("AceOfSpades,FourOfDiamonds,SixOfSpades,JackOfHearts"),
    generated_common::parse_toml_hand("AceOfClubs,TwoOfClubs,ThreeOfClubs,FourOfClubs"),
    cards::Card::from_string("FiveOfDiamonds"),
    15,
    9,
    7)]
fn generated_test(
    #[case] player_hand: Vec<cards::Card>,
    #[case] computer_hand: Vec<cards::Card>,
    #[case] crib_hand: Vec<cards::Card>,
    #[case] shared_card: cards::Card,
    #[case] expected_player_score: i32,
    #[case] expected_computer_score: i32,
    #[case] expected_crib_score: i32)  {
    generated_common::execute_toml_test(
        player_hand,
        computer_hand,
        crib_hand,
        shared_card,
        expected_player_score,
        expected_computer_score,
        expected_crib_score);
}