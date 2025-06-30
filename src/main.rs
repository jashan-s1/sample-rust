use actix_web::{get, post, web, App, HttpServer, HttpResponse, Responder};
use serde::Serialize;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use std::env;
use bs58;

// === Response Wrappers ===

#[derive(Serialize)]
struct SuccessResponse<T: Serialize> {
    success: bool,
    data: T,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
}

fn success<T: Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok().json(SuccessResponse { success: true, data })
}

fn error(message: &str) -> HttpResponse {
    HttpResponse::BadRequest().json(ErrorResponse {
        success: false,
        error: message.to_string(),
    })
}

// === Default Route ===

#[get("/")]
async fn hello() -> impl Responder {
    success("Hello from Actix on Railway!")
}

// === /keypair Endpoint ===

#[derive(Serialize)]
struct KeypairResponse {
    pubkey: String,
    secret: String,
}

#[post("/keypair")]
async fn generate_keypair() -> impl Responder {
    let keypair = Keypair::new();

    let pubkey = keypair.pubkey().to_string(); // base58
    let secret = bs58::encode(keypair.to_bytes()).into_string();

    success(KeypairResponse { pubkey, secret })
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8888".to_string());
    let addr = format!("0.0.0.0:{}", port);
    println!("🚀 Server running at http://{}", addr);

    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(generate_keypair)
    })
    .bind(addr)?
    .run()
    .await
}
