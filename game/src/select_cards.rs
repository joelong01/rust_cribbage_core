use crate::{   
    cards::{Card, Rank, Suit},
    combinator::all_combinations_of_size,
    counting::score_counting_cards_played,
    cribbage_errors::{CribbageError, CribbageErrorKind},
    scoring::score_hand
};


/// go through each of the 16 combinations looking for the hand
/// that will perform best based on the value of the hand plus
/// or minus the value of the crib
/// 
pub fn select_crib_cards(
    six_card_hand: &[Card],
    my_crib: bool,
) -> Result<Vec<Card>, CribbageError> {
    // get all possible hands
    let mut max_crib = Vec::<Card>::new();
    let mut max_score: f32 = -1000.0;

    if six_card_hand.len() != 6 {
        return Err(CribbageError::new(
            CribbageErrorKind::BadHand,
            format!(
                "a hand should have 6 cards to select a crib.  you passed in {}",
                six_card_hand.len()
            ),
        ));
    }

    let potential_hands = all_combinations_of_size(six_card_hand.to_vec(), 4, 4);

    for hand_to_try in potential_hands {
        if hand_to_try.len() != 4 {
            panic!("while looking for a crib, the hand size should be 4");
        }

        // get the score for the current hand we are evaluating
        let mut score: f32 = score_hand(hand_to_try.clone(), None, false).total_score as f32;
        let crib = get_crib_cards(&six_card_hand.to_vec(), &hand_to_try.clone());

        let mut expected_value: f32;
        if my_crib {
            expected_value =
                card_value_to_my_crib(crib[0].rank as usize - 1, crib[1].rank as usize - 1);
            if crib[0].suit == crib[1].suit {
                expected_value = expected_value + 0.01; // all things being equal, discard cards of the same suit
            }
            score = score + expected_value;
        } else {
            expected_value =
                card_value_to_your_crib(crib[0].rank as usize - 1, crib[1].rank as usize - 1);
            if crib[0].suit == crib[1].suit {
                expected_value = expected_value + 0.01; // all things being equal, discard cards of the same suit
            }
            score = score - expected_value;
        }

        if score > max_score {
            max_score = score;
            max_crib = crib.clone();
        }
    }

    Ok(max_crib)
}

static VALUE_MY_CRIB:[[f32;13];13] =
               [[5.26, 4.18, 4.47, 5.45, 5.48, 3.80, 3.73, 3.70, 3.33, 3.37, 3.65, 3.39, 3.42],
               [4.18, 5.67, 6.97, 4.51, 5.44, 3.87, 3.81, 3.58, 3.63, 3.51, 3.79, 3.52, 3.55] ,
               [4.47, 6.97, 5.90, 4.88, 6.01, 3.72, 3.67, 3.84, 3.66, 3.61, 3.88, 3.62, 3.66] ,
               [5.45, 4.51, 4.88, 5.65, 6.54, 3.87, 3.74, 3.84, 3.69, 3.62, 3.89, 3.63, 3.67] ,
               [5.48, 5.44, 6.01, 6.54, 8.95, 6.65, 6.04, 5.49, 5.47, 6.68, 7.04, 6.71, 6.70] ,
               [3.80, 3.87, 3.72, 3.87, 6.65, 5.74, 4.94, 4.70, 5.11, 3.15, 3.40, 3.08, 3.13] ,
               [3.73, 3.81, 3.67, 3.74, 6.04, 4.94, 5.98, 6.58, 4.06, 3.10, 3.43, 3.17, 3.21] ,
               [3.70, 3.58, 3.84, 3.84, 5.49, 4.70, 6.58, 5.42, 4.74, 3.86, 3.39, 3.16, 3.20] ,
               [3.33, 3.63, 3.66, 3.69, 5.47, 5.11, 4.06, 4.74, 5.09, 4.27, 3.98, 2.97, 3.05] ,
               [3.37, 3.51, 3.61, 3.62, 6.68, 3.15, 3.10, 3.86, 4.27, 4.73, 4.64, 3.36, 2.86] ,
               [3.65, 3.79, 3.88, 3.89, 7.04, 3.40, 3.43, 3.39, 3.98, 4.64, 5.37, 4.90, 4.07] ,
               [3.39, 3.52, 3.62, 3.63, 6.71, 3.08, 3.17, 3.16, 2.97, 3.36, 4.90, 4.66, 3.50] ,
               [3.42, 3.55, 3.66, 3.67, 6.70, 3.13, 3.21, 3.20, 3.05, 2.86, 4.07, 3.50, 4.62] ];


static VALUE_YOUR_CRIB:[[f32;13];13] = [
                [6.02,  5.07,   5.07,   5.72,   6.01,   4.91,   4.89,   4.85,   4.55,   4.48,   4.68,   4.33,   4.30],
                [5.07,  6.38,   7.33,   5.33,   6.11,   4.97,   4.97,   4.94,   4.70,   4.59,   4.81,   4.56,   4.45],
                [5.07,  7.33,   6.68,   5.96,   6.78,   4.87,   5.01,   5.05,   4.87,   4.63,   4.86,   4.59,   4.48],
                [5.72,  5.33,   5.96,   6.53,   7.26,   5.34,   4.88,   4.94,   4.68,   4.53,   4.85,   4.46,   4.36],
                [6.01,  6.11,   6.78,   7.26,   9.37,   7.47,   7.00,   6.30,   6.15,   7.41,   7.76,   7.34,   7.25],
                [4.91,  4.97,   4.87,   5.34,   7.47,   7.08,   6.42,   5.86,   6.26,   4.31,   4.57,   4.22,   4.14],
                [4.89,  4.97,   5.01,   4.88,   7.00,   6.42,   7.14,   7.63,   5.26,   4.31,   4.68,   4.32,   4.27],
                [4.85,  4.94,   5.05,   4.94,   6.30,   5.86,   7.63,   6.82,   5.83,   5.10,   4.59,   4.31,   4.20],
                [4.55,  4.70,   4.87,   4.68,   6.15,   6.26,   5.26,   5.83,   6.39,   5.43,   4.96,   4.11,   4.03],
                [4.48,  4.59,   4.63,   4.53,   7.41,   4.31,   4.31,   5.10,   5.43,   6.08,   5.63,   4.61,   3.88],
                [4.68,  4.81,   4.86,   4.85,   7.76,   4.57,   4.68,   4.59,   4.96,   5.63,   6.42,   5.46,   4.77],
                [4.33,  4.56,   4.59,   4.46,   7.34,   4.22,   4.32,   4.31,   4.11,   4.61,   5.46,   5.79,   4.49],
                [4.30,  4.45,   4.48,   4.36,   7.25,   4.14,   4.27,   4.20,   4.03,   3.88,   4.77,   4.49,   5.65]]; 


fn card_value_to_my_crib(rank1: usize, rank2: usize) -> f32 {    
    VALUE_MY_CRIB[rank1][rank2]
}

fn card_value_to_your_crib(rank1: usize, rank2: usize) -> f32 {    
    return VALUE_YOUR_CRIB[rank1][rank2];
}


/// hand has 6 cards and is passed in by the client
/// heldCards has 4 cards and is generated via permutation
/// this returns the 2 cards that are in the hand but not the crib
/// 
fn get_crib_cards(full_hand_of_six: &Vec<Card>, kept_cards: &Vec<Card>) -> Vec<Card> {
    let mut send_to_crib = Vec::<Card>::new();

    for card in full_hand_of_six.iter() {
        if !kept_cards.contains(card) {
            send_to_crib.push(*card);
        }
    }
    send_to_crib
}

/// called during counting phase
/// this api looks at the cards that have already been played and then the cards that could be played and tries to pick the right one
///
/// this is where "strategy" is implemented as selecting what cards to give to the crib is straightforward probability whereas counting
/// cards is largely a function of anticipating what the other player is going to play -- e.g. if you have a 2 and a 3, you might play the
/// 2 hoping that the other player has only cards with value = 10, so that you can play your 3 and get two points for the 15.  But the other
/// player knows this is a normal thing to do, so by playing a 2 you imply you also have a 3.  So maybe the right card to play is a 4 so that the
/// apponent plays the 3 to get the run (2, 4, 3) and 3 points - and then bam! -- you play an Ace to get 4 points.  and so on.
///
/// my strategy for this is to implement it the way I play the game.  by its nature it is a bunch of "if this then that" calls, so this function
/// tends to be very long and very experimental.  I have added logic in the past and then tested it by playing millions (literally) of games of
/// one algorithm against the other to see if it really works (it takes about 20 seconds to run a million games on a beefy PC...)
///

pub fn get_next_counted_card(
    played_cards: Vec<Card>,
    mut available_cards: Vec<Card>, // needs to be mut because we .sort() it
) -> Result<Option<Card>, CribbageError> {
    let current_count: i32 = played_cards.iter().map(|c| c.value).sum::<i32>();
    //
    //  if you only have one card left, play it if you can
    if available_cards.len() == 1 {
        if current_count + available_cards[0].value <= 31 {
            return Ok(Some(available_cards[0]));
        } else {
            return Ok(None);
        }
    }

    available_cards.sort(); // this is a sort by rank

    //  
    //  here is the first strategic decision - this algorithm will take points if they are available.  for example, when
    //  humans play, they might decide to defer points (say a pair) in order to get the opponent to start a run.  e.g. if cards_played is something like
    //  a 2, then it would be common for that person to also have a 3.  they played the 2 hoping that a card of value 10 would be played and then they
    //  could play a 3 - 15 for 2 points!.  This algorithm will always play a 2 to pair the 2 if it is possible.  Another strategy would be to play a 4
    //  or an Ace (*never* a 3 - because you don't want the count to be 5 because of how many cards are available with value=10).  in this case, the opponent
    //  would play the 3 and get a run of 3 for 3 points.  But then you could build on the run and get 4 points (assuming you had the right cards)
    //
    //  one of the things I want to explore is defeating the "get a 15" strategy above. the other common strategy (which we use below) is to start with a card
    //  that you have a pair...the hope is the apponent pairs the card, so you can get 3 of a kind (e.g. play a 4, opponent plays a 4, you play your second 4)
    //  would it be better to not pair the card and give up the two points?  over the course of the game, stopping 3 of a kind might be worth it.
    //
    //  this is an area ripe for innovation!
    //

    //
    //  Note: I need the card to play for the max score - i tried to do this in a variety of ways using map/fold and I could get the max score, but the card
    //        that generated the max score was lost in the map.  i gave up and did it via iteration.

    let mut max: i32 = -1;
    let mut card_to_play = Card::new(Rank::Unknown, Suit::Unknown);
    let mut playable_cards: Vec<Card> = Vec::new();
    for potential_card in available_cards.as_slice() {
        if let Ok(s) = score_counting_cards_played(played_cards.as_slice(), *potential_card) {
            if max < s.total_score as i32 {
                // innovation idea:  if you can play 2 cards and get the same score, which one should you play?
                max = s.total_score as i32;
                card_to_play = *potential_card;
            }

            playable_cards.push(*potential_card);
        }
    }

    if playable_cards.is_empty() {
        // this means we have no valid cards to play
        return Ok(None);
    }

    //
    //  if there is only one card that can be played, return it
    if playable_cards.len() == 1 {
        return Ok(Some(playable_cards[0]));
    }

    //
    //  get the most points - innovate here and pick the *best* one, not necessarily the one with most points!
    if max > 0 {
        return Ok(Some(card_to_play));
    }
    //
    //  we can play, but we can't get points.

    //
    //  find all combinations of 2 cards in the cards I have left...
    let two_card_combi = all_combinations_of_size(playable_cards.clone(), 2, 2);
    //
    //  we only want to interate through the 2 card combinations once, so
    //  we set a strategic weight to pick between the various scenarios and
    //  set the card_to_play when we find one
    //
    let mut strategic_weight = 0;
    let mut card_to_play: Card = Card::new(Rank::Unknown, Suit::Unknown);

    for mut cards in two_card_combi {
        if cards.len() != 2 {
            panic!("all_combinations_of_min_size returned the wrong size Vec!");
        }
        cards.sort(); // this sort might not be needed, but i'm not sure if all_combinations_of_size guarantees to returns sorted if the input was sorted
        if cards[0].rank == cards[1].rank
            && current_count + 3 * cards[0].value <= 31
            && strategic_weight < 10 && cards[1].value != 5
        {
            //
            //  this means that we have a pair and if the opponent plays the same card to get a pair, we can play our second to get 6 points
            //  an innovation to be tested here is to be careful with playing a 5 - which are typically held to try to get to a 15.
            card_to_play = cards[1];
            strategic_weight = 10;
        }
        //
        // I have no pairs, try to start a run
        match (cards[0].rank as i32 - cards[1].rank as i32).abs() {
            1 => {
                // there are 2 cards that the other person can play that help us - the values would be
                // cards[0]-1 or cards[0]+1 - we only check one here, and we pick the optimistic one
                // because if the card is bigger one, we might go over 31.  but we should run a test
                // to see how much it matters over the course of a game
                if cards[0].value - 1 + cards[0].value + cards[1].value + current_count <= 31
                    && strategic_weight < 5
                {
                    card_to_play = cards[1]; // I like to play the pick one when possible
                    strategic_weight = 5;
                }
            }
            2 => {
                // this means we are in a situation where we have something like a 7 and a 9, but no 8
                // cards[0].value + 1 is the value of the card we want the opponent to play
                //
                // i want to use "31" here because it is the key rule number, but the linter complains about
                //  the +1, so turn the linter rule off
                #[allow(clippy::int_plus_one)]
                if cards[0].value + cards[1].value + cards[0].value + 1 <= 31
                    && strategic_weight < 5
                {
                    if cards[1].rank != Rank::Five {
                        card_to_play = cards[1]; // I like to play the big one when possible
                    } else {
                        card_to_play = cards[0];
                    }

                    strategic_weight = 5;
                }
            }
            _ => {
                // we don't have cards within 1 rank of each other -- not worth trying to get a run
            }
        }
        //  remember the cards in case we need  to pick a card assuming that the other player only has cards of rank 10 to play left
        match cards[0].value + cards[1].value + 10 {
            15 | 21 => {
                if strategic_weight < 8 {
                    card_to_play = cards[1];
                    strategic_weight = 8;
                }
            }
            _ => {}
        }
    }

    //
    //  if we get here - we have no cards that we can play that give points, no runs we can try to start, and no pairs to leverage.
    //  so we will try to pick a card assuming that the other player only has cards of rank 10 to play left
    //
    //  Another innovation idea to test is that a human player will figure out what the game is always doing -- so, for example, it is easy
    //  to figure out that if the computer starts with (say) a 8, then they are probably trying to match a pair.  if they start with a card
    //  less than 5, they are trying to get to 15.  this extra information can be leveraged to help the player.  So instead we should figure
    //  out what valid cards could be played and then if then set strategic_weight to be the same for adding up to 15 and for pairs and then
    //  randomly pick which one to use.
    //

    if strategic_weight > 0 {
        if card_to_play.rank == Rank::Unknown {
            panic!("strategic weight is set but the card_to_play is not not");
        }
        return Ok(Some(card_to_play));
    }
    //
    //  if the last card (the highest value) is not a 5, return it
    //
    if playable_cards[playable_cards.len() - 1].value != 5 {
        return Ok(Some(playable_cards[playable_cards.len() - 1]));
    }
    //
    //  if it is a 5, return the next highest one -- we know it is not a pair because if it was, we'd have played it based on what we have above
    //  we also know that we have at least two cards, because if there was only one, we would have already picked it.
    Ok(Some(playable_cards[playable_cards.len() - 2]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::{Card, Rank::*, Suit as Of};
     use crate::new_card as c;

    macro_rules! test_case {
        (
    $name:ident,
    $counted_cards:expr,
    $cards_left:expr,
    $expected_card:expr,
    $expected_go:literal,
    $negative_test:literal        
    ) => {
            #[test]
            fn $name() {

                let result = super::get_next_counted_card($counted_cards, $cards_left);
                match result { // is there an error?
                    Err(e) => {
                        println!("an error returned: {:?}", e);
                        if ($negative_test == false) {
                            assert!($negative_test, "Unexpected Fail!");
                        }
                    }
                    Ok(card) => match card { // did I get a card back
                        Some(card) => {
                            println!("card returned: {}", card.name());
                            assert_eq!(
                                card.name(),
                                $expected_card.name(),
                                "unexpected card returned: {}.  expected: {}",
                                card.name(),
                                $expected_card.name()
                            );
                        }
                        None => {
                            if ($expected_go == false) {
                                assert!($expected_go, "Unexpected Go!");
                            }
                        }
                    },
                }
            }
        };
    }

    test_case!(
        first_card,
        [].to_vec(),
        [
            c!(Ace, Of::Spades),
            c!(Four, Of::Spades),
            c!(Six, Of::Spades),
            c!(Jack, Of::Spades)
        ]
        .to_vec(),
        c!(Four, Of::Spades),
        false,
        false
    );

    test_case!(
        second_card_hit_fifteen,
        [c!(Four, Of::Spades), c!(Ten, Of::Diamonds)].to_vec(),
        [
            c!(Ace, Of::Spades),
            c!(Six, Of::Spades),
            c!(Jack, Of::Spades)
        ]
        .to_vec(),
        c!(Ace, Of::Spades),
        false,false
    );

    test_case!(
        hit_31,
        [
            c!(Four, Of::Spades),
            c!(Ten, Of::Diamonds),
            c!(Ace, Of::Spades),
            c!(Jack, Of::Diamonds),
        ]
        .to_vec(),
        [c!(Six, Of::Spades), c!(Jack, Of::Hearts)].to_vec(),
        c!(Six, Of::Spades),
        false,false
    );

    test_case!(
        induce_run,
        [c!(Four, Of::Diamonds)].to_vec(),
        [
            c!(Jack, Of::Hearts),
            c!(Three, Of::Spades),
            c!(Five, Of::Spades),
            c!(Jack, Of::Spades)
        ]
        .to_vec(),
        c!(Three, Of::Spades),
        false,false
    );

    test_case!(
        run_of_4,
        [
            c!(Four, Of::Diamonds),
            c!(Three, Of::Spades),
            c!(Two, Of::Diamonds),
        ]
        .to_vec(),
        [
            c!(Six, Of::Hearts),  // this gives 15 for 2, but we can do better!
            c!(Five, Of::Spades), // this gives 4 points
            c!(Jack, Of::Spades)
        ]
        .to_vec(),
        c!(Five, Of::Spades),
        false,false
    );

    test_case!(
        induce_3_of_a_kind,
        [].to_vec(),
        [
            c!(Six, Of::Hearts),
            c!(Five, Of::Spades),
            c!(Jack, Of::Spades),
            c!(Jack, Of::Hearts)
        ]
        .to_vec(),
        c!(Jack, Of::Spades),
        false,false
    );

    test_case!(
        three_of_a_kind,
        [c!(Jack, Of::Spades), c!(Jack, Of::Diamonds),].to_vec(),
        [
            c!(Six, Of::Hearts),
            c!(Five, Of::Spades),
            c!(Jack, Of::Hearts)
        ]
        .to_vec(),
        c!(Jack, Of::Hearts),
        false,false
    );

    test_case!(
        test_go,
        [
            c!(Jack, Of::Spades),
            c!(Jack, Of::Diamonds),
            c!(Jack, Of::Hearts)
        ]
        .to_vec(),
        [c!(Six, Of::Hearts), c!(Five, Of::Spades),].to_vec(),
        c!(Jack, Of::Hearts),
        true,false
    );

    #[test]
    fn test_select_crib_cards_hand_returns_three_cards() {
        // prepare test parameters
        let test_hand = "FiveOfHearts,FiveOfClubs,SixOfHearts,SixOfClubs";
        let mut hand: Vec<Card> = Vec::new();

        for card_name in test_hand.split(',') {
            hand.push(Card::from_string(card_name).unwrap());
        }

        // execute the method under test
        let crib = select_crib_cards(&hand, true);

        // expect an error
        match crib {
            Ok(_) => {
                assert!(false, "this should be an error");
            }
            Err(e) => {
                assert_eq!(
                    e.error_kind,
                    CribbageErrorKind::BadHand,
                    "shoudl be a bad hand"
                );
            }
        };
    }

    #[test]
    fn test_get_crib_cards_match_expected_length() {
        // prepare test parameters
        let test_hand = "FiveOfHearts,FiveOfClubs,SixOfHearts,SixOfClubs";
        let held_cards = "AceOfSpades,FourOfDiamonds,SixOfClubs,JackOfDiamonds";
        let mut hand: Vec<Card> = Vec::new();
        let mut held: Vec<Card> = Vec::new();

        for card_name in test_hand.split(',') {
            hand.push(Card::from_string(card_name).unwrap());
        }

        for held_name in held_cards.split(',') {
            held.push(Card::from_string(held_name).unwrap());
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
            hand.push(Card::from_string(card_name).unwrap());
        }

        for held_name in held_cards.split(',') {
            held.push(Card::from_string(held_name).unwrap());
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
