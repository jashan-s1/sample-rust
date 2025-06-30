use actix_web::{get, post, web, App, HttpServer, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use std::{env, str::FromStr};
use bs58;
use base64;
use spl_token::{instruction::initialize_mint, id as token_program_id};

// ======= RESPONSE WRAPPERS =======

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

// ======= GET / =======

#[get("/")]
async fn hello() -> impl Responder {
    success("Hi Rusty")
}

// ======= POST /keypair =======

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

// ======= POST /token/create =======

#[derive(Deserialize)]
struct CreateTokenRequest {
    mintAuthority: String,
    mint: String,
    decimals: u8,
}

#[derive(Serialize)]
struct AccountMetaResponse {
    pubkey: String,
    is_signer: bool,
    is_writable: bool,
}

#[derive(Serialize)]
struct CreateTokenResponse {
    program_id: String,
    accounts: Vec<AccountMetaResponse>,
    instruction_data: String,
}

#[post("/token/create")]
async fn create_token(req: web::Json<CreateTokenRequest>) -> impl Responder {
    let mint = match Pubkey::from_str(&req.mint) {
        Ok(pk) => pk,
        Err(_) => return error("Invalid mint pubkey"),
    };

    let mint_authority = match Pubkey::from_str(&req.mintAuthority) {
        Ok(pk) => pk,
        Err(_) => return error("Invalid mint authority pubkey"),
    };

    let instruction = match initialize_mint(
        &token_program_id(),
        &mint,
        &mint_authority,
        None,
        req.decimals,
    ) {
        Ok(instr) => instr,
        Err(e) => return error(&format!("Failed to build instruction: {}", e)),
    };

    let serialized_data = base64::encode(&instruction.data);
    let accounts = instruction
        .accounts
        .into_iter()
        .map(|meta| AccountMetaResponse {
            pubkey: meta.pubkey.to_string(),
            is_signer: meta.is_signer,
            is_writable: meta.is_writable,
        })
        .collect();

    let response = CreateTokenResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: serialized_data,
    };

    success(response)
}

// ======= MAIN =======

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8888".to_string());
    let addr = format!("0.0.0.0:{}", port);

    println!("🚀 Server running at http://{}", addr);

    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(generate_keypair)
            .service(create_token)
    })
    .bind(addr)?
    .run()
    .await
}
