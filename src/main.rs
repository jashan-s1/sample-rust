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
use solana_sdk::system_instruction;
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

#[derive(Deserialize)]
struct MintTokenRequest {
    mint: String,
    destination: String,
    authority: String,
    amount: u64,
}

#[post("/token/mint")]
async fn mint_token(req: web::Json<MintTokenRequest>) -> impl Responder {
    let mint = match Pubkey::from_str(&req.mint) {
        Ok(pk) => pk,
        Err(_) => return error("Invalid mint pubkey"),
    };

    let destination = match Pubkey::from_str(&req.destination) {
        Ok(pk) => pk,
        Err(_) => return error("Invalid destination pubkey"),
    };

    let authority = match Pubkey::from_str(&req.authority) {
        Ok(pk) => pk,
        Err(_) => return error("Invalid authority pubkey"),
    };

    let instruction = match spl_token::instruction::mint_to(
        &token_program_id(),
        &mint,
        &destination,
        &authority,
        &[], // multisig signers if any
        req.amount,
    ) {
        Ok(instr) => instr,
        Err(e) => return error(&format!("Failed to create mint instruction: {}", e)),
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
        .collect::<Vec<_>>();

    let response = CreateTokenResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: serialized_data,
    };

    success(response)
}

#[derive(Deserialize)]
struct SignMessageRequest {
    message: String,
    secret: String,
}

#[derive(Serialize)]
struct SignMessageResponse {
    signature: String,
    public_key: String,
    message: String,
}

#[post("/message/sign")]
async fn sign_message(req: web::Json<SignMessageRequest>) -> impl Responder {
    if req.message.is_empty() || req.secret.is_empty() {
        return error("Missing required fields");
    }

    let secret_bytes = match bs58::decode(&req.secret).into_vec() {
        Ok(bytes) => bytes,
        Err(_) => return error("Invalid base58 secret key"),
    };

    if secret_bytes.len() != 64 {
        return error("Secret key must be 64 bytes long");
    }

    let keypair = match Keypair::from_bytes(&secret_bytes) {
        Ok(kp) => kp,
        Err(_) => return error("Invalid keypair bytes"),
    };

    let signature = keypair.sign_message(req.message.as_bytes());

    success(SignMessageResponse {
        signature: base64::encode(signature.as_ref()),
        public_key: keypair.pubkey().to_string(),
        message: req.message.clone(),
    })
}

#[derive(Deserialize)]
struct VerifyMessageRequest {
    message: String,
    signature: String,
    pubkey: String,
}

#[derive(Serialize)]
struct VerifyMessageResponse {
    valid: bool,
    message: String,
    pubkey: String,
}

#[post("/message/verify")]
async fn verify_message(req: web::Json<VerifyMessageRequest>) -> impl Responder {
    use solana_sdk::signature::Signature;
    use solana_sdk::pubkey::Pubkey;

    if req.message.is_empty() || req.signature.is_empty() || req.pubkey.is_empty() {
        return error("Missing required fields");
    }

    let pubkey = match Pubkey::from_str(&req.pubkey) {
        Ok(pk) => pk,
        Err(_) => return error("Invalid public key"),
    };

    let signature_bytes = match base64::decode(&req.signature) {
        Ok(bytes) => bytes,
        Err(_) => return error("Invalid base64 signature"),
    };

    let signature = match Signature::try_from(signature_bytes.as_slice()) {
        Ok(sig) => sig,
        Err(_) => return error("Invalid signature format"),
    };

    let is_valid = signature.verify(pubkey.as_ref(), req.message.as_bytes());

    success(VerifyMessageResponse {
        valid: is_valid,
        message: req.message.clone(),
        pubkey: req.pubkey.clone(),
    })
}

#[derive(Deserialize)]
struct SendSolRequest {
    from: String,
    to: String,
    lamports: u64,
}

#[derive(Serialize)]
struct SendSolResponse {
    program_id: String,
    accounts: Vec<String>,
    instruction_data: String,
}

#[post("/send/sol")]
async fn send_sol(req: web::Json<SendSolRequest>) -> impl Responder {
    let from = match Pubkey::from_str(&req.from) {
        Ok(pk) => pk,
        Err(_) => return error("Invalid sender address"),
    };

    let to = match Pubkey::from_str(&req.to) {
        Ok(pk) => pk,
        Err(_) => return error("Invalid recipient address"),
    };

    if req.lamports == 0 {
        return error("Lamports must be greater than 0");
    }

    // Create system transfer instruction
    let instruction = system_instruction::transfer(&from, &to, req.lamports);

    let serialized = base64::encode(instruction.data.clone());

    success(SendSolResponse {
        program_id: instruction.program_id.to_string(),
        accounts: instruction
            .accounts
            .into_iter()
            .map(|meta| meta.pubkey.to_string())
            .collect(),
        instruction_data: serialized,
    })
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
            .service(mint_token)
            .service(sign_message)
            .service(verify_message)
            .service(send_sol)
    })
    .bind(addr)?
    .run()
    .await
}
