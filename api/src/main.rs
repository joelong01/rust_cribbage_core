mod handlers;

use actix_web::{web, App, HttpServer};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // Start http server
    HttpServer::new(|| App::new().route("v1.0/game", web::get().to(handlers::game)))
        .bind("127.0.0.1:8080")? // TODO pull address:port from config
        .run()
        .await
}
