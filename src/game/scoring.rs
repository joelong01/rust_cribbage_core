use crate::game::cards::*;
use arrayvec::IntoIter;
use itertools::*;
use std::convert::TryInto;

pub fn score_hand(hand: &[Card], shared_card: &Card, is_crib: bool) -> i32 {
    let mut local_score = score_nibs(hand, shared_card);

    let mut local_hand: Vec<Card> = hand.to_vec();
    local_hand.push(shared_card.clone());

    local_score += score_flush(&local_hand, is_crib);

    local_hand.sort_by_key(|c| c.rank());

    local_score += score_fifteens(&local_hand);

    local_score += score_runs(&local_hand);

    local_score += score_same_kind(&local_hand);

    local_score
}

fn score_same_kind(hand: &[Card]) -> i32 {
    let mut local_score = 0;    
    for cards in hand.iter().combinations(2).into_iter(){
        if cards[0].ordinal() == cards[1].ordinal()  {
            local_score += 2;
        }
    }

    local_score
}

fn score_nibs(hand: &[Card], shared_card: &Card) -> i32 {
    if hand
        .iter()
        .any(|card| card.suit() == shared_card.suit() && card.ordinal() == Ordinal::Jack)
    {
        return 1;
    }

    0
}

fn is_run(hand: &[Card]) -> bool {
    hand.iter()
        .map(|c| c.ordinal() as usize - hand[0].ordinal() as usize)
        .eq(0..hand.len())
}

fn is_run_by_ref(hand: &[&Card]) -> bool {
    hand.iter()
        .map(|c| c.ordinal() as usize - hand[0].ordinal() as usize)
        .eq(0..hand.len())
}

fn score_runs(hand: &[Card]) -> i32 {
    //
    //  first check if the whole thing is a run
    if is_run(hand) {
        return hand.len().try_into().unwrap();
    }

    //
    //  now look for 3 or 4 card runs - you can have 2 four card runs
    //  but you can't have both a 4 card run and a 3 card run (3+4=7)
    let mut local_score:i32 = 0;

    let mut combi = hand.iter().combinations(4);
    for cards in combi.into_iter(){
        if is_run_by_ref(&cards) {
            local_score += 4;
        }
    }
    
    if local_score > 0 {
        return local_score;
    }

    //
    //  now look for 3 card runs
    //  you can have multiple of these (up to 4)
    combi = hand.iter().combinations(3);
    for cards in combi.into_iter(){
        if is_run_by_ref(&cards) {
            local_score += 3;
        }
    }

    local_score
}

fn score_fifteens(hand: &[Card]) -> i32 {
    let mut local_score: i32 = 0;
    let total: i32 = hand.iter().map(|x| x.value() as i32).sum();
    if total == 15 {
        local_score = 2;
    }

    for length in 2..hand.len() - 1 {
        let comb = hand.iter().combinations(length);
        for set in comb.into_iter() {
            let sum: i32 = set.iter().map(|x| x.value() as i32).sum();
            if sum == 15 {
                local_score += 2;
            }
        }
    }

    local_score
}

fn score_flush(hand: &[Card], is_crib: bool) -> i32 {
    if hand.iter().all(|card| card.suit() == hand[0].suit()) {
        return hand.len().try_into().unwrap();
    }

    if is_crib {
        return 0;
    }

    let combis = hand.iter().combinations(4);
    for set in combis.into_iter() {
        if set.iter().all(|card| card.suit() == hand[0].suit()) {
            return 4;
        }
    }

    0
}
