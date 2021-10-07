#![allow(unused_imports)]
use std::convert::TryInto;

use actix_web::{get, web::Path, HttpRequest, HttpResponse, Responder};
use azure_core::response_from_headers;
use cribbage_library::cards::{Card, Rank, Suit};
use cribbage_library::counting::score_counting_cards_played;
use cribbage_library::cribbage_errors::{CribbageError, CribbageErrorKind};
use cribbage_library::scoring::{score_hand as scorehand, CombinationName, Score};
use cribbage_library::select_cards::{get_next_counted_card, select_crib_cards};
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};
use serde::Serialize;

const HOST_NAME: &'static str = "localhost:8088/api"; // important:  no ending '/'

#[allow(dead_code)]
#[allow(non_snake_case)] // backwards compatibility
#[derive(Debug, Serialize)]
struct CutCardResponse {
    Player: String,
    Computer: String,
    RepeatUrl: String,
}

#[allow(non_snake_case)] // backwards compatibility
#[derive(Debug, Serialize)]
struct ScoreInfo {
    ScoreName: CombinationName,
    Score: u32,
    Cards: Vec<Card>,
}

#[allow(non_snake_case)] // backwards compatibility
#[derive(Debug, Serialize)]
struct ScoreResponse {
    Score: u32,
    ScoreInfo: Vec<ScoreInfo>,
}

impl ScoreResponse {
    pub fn default() -> ScoreResponse {
        ScoreResponse {
            Score: 0,
            ScoreInfo: Vec::<ScoreInfo>::default(),
        }
    }
}

#[derive(Debug)]
struct ParsedHand {
    pub hand: Vec<Card>,
}

impl ParsedHand {
    pub fn from_string(csv: String) -> Result<ParsedHand, CribbageError> {
        let mut hand: Vec<Card> = Vec::<Card>::new();
        let tokens: Vec<&str> = csv.split(",").collect();
        for token in tokens {
            let _ = match Card::from_string(token) {
                Ok(card) => {
                    hand.push(card);
                }
                Err(e) => {
                    return Err(e);
                }
            };
        }

        Ok(ParsedHand { hand })
    }
}

//
//  i'm leaving the rust structs in  the library to look like rust structs, but will use this macro
//  to convert to the format that the client wants, which are based on javascript
#[allow(unused_macros)]
macro_rules! to_client_struct {
    ($score: expr) => {{
        let mut score_info = Vec::<ScoreInfo>::new();
        for s in $score.combinations {
            let si = ScoreInfo {
                ScoreName: s.name,
                Score: s.points,
                Cards: s.cards,
            };
            score_info.push(si);
        }
        ScoreResponse {
            Score: $score.total_score,
            ScoreInfo: score_info,
        }
    }};
}

//
//  converts a CSV of cards into a Vec<Card>
//  returns: Vec<Card> or HttpResponse::BadRequest
#[allow(unused_macros)]
macro_rules! csv_to_cards {
    ($csv: expr) => {{
        let mut hand: Vec<Card> = Vec::<Card>::new();
        let tokens: Vec<&str> = $csv.split(",").collect();
        let ret: Result<Vec<Card>, String>;

        for token in tokens {
            let _ = match Card::from_string(token) {
                Ok(card) => {
                    hand.push(card);
                }
                Err(e) => {
                    ret = Err(e.to_string());
                    return ret;
                }
            };
        }

        Ok(hand)
    }};
}

//
//  cut the cards to see who goes first
//
//  sample URLs:
//              http://localhost:8080/api/cutcards
//
//  returns: the two cut cards and the repeat URL.  the client is written to assume a shared notion of the deck
//           so we just return 2 numbers bewtween 0 and 51
//
pub async fn cut_cards() -> impl Responder {
    // let mut rng = rand::thread_rng();
    // let first = rng.gen_range(0..51);
    // let mut second:i32;
    // loop
    // {
    //     second = rng.gen_range(0..51);
    //     if first != second {
    //         break;
    //     }
    // }
    let c_card = Card::random_card();
    let mut p_card: String;
    //
    // better not be the same card...
    loop {
        p_card = Card::random_card();
        if p_card.eq(&c_card) == false {
            break;
        }
    }

    let response = CutCardResponse {
        Player: p_card.clone(),
        Computer: c_card.clone(),
        RepeatUrl: format!("{}/cutcards/{},{}", HOST_NAME, p_card, c_card),
    };

    HttpResponse::Ok().body(serde_json::to_string(&response).unwrap())
}
//
//  cut the cards to see who goes first - pass in the random numbers that you get the same result
//  the last time the api was called. useful for testing.
//
//  sample URLs:
//              http://localhost:8080/api/cutcards/AceOfSpades,TwoOfHearts
//
//  returns: the two cut cards
//
pub async fn cut_cards_repeat(cards: Path<String>) -> impl Responder {
    let cards = cards.into_inner();
    let tokens: Vec<&str> = cards.split(",").collect();
    if tokens.len() != 2 {
        return HttpResponse::BadRequest().body(
            "there should be two cards seperated by a ',' such as 'AceOfHearts,ThreeOfClubs'",
        );
    }
    let response = CutCardResponse {
        Player: tokens[0].to_string(),
        Computer: tokens[1].to_string(),
        RepeatUrl: format!("{}/cutcards/{},{}", HOST_NAME, tokens[0], tokens[1]),
    };

    HttpResponse::Ok().body(serde_json::to_string(&response).unwrap())
}
//
//  score the hand (or crib)
//
//  sample URLs:
//              localhost:8088/api/scorehand/FiveOfHearts,FiveOfClubs,FiveOfSpades,JackOfDiamonds/FourOfDiamonds/false
//              localhost:8088/api/scorehand/FiveOfHearts,SixOfHearts,SevenOfHearts,EightOfHearts/NineOfDiamonds/false  (should be a flush)
//              localhost:8088/api/scorehand/FiveOfHearts,SixOfHearts,SevenOfHearts,EightOfHearts/NineOfDiamonds/true   (no flush - need 5 of same suit in crib)
//              localhost:8088/api/scorehand/FiveOfHearts,SixOfHearts,SevenOfHearts,EightOfHearts/NineOfHearts/true     (should be a flush)
//              localhost:8088/api/scorehand/FiveOfHearts,SixOfHearts,FourOfHearts,FourOFClubs/SixOfDiamonds/true     (bad card)
//              localhost:8088/api/scorehand/FiveOfHearts,SixOfHearts,FourOfHearts,FourOfClubs/SixOfDiamonds/true     (double double run with 15s - 24 points)
//              localhost:8088/api/scorehand/ThreeOfSpades,TwoOfSpades,QueenOfHearts,QueenOfClubs/AceOfHearts/false
//
pub async fn score_hand(path: Path<(String, String, bool)>) -> impl Responder {
    let path = path.into_inner();
    let parsed_hand = match ParsedHand::from_string(path.0) {
        Ok(parsed_hand) => parsed_hand,
        Err(e) => {
            return HttpResponse::BadRequest().body(serde_json::to_string(&e).unwrap());
        }
    };

    let shared_card = match Card::from_string(&path.1) {
        Ok(shared_card) => shared_card,
        Err(e) => {
            return HttpResponse::BadRequest().body(serde_json::to_string(&e).unwrap());
        }
    };

    let score: Score = scorehand(parsed_hand.hand, Some(shared_card), path.2);

    let score_response: ScoreResponse = to_client_struct!(score);

    HttpResponse::Ok().body(serde_json::to_string(&score_response).unwrap())
}
//
//  given 6 cards, return 2.  if isMyCrib is true, then optimize to make the hand + crib have the most points possible
//
//  sample URLs:
//   localhost:8088/api/getcribcards/FiveOfHearts,FiveOfClubs,FiveOfSpades,JackOfDiamonds,SixOfClubs,FourOfDiamonds/false
//   localhost:8088/api/getcribcards/FiveOfHearts,FiveOfClubs,FiveOfSpades,JackOfDiamonds,SixOfClubs,FourOfDiamonds/true
//   localhost:8088/api/getcribcards/FourOfHearts,FiveOfHearts,SixOfSpades,JackOfHearts,QueenOfHearts,SixOfDiamonds/true
//   localhost:8088/api/getcribcards/FourOfHearts,FiveOfHearts,SixOfSpades,JackOfHearts,QueenOfHearts,SixOfDiamonds/false
//
//
pub async fn get_crib(path: Path<(String, bool)>) -> impl Responder {
    let path = path.into_inner();

    let parsed_hand = match ParsedHand::from_string(path.0) {
        Ok(parsed_hand) => parsed_hand,
        Err(e) => {
            return HttpResponse::BadRequest().body(serde_json::to_string(&e).unwrap());
        }
    };

    let crib = select_crib_cards(parsed_hand.hand.as_slice(), path.1);

    HttpResponse::Ok().body(serde_json::to_string(&crib).unwrap())
}
//
//  struct for returning the counted card == the client expects something like this:
//  {
//     "countedCard": {
//         "OrdinalName": "Ace",
//         "Rank": 1,
//         "Value": 1,
//         "Suit": "Spades",
//         "cardName": "AceOfSpades",
//         "Owner": "shared",
//         "Ordinal": 1
//     },
//     "Scoring": {
//         "Score": 0,
//         "ScoreInfo": []
//     }
// }
//
#[allow(non_snake_case)] // backwards compatibility
#[derive(Debug, Serialize)]
struct CountedCardResponse {
    countedCard: Option<Card>,
    Scoring: ScoreResponse,
}
#[allow(dead_code)]
impl CountedCardResponse {
    fn new() -> CountedCardResponse {
        CountedCardResponse {
            countedCard: None,
            Scoring: ScoreResponse::default(),
        }
    }
}

//
//  URL example:
//          localhost:8088/api/getnextcountedcard/AceOfSpades,AceOfHearts,TwoOfClubs,TenOfDiamonds/0
//          localhost:8088/api/getnextcountedcard/FiveOfClubs,QueenOfDiamonds/25/ThreeOfDiamonds,TenOfClubs,TwoOfSpades,QueenOfSpades
//          localhost:8088/api/getnextcountedcard/SixOfClubs,QueenOfDiamonds/25/ThreeOfDiamonds,TenOfClubs,TwoOfSpades,QueenOfSpades
//          localhost:8088/api/getnextcountedcard/SixOfClubs,QueenOfDiamonds/25/ThreeOfDiamonds,TenOfClubs,TwoOfSpades,QueenOfSpades
//
//  Note that the last parameters contains all the cards that have already been counted, which means it starts empty, so there are two routes.
//
pub async fn get_first_counted_card(path: Path<(String, u32)>) -> impl Responder {
    let path = path.into_inner();
    let available_cards = match ParsedHand::from_string(path.0) {
        Ok(parsed_hand) => parsed_hand.hand,
        Err(e) => {
            return HttpResponse::BadRequest().body(serde_json::to_string(&e).unwrap());
        }
    };

    let response = internal_get_next_counted_card(Vec::<Card>::new(), available_cards);
    return HttpResponse::Ok().body(serde_json::to_string(&response).unwrap());
}

pub async fn next_counted_card(path: Path<(String, u32, String)>) -> impl Responder {
    let path = path.into_inner();
    let available_cards = match ParsedHand::from_string(path.0) {
        Ok(parsed_hand) => parsed_hand.hand,
        Err(e) => {
            return HttpResponse::BadRequest().body(serde_json::to_string(&e).unwrap());
        }
    };

    let played_cards = match ParsedHand::from_string(path.2) {
        Ok(played_cards) => played_cards.hand,
        Err(e) => {
            return HttpResponse::BadRequest().body(serde_json::to_string(&e).unwrap());
        }
    };

    let response = internal_get_next_counted_card(played_cards, available_cards);
    return HttpResponse::Ok().body(serde_json::to_string(&response).unwrap());
}

fn internal_get_next_counted_card(
    played_cards: Vec<Card>,
    available_cards: Vec<Card>,
) -> CountedCardResponse {
    let _ = match get_next_counted_card(played_cards.clone(), available_cards) {
        Ok(card) => {
            let _ = match card {
                Some(card) => {
                    let score = score_counting_cards_played(played_cards.as_slice(), card).unwrap(); // this can't be an error because we are in the Ok() block
                    return CountedCardResponse {
                        countedCard: Some(card),
                        Scoring: to_client_struct!(score),
                    };
                }
                None => {
                    return CountedCardResponse {
                        countedCard: None,
                        Scoring: ScoreResponse::default(),
                    };
                }
            };
        }
        Err(_e) => {
            return CountedCardResponse {
                countedCard: None,
                Scoring: ScoreResponse::default(),
            };
        }
    };
}

pub async fn score_first_counted_card(path: Path<(String, u32)>) -> impl Responder {
    let path = path.into_inner();
    if path.1 != 0 {
        let err = CribbageError::new(
            CribbageErrorKind::BadCount,
            format!("count shoudl be 0 instead of {}", path.1),
        );
        return HttpResponse::BadRequest().body(serde_json::to_string(&err).unwrap());
    }

    return HttpResponse::Ok().body(serde_json::to_string(&ScoreResponse::default()).unwrap());
}

pub async fn score_counted_cards(path: Path<(String, u32, String)>) -> impl Responder {
    let path = path.into_inner();
    let played_cards = match ParsedHand::from_string(path.2) {
        Ok(parsed_hand) => parsed_hand.hand,
        Err(e) => {
            return HttpResponse::BadRequest().body(serde_json::to_string(&e).unwrap());
        }
    };

    let card = match Card::from_string(&path.0) {
        Ok(card) => card,
        Err(e) => {
            return HttpResponse::BadRequest().body(serde_json::to_string(&e).unwrap());
        }
    };

    let _ = match score_counting_cards_played(played_cards.as_slice(), card) {
        Ok(score) => {
            let score_response: ScoreResponse = to_client_struct!(score);

            return HttpResponse::Ok().body(serde_json::to_string(&score_response).unwrap());
        }

        Err(e) => {
            return HttpResponse::BadRequest().body(serde_json::to_string(&e).unwrap());
        }
    };
}
#[allow(non_snake_case)] // backwards compatibility
#[derive(Debug, Serialize)]
struct RandomHandResponse {
    RandomCards: Vec<ClientCard>,
    ComputerCribCards: Vec<ClientCard>,
    SharedCard: ClientCard,
    HiNobs: bool,
    RepeatUrl: String,
}
impl RandomHandResponse {
    pub fn default() -> RandomHandResponse {
        RandomHandResponse {
            RandomCards: Vec::<ClientCard>::default(),
            ComputerCribCards: Vec::<ClientCard>::default(),
            SharedCard: ClientCard::default(),
            HiNobs: false,
            RepeatUrl: "".to_string(),
        }
    }
}
#[allow(non_snake_case)] // backwards compatibility
#[derive(Debug, Serialize, Clone)]
struct ClientCard {
    OrdinalName: Rank,
    Rank: i32,
    Value: i32,
    Suit: Suit,
    cardName: String,
    Owner: String,
    Ordinal: i32,
}

impl ClientCard {
    pub fn from_card(card: Card, owner: String) -> ClientCard {
        ClientCard {
            OrdinalName: card.rank,
            Rank: card.rank as i32,
            Value: card.value,
            Suit: card.suit,
            cardName: format!("{:?}Of{:?}", card.rank, card.suit),
            Owner: owner,
            Ordinal: card.rank as i32,
        }
    }

    pub fn default() -> ClientCard {
        let c = Card::default();
        ClientCard::from_card(c, "unknown".to_string())
    }
}

fn get_random_hand_internal(
    is_computer_crib: bool,
    cards: Vec<usize>,
) -> Option<RandomHandResponse> {
    if cards.len() < 13 {
        return None;
    }
    let owner_array: [String; 2] = ["player".to_string(), "computer".to_string()];
    let mut toggle_owner: usize = 1; // which owner gets the first card?
    if is_computer_crib {
        toggle_owner = 0;
    }

    let mut indices: String = "".to_owned();
    let mut response = RandomHandResponse::default();
    let mut computer_hand: Vec<Card> = Vec::<Card>::default();
    for i in 0..12 {
        let index = cards[i];
        if owner_array[toggle_owner] == "computer" {
            computer_hand.push(Card::from_index(index));
        }
        response.RandomCards.push(ClientCard::from_card(
            Card::from_index(index),
            owner_array[toggle_owner].to_string(),
        ));
        indices.push_str(&format!("{},", index));
        toggle_owner = 1 - toggle_owner;
    }
    indices.pop(); // remove the trailing ","
    response.SharedCard = ClientCard::from_card(Card::from_index(cards[12]), "shared".to_string());
    response.RandomCards.insert(0, response.SharedCard.clone()); // to avoid "partially borrowing" the response object

    if is_computer_crib {
        let crib_cards = select_crib_cards(computer_hand.as_slice(), true).unwrap();
        for card in crib_cards.iter() {
            response
                .ComputerCribCards
                .push(ClientCard::from_card(*card, "computer".to_string()));
        }
    }

    response.RepeatUrl = format!("{}/getrandomhand/{}/{}/{}", HOST_NAME, is_computer_crib, indices, cards[12]);
    Some(response)
}

pub async fn get_random_hand(path: Path<bool>) -> impl Responder {
    let is_computer_crib = path.into_inner();

    let mut rng = rand::thread_rng();
    let mut deck = (0..51).collect::<Vec<_>>();
    deck.shuffle(&mut rng);

    let response = match get_random_hand_internal(is_computer_crib, deck) {
        Some(response) => response,
        None => {
            return HttpResponse::BadRequest().body(
                serde_json::to_string(&CribbageError::new(
                    CribbageErrorKind::BadCount,
                    "Error getting random hand".to_string(),
                ))
                .unwrap(),
            );
        }
    };

    HttpResponse::Ok().body(serde_json::to_string(&response).unwrap())
}
pub async fn get_random_hand_repeat(path: Path<(bool, String, String)>) -> impl Responder {
    
    let path = path.into_inner();
    let is_computer_crib = path.0;

    let shared_card_index = match path.2.parse::<usize>() {
            Ok(shared_card_index) => shared_card_index,
            Err(_) => {
                let msg = format!("unable to parse {} into usize.  Bad shared card index.", path.2);
                return HttpResponse::BadRequest().body(
                    serde_json::to_string(&CribbageError::new(CribbageErrorKind::BadCount, msg))
                        .unwrap(),
                );
            }
        };



    let tokens = path.1.split(",").collect::<Vec<_>>();
    if tokens.len() != 12 {
        let msg = format!(
            "Expected 12 tokens and got {} instead the CSV of indices is incorrect",
            tokens.len()
        );
        return HttpResponse::BadRequest().body(
            serde_json::to_string(&CribbageError::new(CribbageErrorKind::BadCount, msg)).unwrap(),
        );
    };

    let mut indices = Vec::<usize>::new();

    for token in tokens {
        let _ = match token.parse::<usize>() {
            Ok(index) => indices.push(index),
            Err(_) => {
                let msg = format!("unable to parse {} into usize.  Bad CSV.", token);
                return HttpResponse::BadRequest().body(
                    serde_json::to_string(&CribbageError::new(CribbageErrorKind::BadCount, msg))
                        .unwrap(),
                );
            }
        };
    }
    indices.push(shared_card_index);
    let response = match get_random_hand_internal(is_computer_crib, indices) {
        Some(response) => response,
        None => {
            return HttpResponse::BadRequest().body(
                serde_json::to_string(&CribbageError::new(
                    CribbageErrorKind::BadCount,
                    "Error getting random hand".to_string(),
                ))
                .unwrap(),
            );
        }
    };

    HttpResponse::Ok().body(serde_json::to_string(&response).unwrap())
}
