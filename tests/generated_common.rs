use rust_cribbage_core::game::cards;
use rust_cribbage_core::game::scoring;

/// Performs the underlying scoring validation for each toml test
pub fn test_scoring_combination(
    player_hand: Vec<cards::Card>,
    computer_hand: Vec<cards::Card>,
    crib_hand: Vec<cards::Card>,
    shared_card: cards::Card,
    expected_player_score: i32,
    expected_computer_score: i32,
    expected_crib_score: i32
) {
    let player_score = scoring::score_hand(&player_hand, &shared_card, false);
    let crib_score = scoring::score_hand(&crib_hand, &shared_card, true);
    let computer_score = scoring::score_hand(&computer_hand, &shared_card, false);

    assert_eq!(expected_player_score, player_score, "Player Algo Score: {} vs. Hand Score: {}", player_score, expected_player_score);
    assert_eq!(expected_computer_score, computer_score, "Computer Algo Score: {} vs. Hand Score: {}", computer_score, expected_computer_score);
    assert_eq!(expected_crib_score, crib_score, "Crib Algo Score: {} vs. Hand Score: {}", crib_score, expected_crib_score);
}

/// Parses toml to produce a vector of cards
pub fn parse_toml_hand(hand_as_string: &str) -> Vec<cards::Card> {
    let mut hand: Vec<cards::Card> = Vec::new();

    for card_name in hand_as_string.split(',') {
        hand.push(cards::Card::from_string(card_name));
    }

    hand
}


