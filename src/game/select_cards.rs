use crate::game::cards::*;
use crate::game::scoring::*;
use itertools::Itertools;

/**
 * go through each of the 16 combinations looking for the hand
 * that will perform best based on the value of the hand plus
 * or minus the value of the crib
 */
pub fn select_crib_cards(hand: &[Card], is_my_crib: bool) -> Vec<Card>{

    // get all possible hands
    let local_hand = hand.to_vec();
    let potential_hands = local_hand.into_iter().combinations(4);
    let mut max_crib = Vec::<Card>::new();
    let mut max_score = -1000;
    
    for h in potential_hands {

        // get the score for the current hand we are evaluating
        let mut score = score_hand(hand, None, false);
        let crib = get_crib_cards(hand, &h);

        if is_my_crib {
            // TODO: implement CardScoring.getCardValueToYourCrib
            score = 1;
        }
        else {
            // TODO: implement CardScoring.getCardValueToYourCrib
            score = -2000;
        }

        if score > max_score {
            max_score = score;
            max_crib = crib.clone();
        }
    }

    return max_crib;
}

/**
 * hand has 6 cards and is passed in by the client
 * heldCards has 4 cards and is generated via permutation
 * this returns the 2 cards that are in the hand but not the crib
*/
fn get_crib_cards(hand: &[Card], held_cards: &[Card]) -> Vec<Card> {
    let local_hand: Vec<Card> = hand.to_vec();
    let mut send_to_crib = Vec::<Card>::new();

    for h in local_hand.iter() {
        if !held_cards.contains(&h) {
            send_to_crib.push(h.clone());
        }
    }

    return send_to_crib;
}

mod tests {
    // import names from outer scope
    use super::*;

    #[test]
    fn test_select_crib_cards_hand_returns_zero() {

        // prepare test parameters
        let test_hand = "FiveOfHearts,FiveOfClubs,SixOfHearts,SixOfClubs";
        let mut hand: Vec<Card> = Vec::new();

        for card_name in test_hand.split(',') {
            hand.push(Card::from_string(card_name));
        }

        // execute the method under test
        let crib = select_crib_cards(&hand, true);

        // returned crib len should equal 0 given the inputs 
        assert_eq!(crib.len(), 0);
    }

    #[test]
    fn test_get_crib_cards_match_expected_length() {

        // prepare test parameters
        let test_hand = "FiveOfHearts,FiveOfClubs,SixOfHearts,SixOfClubs";
        let held_cards = "AceOfSpades,FourOfDiamonds,SixOfClubs,JackOfDiamonds";
        let mut hand: Vec<Card> = Vec::new();
        let mut held: Vec<Card> = Vec::new();

        for card_name in test_hand.split(',') {
            hand.push(Card::from_string(card_name));
        }

        for held_name in held_cards.split(',') {
            held.push(Card::from_string(held_name));
        }

        // execute the method under test
        let crib = get_crib_cards(&hand, &held);

        // returned crib len should equal 3 given the inputs 
        assert_eq!(crib.len(), 3);
    }

    #[test]
    fn test_get_crib_cards_all_match_length_zero() {

        // prepare test parameters
        let test_hand = "FiveOfHearts,FiveOfClubs,SixOfHearts,SixOfClubs";
        let held_cards = "FiveOfHearts,FiveOfClubs,SixOfHearts,SixOfClubs";
        let mut hand: Vec<Card> = Vec::new();
        let mut held: Vec<Card> = Vec::new();

        for card_name in test_hand.split(',') {
            hand.push(Card::from_string(card_name));
        }

        for held_name in held_cards.split(',') {
            held.push(Card::from_string(held_name));
        }

        // execute the method under test
        let crib = get_crib_cards(&hand, &held);

        // returned crib len should equal 0 given the inputs 
        assert_eq!(crib.len(), 0);
    }

    #[test]
    fn test_get_crib_cards_empty_input_length_zero() {

        // prepare test parameters
        let hand: Vec<Card> = Vec::new();
        let held: Vec<Card> = Vec::new();

        // execute the method under test
        let crib = get_crib_cards(&hand, &held);

        // returned crib len should equal 0 given the inputs 
        assert_eq!(crib.len(), 0);
    }
}