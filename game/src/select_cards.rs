#![allow(unused_imports)]
use crate::cards::Card;
use crate::cards::{Rank, Suit};
use crate::combinator::all_combinations_of_size;
use crate::counting::score_counting_cards_played;
use crate::scoring::{score_hand, Score};
use std::error::Error;

/**
 * go through each of the 16 combinations looking for the hand
 * that will perform best based on the value of the hand plus
 * or minus the value of the crib
 */
pub fn select_crib_cards(hand: &[Card], _: bool) -> Vec<Card> {
    // get all possible hands
    let mut max_crib = Vec::<Card>::new();
    let mut max_score: i32 = -1000;

    let potential_hands = all_combinations_of_size(hand.to_vec(), 4, 4);

    for p in potential_hands {
        for h in p {
            // get the score for the current hand we are evaluating
            let score: i32 = score_hand(hand.to_vec(), None, false).total_score as i32;
            let crib = get_crib_cards(hand, &[h]);

            // TODO: implement CardScoring.getCardValueToYourCrib

            if score > max_score {
                max_score = score;
                max_crib = crib.clone();
            }
        }
    }

    max_crib
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
        if !held_cards.contains(h) {
            send_to_crib.push(*h);
        }
    }
    send_to_crib
}
/**
 *  called during counting phase
 *  this api looks at the cards that have already been played and then the cards that could be played and tries to pick the right one
 *
 *  this is where "strategy" is implemented as selecting what cards to give to the crib is straightforward probability whereas counting
 *  cards is largely a function of anticipating what the other player is going to play -- e.g. if you have a 2 and a 3, you might play the
 *  2 hoping that the other player has only cards with value = 10, so that you can play your 3 and get two points for the 15.  But the other
 *  player knows this is a normal thing to do, so by playing a 2 you imply you also have a 3.  So maybe the right card to play is a 4 so that the
 *  apponent plays the 3 to get the run (2, 4, 3) and 3 points - and then bam! -- you play an Ace to get 4 points.  and so on.
 *
 *  my strategy for this is to implement it the way I play the game.  by its nature it is a bunch of "if this then that" calls, so this function
 *  tends to be very long and very experimental.  I have added logic in the past and then tested it by playing millions (literally) of games of
 *  one algorithm against the other to see if it really works (it takes about 20 seconds to run a million games on a beefy PC...)
 *
 *
 */
pub fn get_next_counted_card(
    counted_cards: Vec<Card>,
    mut cards_left: Vec<Card>, // needs to be mut because we .sort() it
) -> Result<Card, Box<dyn Error>> {
    let current_count: i32 = counted_cards.iter().map(|c| c.value).sum::<i32>();
    //
    //  if you only have one card left, play it if you can
    if cards_left.len() == 1 {
        if current_count + cards_left[0].value <= 31 {
            return Ok(cards_left[0]);
        } else {
            return Err("Go".into());
        }
    }

    cards_left.sort(); // this is a sort by rank

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
    for potential_card in cards_left.as_slice() {
        if let Ok(s) = score_counting_cards_played(counted_cards.as_slice(), *potential_card) {
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
        return Err("Go".into());
    }
    //
    //  get the most points - innovate here and pick the *best* one, not necessarily the one with most points!
    if max > 0 {
        return Ok(card_to_play);
    }
    //
    //  we can play, but we can't get points.

    //
    //  find all combinations of 2 cards in the cards I have left...
    let two_card_combi = all_combinations_of_size(playable_cards, 2, 2);
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
        cards.sort();   // this sort might not be needed, but i'm not sure if all_combinations_of_size guarantees to returns sorted if the input was sorted
        if cards[0].rank == cards[1].rank
            && current_count + 3 * cards[0].value <= 31
            && strategic_weight < 10
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
        return Ok(card_to_play);
    }
    //
    //  if the last card (the highest value) is not a 5, return it
    //
    if cards_left[cards_left.len() - 1].value != 5 {
        return Ok(cards_left[cards_left.len() - 1]);
    }
    //
    //  if it is a 5, return the next highest one -- we know it is not a pair because if it was, we'd have played it based on what we have above
    //  we also know that we have at least two cards, because if there was only one, we would have already picked it.
    Ok(cards_left[cards_left.len() - 2])
}
#[cfg(test)]
mod tests {
    use crate::cards::{Card, Rank::*, Suit as Of};
    use card as c;
    use super::*;

    macro_rules! test_case {
        (
    $name:ident,
    $counted_cards:expr,
    $cards_left:expr,        
    $expected_card:expr,   
    $expected_go:literal,             

    ) => {
            #[test]
            fn $name() {
                let result = super::get_next_counted_card($counted_cards, $cards_left);
                match result {
                    Err(e) => {
                        println!("an error returned: {:?}", e);
                        if ($expected_go == false) {
                            assert!($expected_go, "Unexpected Go!");
                        }
                    }
                    Ok(card) => {
                        println!("card returned: {}", card.name());
                        assert_eq!(
                            card.name(),
                            $expected_card.name(),
                            "unexpected card returned: {}.  expected: {}",
                            card.name(),
                            $expected_card.name()
                        );
                    }
                }
            }
        };
    }
    /*
        Testing the pick cards agorithm.  this is a little strange because innovation will cause different cards to be picked and then
        we'll have to update the tests.

    */
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
        false,
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
        false,
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
        false,
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
        false,
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
        false,
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
        false,
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
        true,
    );

     #[test]
    fn test_select_crib_cards_hand_returns_three_cards() {
        // prepare test parameters
        let test_hand = "FiveOfHearts,FiveOfClubs,SixOfHearts,SixOfClubs";
        let mut hand: Vec<Card> = Vec::new();

        for card_name in test_hand.split(',') {
            hand.push(Card::from_string(card_name));
        }

        // execute the method under test
        let crib = select_crib_cards(&hand, true);

        // returned crib len should equal 3 cards given the inputs
        assert_eq!(crib.len(), 3);
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
