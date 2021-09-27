use actix_web::{HttpResponse, Responder};

pub async fn game() -> impl Responder {
    HttpResponse::Ok().body("Welcome to Cribbage Rust!!!")
}
