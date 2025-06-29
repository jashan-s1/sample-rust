use actix_web::{get, App, HttpServer, Responder, HttpResponse};
use std::env;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello from Actix-Web on Render!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Get the PORT from environment or default to 8080
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    println!("🚀 Server running on {}", addr);

    HttpServer::new(|| {
        App::new().service(hello)
    })
    .bind(addr)?
    .run()
    .await
}
