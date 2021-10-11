/// this is the main entry point for the REST API.  This project is a wire-compatible replacement for the REST API found at
/// https://github.com/joelong01/CribbageJS , including all of its mistakes and foibles such as non-versioned API.  The goal
/// is to make the client at https://github.com/joelong01/CribbageUi.Js run unmodified.
///
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
mod client_structs;
mod game_handlers;
mod handlers;
use once_cell::sync::OnceCell;
use std::env;

///
/// the client expects 8080, but this is the one thing we change on the client
/// see the serviceProxy.js file where HOST_NAME is defined
///
static PORT: OnceCell<String> = OnceCell::new();
pub static HOST_NAME: OnceCell<String> = OnceCell::new();

#[allow(unused_macros)]
#[macro_export]
macro_rules! safe_set_port {
                () => {{
                    let port: String;
                    match PORT.get() {
                        Some(val) => {port = val.to_string();}
                        None => {

                            match env::var("CRIBBAGE_PORT") {
                                Ok(val) => port = val.to_string(),
                                Err(_e) => port = "8080".to_string(),
                            }
                            println!("setting port to: {}", port);
                            match PORT.set(port.clone()) {
                                Ok(_) => {}
                                Err(e) => {
                                    println!("error setting port: {:?}", e.to_string());
                                }
                            }
                            let host: String = format!("localhost:{}/api", port); // TODO:  this should be picked out of the
                            match HOST_NAME.set(host.clone()) {
                                Ok(_) => {}
                                Err(e) => {
                                    println!("error setting host: {:?}", e);
                                }
                            }

                            println!("setting host to {}", host.clone());
                        }


                    };
                    port
                }};
            }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port: String = safe_set_port!();
    HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive())
            .service(
                web::scope("/api/") // normally this would have a version number in it, but the JS implementation does not have it.
                    .service(
                        web::resource("cutcards").route(web::get().to(game_handlers::cut_cards)),
                    )
                    .service(
                        web::resource("cutcards/{cards}")
                            .route(web::get().to(game_handlers::cut_cards_repeat)),
                    )
                    .service(
                        web::resource("scorehand/{hand}/{shared_card}/{is_crib}")
                            .route(web::get().to(game_handlers::score_hand)),
                    )
                    .service(
                        web::resource("getcribcards/{hand}/{my_crib}")
                            .route(web::get().to(game_handlers::get_crib)),
                    )
                    .service(
                        // note trailing '/' as that is what the client uses
                        web::resource("getnextcountedcard/{available_cards}/{total_count}/")
                            .route(web::get().to(game_handlers::get_first_counted_card)),
                    )
                    .service(
                        web::resource(
                            "getnextcountedcard/{available_cards}/{total_count}/{cards_played}",
                        )
                        .route(web::get().to(game_handlers::next_counted_card)),
                    )
                    .service(
                        //
                        //  another trailing /
                        web::resource("scorecountedcards/{played_card}/{total_count}/")
                            .route(web::get().to(game_handlers::score_first_counted_card)),
                    )
                    .service(
                        web::resource(
                            "scorecountedcards/{played_card}/{total_count}/{counted_cards}",
                        )
                        .route(web::get().to(game_handlers::score_counted_cards)),
                    )
                    .service(
                        web::resource("getrandomhand/{is_computer_crib}")
                            .route(web::get().to(game_handlers::get_random_hand)),
                    )
                    .service(
                        web::resource("getrandomhand/{is_computer_crib}/{indices}/{shared_index}")
                            .route(web::get().to(game_handlers::get_random_hand_repeat)),
                    ),
            )
            .service(
                //
                //  these are still experimental as I try to figure out how to post to cosmos...
                web::scope("registeredai")
                    .service(web::resource("/").route(web::get().to(handlers::get_registered_ais)))
                    .service(
                        web::resource("ai/{name}/{author}/{description}/{uri}")
                            .route(web::post().to(handlers::add_ai)),
                    )
                    .service(
                        web::resource("testcosmos").route(web::get().to(handlers::test_cosmos)),
                    ),
            )
    })
    .bind(format!("localhost:{}", port))?
    .run()
    .await
}
