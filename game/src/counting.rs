use crate::cards::Card;
use crate::scoring::{score_run, Combination, CombinationKind, Score};
use std::error::Error;

/// Calculates the score during the counting phase where played_cards have already been played and card is now played
///
///
pub fn score_counting_cards_played(
    played_cards: &[Card],
    card: Card,
) -> Result<Score, Box<dyn Error>> {
    let count: i32 = played_cards.iter().map(|c| c.value).sum::<i32>() + card.value;
    if count > 31 {
        return Err("invalid card + count > 31".into());
    }

    let mut score: Score = Score::new();
    let mut all_cards: Vec<Card> = played_cards.to_vec();

    all_cards.push(card);
    if count == 15 {
        score.combinations.push(Combination::new(
            CombinationKind::Fifteen,
            all_cards.clone(),
        ));
    } else if count == 31 {
        score.combinations.push(Combination::new(
            CombinationKind::ThirtyOne,
            all_cards.clone(),
        ));
    }

    //
    //  check to see if we have matching rank
    let mut len = played_cards.len();
    let mut count = 0;
    if len > 0 {
        for i in 1..=3 {
            if i > len {
                // we need to loop for up to 4 of a kind, but not if we don't have 4 cards...
                break;
            }
            if played_cards[len - i..].iter().all(|c| c.rank == card.rank)
            // card is the played card passed in
            {
                count += 1;
            } else {
                break;
            }
        }
    }

    if count > 0 {
        score.combinations.push(Combination::new(
            CombinationKind::RankMatch,
            all_cards.as_slice()[len - count..].to_vec(),
        ));
    }

    //
    //   now look for runs - this is a bit complicated in counting because the run doesn't have to be in order
    //   e.g. Ace, Five, Four, Two, Three is worth 5 points to whoever played the three.
    //
    len = all_cards.len();
    if len > 2 {
        for i in 0..all_cards.len() - 1 {
            // get a slice of the last i cards, sort them, and see if they are a run.
            // we can't sort the slice because they sort in place and return ()
            let mut possible_run = all_cards.as_slice()[i..].to_vec();
            possible_run.sort();
            if let Some(c) = score_run(possible_run) {
                score.combinations.push(c);
                break; // stop on the largest run
            }
        }
    }

    score.total_score = score.combinations.iter().map(|combi| combi.points).sum();
    Ok(score)
}

#[cfg(test)]
mod tests {
    use crate::cards::{Card, Rank::*, Suit as Of};
    use card as c;

    macro_rules! test_case {
        ($name:ident,$cards_played:expr,$card:expr,$expected_score:literal, $expect_error:literal) => {
            #[test]
            fn $name() {
                let computer_score = super::score_counting_cards_played($cards_played, $card);

                match computer_score {
                    Ok(s) => {
                        assert_eq!(
                            $expected_score, s.total_score,
                            "Player Algo Score: {} vs. Hand Score: {}",
                            s.total_score, $expected_score
                        );
                    }
                    Err(e) => {
                        if $expect_error == false {
                            assert_eq!(true, false, "Error in scoring: {:?}", e);
                        }
                    }
                }
            }
        };
    }
    test_case!(counting_first_card, &[], c!(Five, Of::Diamonds), 0, false);
    test_case!(
        counting_2_fives,
        &[c!(Five, Of::Hearts),],
        c!(Five, Of::Diamonds),
        2,
        false
    );
    test_case!(
        counting_3_fives,
        &[c!(Five, Of::Hearts), c!(Five, Of::Clubs),],
        c!(Five, Of::Diamonds),
        8,
        false
    );
    test_case!(
        counting_4_fives,
        &[
            c!(Five, Of::Hearts),
            c!(Five, Of::Clubs),
            c!(Five, Of::Diamonds),
        ],
        c!(Five, Of::Spades),
        12,
        false
    );
    test_case!(
        counting_no_points,
        &[
            c!(Five, Of::Hearts),
            c!(Five, Of::Clubs),
            c!(Five, Of::Diamonds),
            c!(Five, Of::Spades),
        ],
        c!(Ten, Of::Spades),
        0,
        false
    );
    test_case!(
        counting_expect_error,
        &[
            c!(Five, Of::Hearts),
            c!(Five, Of::Clubs),
            c!(Five, Of::Diamonds),
            c!(Five, Of::Spades),
            c!(Ten, Of::Spades),
        ],
        c!(Ten, Of::Clubs),
        0,
        true
    );
    test_case!(
        counting_31,
        &[
            c!(Five, Of::Hearts),
            c!(Five, Of::Clubs),
            c!(Five, Of::Diamonds),
            c!(Five, Of::Spades),
            c!(Ten, Of::Spades),
        ],
        c!(Ace, Of::Clubs),
        2,
        false
    );

    test_case!(
        run_of_three,
        &[c!(Five, Of::Hearts), c!(Four, Of::Clubs),],
        c!(Six, Of::Clubs),
        5,
        false
    );
    test_case!(
        no_run_too_short,
        &[c!(Five, Of::Hearts)],
        c!(Six, Of::Clubs),
        0,
        false
    );

    test_case!(
        run_of_four,
        &[
            c!(Five, Of::Hearts),
            c!(Four, Of::Clubs),
            c!(Six, Of::Clubs),
        ],
        c!(Three, Of::Clubs),
        4,
        false
    );
    test_case!(
        run_of_five,
        &[
            c!(Five, Of::Hearts),
            c!(Four, Of::Clubs),
            c!(Six, Of::Clubs),
            c!(Two, Of::Clubs)
        ],
        c!(Three, Of::Clubs),
        5,
        false
    );
    test_case!(
        run_of_six,
        &[
            c!(Five, Of::Hearts),
            c!(Four, Of::Clubs),
            c!(Six, Of::Clubs),
            c!(Two, Of::Clubs),
            c!(Three, Of::Clubs)
        ],
        c!(Seven, Of::Clubs),
        6,
        false
    );
    test_case!(
        run_of_seven,
        &[
            c!(Five, Of::Hearts),
            c!(Four, Of::Clubs),
            c!(Six, Of::Clubs),
            c!(Two, Of::Clubs),
            c!(Three, Of::Clubs),
            c!(Seven, Of::Clubs)
        ],
        c!(Ace, Of::Clubs),
        7,
        false
    );
    test_case!(
        second_run_of_three,
        &[
            c!(Five, Of::Hearts),
            c!(Four, Of::Clubs),
            c!(Six, Of::Clubs),
        ],
        c!(Five, Of::Clubs),
        3,
        false
    );

    test_case!(
        run_of_3_on_the_end,
        &[
            c!(Five, Of::Hearts),
            c!(Four, Of::Clubs),
            c!(Six, Of::Clubs),
            c!(Five, Of::Clubs)
        ],
        c!(Four, Of::Clubs),
        3,
        false
    );
    test_case!(
        broken_run,
        &[
            c!(Five, Of::Hearts),
            c!(Four, Of::Clubs),
            c!(Four, Of::Clubs),
            c!(Three, Of::Clubs)
        ],
        c!(Six, Of::Clubs),
        0,
        false
    );
}
