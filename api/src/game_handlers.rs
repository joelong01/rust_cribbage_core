use crate::client_structs::{
    ClientCard, CountedCardResponse, CutCardResponse, CutCards, ParsedHand, RandomHandResponse,
    ScoreResponse,
};
use actix_web::{web::Path, HttpRequest, HttpResponse, Responder};
use cribbage_library::{
    cards::Card,
    counting::score_counting_cards_played,
    cribbage_errors::{CribbageError, CribbageErrorKind},
    scoring::{score_hand as scorehand, Score},
    select_cards::{get_next_counted_card, select_crib_cards},
};
use rand::prelude::{Rng, SliceRandom};

///
/// given the HttpRequest returns the hostname in the form of localhost:8080/api
macro_rules! get_hostname {
    ($req:expr) => {
        format!("{}/api", $req.app_config().host());
    };
}

/// cut the cards to see who goes first
///
///  sample URLs:
///              http:///localhost:8080/api/cutcards
///
///  returns: the two cut cards and the repeat URL.  the client is written to assume a shared notion of the deck
///           so we just return 2 numbers bewtween 0 and 51
pub async fn cut_cards(req: HttpRequest) -> impl Responder {
    let mut rng = rand::thread_rng();
    let first = rng.gen_range(0..51) as usize;
    let mut second = rng.gen_range(0..51) as usize;
    while first % 13 == second % 13 {
        // % 13 gives Rank and we can't have the rank the same, as it'd be a tie and we'd just draw again
        second = rng.gen_range(0..51) as usize;
    }

    let _cc: CutCards = match CutCards::new(first, second) {
        Ok(cc) => {
            let response = CutCardResponse {
                CutCards: cc,
                RepeatUrl: format!("{}/cutcards/{},{}", get_hostname!(req), first, second),
            };
            return HttpResponse::Ok().body(serde_json::to_string(&response).unwrap());
        }
        Err(e) => {
            return HttpResponse::BadRequest().body(serde_json::to_string(&e).unwrap());
        }
    };
}

/// cut the cards to see who goes first - pass in the random numbers that you get the same result
/// the last time the api was called. useful for testing.
///
/// sample URLs:
///              http://localhost:8080/api/cutcards/1,8
///
/// returns: the two cut cards
///
pub async fn cut_cards_repeat(req: HttpRequest, cards: Path<String>) -> impl Responder {
    let cards = cards.into_inner();
    let tokens: Vec<&str> = cards.split(",").collect();
    if tokens.len() != 2 {
        return HttpResponse::BadRequest()
            .body("there should be two cards seperated by a ',' such as '1,2'");
    }
    let pair = match (tokens[0].parse::<usize>(), tokens[1].parse::<usize>()) {
        (Ok(first), Ok(second)) => (first, second),
        _ => {
            return HttpResponse::BadRequest()
                .body("there should be two numbers seperated by a ',' such as '1,2'");
        }
    };
    match CutCards::new(pair.0, pair.1) {
        Ok(cc) => {
            let response = CutCardResponse {
                CutCards: cc,
                RepeatUrl: format!("{}/cutcards/{},{}", get_hostname!(req), pair.0, pair.1),
            };
            HttpResponse::Ok().body(serde_json::to_string(&response).unwrap())
        }
        Err(e) => HttpResponse::BadRequest().body(serde_json::to_string(&e).unwrap()),
    }
}

///  score the hand (or crib)
///
///  sample URLs:
///              localhost:8088/api/scorehand/FiveOfHearts,FiveOfClubs,FiveOfSpades,JackOfDiamonds/FourOfDiamonds/false
///              localhost:8088/api/scorehand/FiveOfHearts,SixOfHearts,SevenOfHearts,EightOfHearts/NineOfDiamonds/false  (should be a flush)
///              localhost:8088/api/scorehand/FiveOfHearts,SixOfHearts,SevenOfHearts,EightOfHearts/NineOfDiamonds/true   (no flush - need 5 of same suit in crib)
///              localhost:8088/api/scorehand/FiveOfHearts,SixOfHearts,SevenOfHearts,EightOfHearts/NineOfHearts/true     (should be a flush)
///              localhost:8088/api/scorehand/FiveOfHearts,SixOfHearts,FourOfHearts,FourOFClubs/SixOfDiamonds/true     (bad card)
///              localhost:8088/api/scorehand/FiveOfHearts,SixOfHearts,FourOfHearts,FourOfClubs/SixOfDiamonds/true     (double double run with 15s - 24 points)
///              localhost:8088/api/scorehand/ThreeOfSpades,TwoOfSpades,QueenOfHearts,QueenOfClubs/AceOfHearts/false
///
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

    let score_response: ScoreResponse = ScoreResponse::from_score(score);

    HttpResponse::Ok().body(serde_json::to_string(&score_response).unwrap())
}

///  given 6 cards, return 2.  if isMyCrib is true, then optimize to make the hand + crib have the most points possible
///
///  sample URLs:
///   localhost:8088/api/getcribcards/FiveOfHearts,FiveOfClubs,FiveOfSpades,JackOfDiamonds,SixOfClubs,FourOfDiamonds/false
///   localhost:8088/api/getcribcards/FiveOfHearts,FiveOfClubs,FiveOfSpades,JackOfDiamonds,SixOfClubs,FourOfDiamonds/true
///   localhost:8088/api/getcribcards/FourOfHearts,FiveOfHearts,SixOfSpades,JackOfHearts,QueenOfHearts,SixOfDiamonds/true
///   localhost:8088/api/getcribcards/FourOfHearts,FiveOfHearts,SixOfSpades,JackOfHearts,QueenOfHearts,SixOfDiamonds/false
///
///
pub async fn get_crib(path: Path<(String, bool)>) -> impl Responder {
    let path = path.into_inner();

    let parsed_hand = match ParsedHand::from_string(path.0) {
        Ok(parsed_hand) => parsed_hand,
        Err(e) => {
            return HttpResponse::BadRequest().body(serde_json::to_string(&e).unwrap());
        }
    };

    let crib = match select_crib_cards(parsed_hand.hand.as_slice(), path.1) {
        Ok(crib) => crib,
        Err(e) => {
            return HttpResponse::BadRequest().body(serde_json::to_string(&e).unwrap());
        }
    };

    let result: [ClientCard; 2] = [
        ClientCard::from_card(crib[0], "unknown".to_string()),
        ClientCard::from_card(crib[1], "unknown".to_string()),
    ];

    HttpResponse::Ok().body(serde_json::to_string(&result).unwrap())
}

///  URL example:
///          localhost:8088/api/getnextcountedcard/AceOfSpades,AceOfHearts,TwoOfClubs,TenOfDiamonds/0
///          localhost:8088/api/getnextcountedcard/FiveOfClubs,QueenOfDiamonds/25/ThreeOfDiamonds,TenOfClubs,TwoOfSpades,QueenOfSpades
///          localhost:8088/api/getnextcountedcard/SixOfClubs,QueenOfDiamonds/25/ThreeOfDiamonds,TenOfClubs,TwoOfSpades,QueenOfSpades
///          localhost:8088/api/getnextcountedcard/SixOfClubs,QueenOfDiamonds/25/ThreeOfDiamonds,TenOfClubs,TwoOfSpades,QueenOfSpades
///
///  Note that the last parameters contains all the cards that have already been counted, which means it starts empty, so there are two routes.
///
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

/// returns the next counted card - distinct from the first counted card
/// in that it also gets the CSV list of cards played
///
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

/// helper function that gets the counted card and then formats the proper response
///
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
                        countedCard: Some(ClientCard::from_card(card, "unknown".to_string())),
                        Scoring: ScoreResponse::from_score(score),
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

/// this gets routed when the URL does not have any cards that have already been played
/// there are never any points scored on the first card.
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

/// routed to when the player plays a card and there are already some cards played.
///
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
            let score_response: ScoreResponse = ScoreResponse::from_score(score);

            return HttpResponse::Ok().body(serde_json::to_string(&score_response).unwrap());
        }

        Err(e) => {
            return HttpResponse::BadRequest().body(serde_json::to_string(&e).unwrap());
        }
    };
}

/// helper function for getting a random hand
///
fn get_random_hand_internal(
    req: HttpRequest,
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

    let crib_cards = select_crib_cards(computer_hand.as_slice(), true).unwrap();
    for card in crib_cards.iter() {
        response
            .ComputerCribCards
            .push(ClientCard::from_card(*card, "computer".to_string()));
    }

    response.RepeatUrl = format!(
        "{}/getrandomhand/{}/{}/{}",
        get_hostname!(req),
        is_computer_crib,
        indices,
        cards[12]
    );
    Some(response)
}

/// routed to when a new hand is needed.
/// only returns the 13 cards (6 for computer, 6 for player, 1 shared)
/// also returns the cards that the computer should give to the crib
///
/// sample url: http://localhost:8080/api/getrandomhand/true
///
pub async fn get_random_hand(req: HttpRequest, path: Path<bool>) -> impl Responder {
    let is_computer_crib = path.into_inner();

    let mut rng = rand::thread_rng();
    let mut deck = (0..51).collect::<Vec<_>>();
    deck.shuffle(&mut rng);

    let response = match get_random_hand_internal(req, is_computer_crib, deck) {
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

/// useful for debugging the client - this will give the same hand that was returned from get_random_hand
///
/// sample url: localhost:8080/api/getrandomhand/true/21,49,41,15,46,17,34,24,22,38,19,31/20
pub async fn get_random_hand_repeat(
    req: HttpRequest,
    path: Path<(bool, String, String)>,
) -> impl Responder {
    let path = path.into_inner();
    let is_computer_crib = path.0;

    let shared_card_index = match path.2.parse::<usize>() {
        Ok(shared_card_index) => shared_card_index,
        Err(e) => {
            let msg = format!(
                "unable to parse {} into usize.  Bad shared card index.\nparse error: {:?}",
                path.2, e
            );
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
    let response = match get_random_hand_internal(req, is_computer_crib, indices) {
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

///  Tests for the Web API.  The actual logic of the game is already tested in the unit tests for that part of the project
///
///  these tests test the "shape" of the Web API - making sure that serialization/deserialization works (by simply using it
///  in the tests) and that the response we get back are non-empty or have other reasonable values.
///
//  reproducibility isn't something the game uses, but is verified here to enforce the correctness of the repeat URL
///
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{game_handlers, safe_set_port, PORT};
    use actix_web::{test, web, App};
    use cribbage_library::scoring::CombinationName;
    use std::env;

    macro_rules! get_repeat_url {
        ($url: expr) => {{
            let index = $url.find("/").unwrap() + 1;
            let repeat_url = $url
                .chars()
                .skip(index - 1) // we want the '/' to start the string
                .take($url.len() - index + 1)
                .collect::<String>();

            println!("repeat url: {}", repeat_url);
            repeat_url
        }};
    }

    macro_rules! vec_contains_card {
        ($vec: expr, $card: expr) => {{
            let mut found = false;
            for c in $vec.iter() {
                if c.cardName == $card {
                    found = true;
                    break;
                }
            }
            found
        }};
    }
    #[allow(unused_macros)]
    macro_rules! vec_contains_score {
        ($vec: expr, $name: expr) => {{
            let mut found = false;
            for score in $vec.iter() {
                if score.ScoreName == $name {
                    found = true;
                    break;
                }
            }
            found
        }};
    }

    /**
     *  this macro takes in the service to call, the orignal response, and the repeat URL
     *  it then calls the service again with the repeat URL and verifies that the return
     *  values are identical.
     */
    macro_rules! test_repeatability {
        ($service:expr, $original_response:expr, $un_parsed_repeat_url:expr) => {{
            let repeat_url = get_repeat_url!($un_parsed_repeat_url);
            let req = test::TestRequest::get().uri(&repeat_url).to_request();
            let response = test::read_response(&mut $service, req).await;
            let repeat_json = response;
            let json_original = serde_json::to_string(&$original_response).unwrap();
            assert_eq!(actix_web::web::Bytes::from(repeat_json), json_original);
        }};
    }

    ///
    ///  this test gets the cut cards and the parses out the repeat URL to call cutcards again
    ///  it verifies that it gets the same results back.
    ///
    ///  this also verifies that the json serialize/deserialize of the CutCardResponse works correctly
    ///
    #[actix_rt::test]
    async fn cut_cards() {
        //
        //  main() is not called for these tests, so we have to set the port and hostname here.
        safe_set_port!();

        //
        //  a hard won learning: the route below does *not* start with a "/"
        //  but the URI to call it must start with a "/"
        let mut app = test::init_service(
            App::new()
                .route("api/cutcards", web::get().to(game_handlers::cut_cards))
                .route(
                    "api/cutcards/{cards}",
                    web::get().to(game_handlers::cut_cards_repeat),
                ),
        )
        .await;
        //
        //  this is the URI that has to start with a "/"
        let req = test::TestRequest::get().uri("/api/cutcards").to_request();
        let ccr: CutCardResponse = test::read_response_json(&mut app, req).await;

        test_repeatability!(app, ccr, ccr.RepeatUrl);

        //  make sure that we are actually getting data back
        assert_ne!(ccr.RepeatUrl, "");
        assert_ne!(ccr.CutCards.Computer.cardName, "");
        assert_ne!(ccr.CutCards.Player.cardName, "");
    }

    #[actix_rt::test]
    async fn test_random_hand() {
        safe_set_port!();
        let mut app = test::init_service(
            App::new()
                .route(
                    "api/getrandomhand/{is_computer_crib}",
                    web::get().to(game_handlers::get_random_hand),
                )
                .route(
                    "api/getrandomhand/{is_computer_crib}/{indices}/{shared_index}",
                    web::get().to(game_handlers::get_random_hand_repeat),
                ),
        )
        .await;
        let req = test::TestRequest::get()
            .uri("/api/getrandomhand/true")
            .to_request();
        let rhr: RandomHandResponse = test::read_response_json(&mut app, req).await;

        test_repeatability!(app, rhr, rhr.RepeatUrl);

        assert_ne!(rhr.SharedCard.cardName, "");
        assert_eq!(rhr.RandomCards.len(), 13);
        assert_eq!(rhr.ComputerCribCards.len(), 2);
    }

    #[actix_rt::test]
    async fn test_score_counted_cards() {
        safe_set_port!();
        let mut app = test::init_service(
            App::new()
                .route(
                    "api/scorecountedcards/{played_card}/{total_count}/",
                    web::get().to(game_handlers::score_first_counted_card),
                )
                .route(
                    "api/scorecountedcards/{played_card}/{total_count}/{counted_cards}",
                    web::get().to(game_handlers::score_counted_cards),
                ),
        )
        .await;
        let uri = "/api/scorecountedcards/AceOfSpades/0/";
        let req = test::TestRequest::get().uri(uri).to_request();

        let score_response: ScoreResponse = test::read_response_json(&mut app, req).await;
        assert_eq!(score_response.Score, 0);
        assert_eq!(score_response.ScoreInfo.len(), 0);

        let uri = "/api/scorecountedcards/TwoOfClubs/13/AceOfHearts,ThreeOfClubs,FiveOfDiamonds,FourOfClubs";
        let req = test::TestRequest::get().uri(uri).to_request();
        let score_response: ScoreResponse = test::read_response_json(&mut app, req).await;
        assert_eq!(score_response.Score, 7);
        assert_eq!(score_response.ScoreInfo.len(), 2);
        assert_eq!(
            vec_contains_score!(score_response.ScoreInfo, CombinationName::Fifteen),
            true
        );
        assert_eq!(
            vec_contains_score!(score_response.ScoreInfo, CombinationName::RunOfFive),
            true
        );
    }

    #[actix_rt::test]
    async fn test_score_hand() {
        safe_set_port!();
        let mut app = test::init_service(App::new().route(
            "api/scorehand/{hand}/{shared_card}/{is_crib}",
            web::get().to(game_handlers::score_hand),
        ))
        .await;
        let uri = "/api/scorehand/FiveOfHearts,FiveOfClubs,FiveOfSpades,JackOfDiamonds/FourOfDiamonds/false";
        let req = test::TestRequest::get().uri(uri).to_request();

        let score_response: ScoreResponse = test::read_response_json(&mut app, req).await;
        assert_eq!(score_response.Score, 15);
        assert_eq!(score_response.ScoreInfo.len(), 6);
        assert_eq!(
            vec_contains_score!(score_response.ScoreInfo, CombinationName::Nob),
            true
        );
        assert_eq!(
            vec_contains_score!(score_response.ScoreInfo, CombinationName::Fifteen),
            true
        );
        assert_eq!(
            vec_contains_score!(score_response.ScoreInfo, CombinationName::RoyalPair),
            true
        );
    }

    #[actix_rt::test]
    async fn test_get_crib_hand() {
        safe_set_port!();
        let mut app = test::init_service(App::new().route(
            "api/getcribcards/{hand}/{my_crib}",
            web::get().to(game_handlers::get_crib),
        ))
        .await;
        let uri = "/api/getcribcards/FiveOfHearts,FiveOfClubs,FiveOfSpades,JackOfDiamonds,SixOfClubs,FourOfDiamonds/false";
        let req = test::TestRequest::get().uri(uri).to_request();

        let response: Vec<ClientCard> = test::read_response_json(&mut app, req).await;
        assert_eq!(response.len(), 2);
        assert_eq!(vec_contains_card!(response, "SixOfClubs"), true);
        assert_eq!(vec_contains_card!(response, "FourOfDiamonds"), true);

        let uri = "/api/getcribcards/FiveOfHearts,FiveOfClubs,FiveOfSpades,JackOfDiamonds,SixOfClubs,FourOfDiamonds/true";
        let req = test::TestRequest::get().uri(uri).to_request();
        let response: Vec<ClientCard> = test::read_response_json(&mut app, req).await;
        assert_eq!(response.len(), 2);
        assert_eq!(vec_contains_card!(response, "FiveOfSpades"), true);
        assert_eq!(vec_contains_card!(response, "JackOfDiamonds"), true);
    }

    #[actix_rt::test]
    async fn test_get_next_counted_card() {
        safe_set_port!();
        let mut app = test::init_service(
            App::new()
                .route(
                    "api/getnextcountedcard/{available_cards}/{total_count}/",
                    web::get().to(game_handlers::get_first_counted_card),
                )
                .route(
                    "api/getnextcountedcard/{available_cards}/{total_count}/{cards_played}",
                    web::get().to(game_handlers::next_counted_card),
                ),
        )
        .await;
        let uri = "/api/getnextcountedcard/AceOfSpades,AceOfHearts,TwoOfClubs,TenOfDiamonds/0/";
        let req = test::TestRequest::get().uri(uri).to_request();

        let response: CountedCardResponse = test::read_response_json(&mut app, req).await;
        assert_eq!(response.countedCard.unwrap().cardName, "AceOfSpades");
        assert_eq!(response.Scoring.Score, 0);
        assert_eq!(response.Scoring.ScoreInfo.len(), 0);

        let uri = "/api/getnextcountedcard/TenOfClubs,AceOfHearts/16/AceOfSpades,ThreeOfClubs,TwoOfClubs,TenOfHearts";
        let req = test::TestRequest::get().uri(uri).to_request();
        let response: CountedCardResponse = test::read_response_json(&mut app, req).await;
        assert_eq!(response.countedCard.unwrap().cardName, "TenOfClubs");
        assert_eq!(response.Scoring.Score, 2);
        assert_eq!(response.Scoring.ScoreInfo.len(), 1);
        assert_eq!(
            vec_contains_score!(response.Scoring.ScoreInfo, CombinationName::Pair),
            true
        );

        let uri =
            "/api/getnextcountedcard/ThreeOfClubs,TwoOfClubs/30/TenOfClubs,TenOfHearts,TenOfSpades";
        let req = test::TestRequest::get().uri(uri).to_request();
        let response: CountedCardResponse = test::read_response_json(&mut app, req).await;
        match response.countedCard {
            Some(_) => {
                assert_eq!(true, false, "there should be no card here!")
            }
            None => {
                // test passes
            }
        };

        assert_eq!(response.Scoring.Score, 0);
        assert_eq!(response.Scoring.ScoreInfo.len(), 0);
    }
}
