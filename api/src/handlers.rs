use actix_web::{
    error::ResponseError, get, http::StatusCode, post, web, HttpResponse, Responder, Result,
};
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize)]
struct AiInfo {
    host_name: String,
    friendly_name: String,
}

#[derive(Error, Debug)]
enum RegistrationError {
    #[error("Bad config")]
    BadConfig,
    #[error("Access to Cosmos failed")]
    Forbidden,
}

pub async fn game() -> impl Responder {
    HttpResponse::Ok().body("Welcome to Cribbage Rust!!!")
}

#[get("/availableAis")]
async fn get_available_ais() -> impl Responder {
    let account_name;
    match std::env::var("RUST_CRIBBAGE_COSMOS_ACCOUNT_NAME") {
        Ok(val) => account_name = val,
        Err(_e) => {}
    }
    HttpResponse::Ok().body("Hello world!")
}

#[post("/registerAi/{host_name}/{friendly_name}")]
async fn register_ai(path: web::Path<AiInfo>) -> Result<String> {
    Ok(format!(
        "registered {}/{}",
        path.host_name, path.friendly_name
    ))
}
