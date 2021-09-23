use actix_web::Responder;

pub async fn game() -> impl Responder {
    format!("Welcome to Cribbage Rust!!!")
}