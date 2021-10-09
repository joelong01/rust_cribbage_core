mod client_structs;
mod game_handlers;
mod handlers;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};

static PORT:u32 = 8080;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Start http server

    HttpServer::new(|| {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
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
            .service(
                web::scope("/api/")
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
                        web::resource("getnextcountedcard/{available_cards}/{total_count}/") // not trailing '/' as that is what the client uses
                            .route(web::get().to(game_handlers::get_first_counted_card)),
                    )
                    .service(
                        web::resource(
                            "getnextcountedcard/{available_cards}/{total_count}/{cards_played}",
                        )
                        .route(web::get().to(game_handlers::next_counted_card)),
                    )
                    .service(
                        web::resource("scorecountedcards/{available_cards}/{total_count}/")
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
    })
    .bind(format!("localhost:{}", PORT))? // TODO pull address:port from config
    .run()
    .await
}

#[cfg(test)]
mod tests {}
