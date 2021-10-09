use cribbage_library::cards::{Card, Rank, Suit};
use cribbage_library::cribbage_errors::CribbageError;
use cribbage_library::scoring::{Combination, CombinationName, Score};
use serde::{Deserialize, Serialize};
#[allow(dead_code)]
#[allow(non_snake_case)] // backwards compatibility
#[derive(Debug, Serialize, Deserialize)]
pub struct CutCards {
    pub Player: ClientCard,
    pub Computer: ClientCard,
}
impl CutCards {
    pub fn new(p_index: usize, c_index: usize) -> CutCards {
        CutCards {
            Player: ClientCard::from_card(Card::from_index(p_index), "Player".to_string()),
            Computer: ClientCard::from_card(Card::from_index(c_index), "Computer".to_string()),
        }
    }
}
#[allow(dead_code)]
#[allow(non_snake_case)] // backwards compatibility
#[derive(Debug, Serialize, Deserialize)]
pub struct CutCardResponse {
    pub CutCards: CutCards,
    pub RepeatUrl: String,
}

#[allow(non_snake_case)] // backwards compatibility
#[derive(Debug, Serialize, Deserialize)]
pub struct ScoreInfo {
    pub ScoreName: CombinationName,
    pub Score: u32,
    pub Cards: Vec<ClientCard>,
}

impl ScoreInfo {
    pub fn from_combination(combi: Combination) -> ScoreInfo {
        let mut client_cards: Vec<ClientCard> = Vec::<ClientCard>::default();
        for card in combi.cards.into_iter() {
            client_cards.push(ClientCard::from_card(card, "unknown".to_string()));
        }

        ScoreInfo {
            ScoreName: combi.name,
            Score: combi.points,
            Cards: client_cards,
        }
    }
}

#[allow(non_snake_case)] // backwards compatibility
#[derive(Debug, Serialize, Deserialize)]
pub struct ScoreResponse {
    pub Score: u32,
    pub ScoreInfo: Vec<ScoreInfo>,
}

impl ScoreResponse {
    pub fn default() -> ScoreResponse {
        ScoreResponse {
            Score: 0,
            ScoreInfo: Vec::<ScoreInfo>::default(),
        }
    }

    pub fn from_score(score: Score) -> ScoreResponse {
        let mut scores = Vec::<ScoreInfo>::new();
        score.combinations.into_iter().for_each(|combi| {
            scores.push(ScoreInfo::from_combination(combi));
        });
        ScoreResponse {
            Score: score.total_score,
            ScoreInfo: scores,
        }
    }
}

#[derive(Debug)]
pub struct ParsedHand {
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
//  converts a CSV of cards into a Vec<Card>
//  returns: Vec<Card> or HttpResponse::BadRequest
#[allow(unused_macros)]
#[macro_export]
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

#[allow(non_snake_case)] // backwards compatibility
#[derive(Debug, Serialize)]
pub struct RandomHandResponse {
    pub RandomCards: Vec<ClientCard>,
    pub ComputerCribCards: Vec<ClientCard>,
    pub SharedCard: ClientCard,
    pub HiNobs: bool,
    pub RepeatUrl: String,
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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientCard {
    OrdinalName: Rank,
    Rank: i32,
    Value: i32,
    Suit: Suit,
    pub cardName: String,
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
pub struct CountedCardResponse {
    pub countedCard: Option<ClientCard>,
    pub Scoring: ScoreResponse,
}
#[allow(dead_code)]
impl CountedCardResponse {
    pub fn new() -> CountedCardResponse {
        CountedCardResponse {
            countedCard: None,
            Scoring: ScoreResponse::default(),
        }
    }
}
