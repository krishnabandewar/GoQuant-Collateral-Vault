use actix_web::{get, post, web, HttpResponse, HttpRequest, Responder};
use actix_web_actors::ws;
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use crate::db;
use crate::websocket::{self, VaultWebSocket, Broadcaster, WsMessage};
use actix::Addr;

#[derive(Serialize, Deserialize)]
pub struct CreateVaultRequest {
    pub owner: String,
    pub pubkey: String,
}

#[derive(Serialize, Deserialize)]
pub struct TransactionRequest {
    pub vault_pubkey: String,
    pub amount: i64,
    pub signature: String, // Transaction signature
}

#[derive(Serialize)]
pub struct TvlResponse {
    pub total_value_locked: i64,
}

#[get("/health")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Vault Manager System Operational")
}

#[post("/vaults")]
pub async fn create_vault(pool: web::Data<PgPool>, req: web::Json<CreateVaultRequest>) -> impl Responder {
    match db::insert_vault(&pool, &req.owner, &req.pubkey).await {
        Ok(_) => HttpResponse::Created().json(req.into_inner()),
        Err(e) => {
            eprintln!("Error creating vault: {:?}", e);
            HttpResponse::InternalServerError().finish()
        },
    }
}

#[get("/vaults/{pubkey}")]
pub async fn get_vault(pool: web::Data<PgPool>, pubkey: web::Path<String>) -> impl Responder {
    match db::find_vault(&pool, &pubkey).await {
        Ok(vault) => HttpResponse::Ok().json(vault),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

// Deposit logic with WebSocket Broadcast
#[post("/vaults/deposit")]
pub async fn deposit(
    pool: web::Data<PgPool>, 
    broadcaster: web::Data<Addr<Broadcaster>>, 
    req: web::Json<TransactionRequest>
) -> impl Responder {
    // 1. Update Balance
    if let Err(_) = db::update_balance(&pool, &req.vault_pubkey, req.amount).await {
        return HttpResponse::InternalServerError().finish();
    }
    // 2. Log Transaction
    if let Err(_) = db::insert_transaction(&pool, &req.vault_pubkey, "deposit", req.amount, &req.signature).await {
        return HttpResponse::InternalServerError().finish();
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    // 3. Broadcast Update
    broadcaster.do_send(WsMessage::Deposit {
        vault_pubkey: req.vault_pubkey.clone(),
        amount: req.amount,
        signature: req.signature.clone(),
        timestamp,
    });

    // Also send pure balance update?? Ideally yes, or client derives it. Let's send a TVL update too?
    // For simplicity, just the event.

    HttpResponse::Ok().json(serde_json::json!({"status": "deposited", "amount": req.amount}))
}

// Withdraw logic with WebSocket Broadcast
#[post("/vaults/withdraw")]
pub async fn withdraw(
    pool: web::Data<PgPool>,
    broadcaster: web::Data<Addr<Broadcaster>>,
    req: web::Json<TransactionRequest>
) -> impl Responder {
     // 1. Update Balance
     if let Err(_) = db::update_balance(&pool, &req.vault_pubkey, -req.amount).await {
        return HttpResponse::InternalServerError().finish();
    }
    // 2. Log Transaction
    if let Err(_) = db::insert_transaction(&pool, &req.vault_pubkey, "withdrawal", req.amount, &req.signature).await {
        return HttpResponse::InternalServerError().finish();
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // 3. Broadcast Update
    broadcaster.do_send(WsMessage::Withdrawal {
        vault_pubkey: req.vault_pubkey.clone(),
        amount: req.amount,
        signature: req.signature.clone(),
        timestamp,
    });

    HttpResponse::Ok().json(serde_json::json!({"status": "withdrawn", "amount": req.amount}))
}

// Get Vault Transactions
#[get("/vaults/{pubkey}/transactions")]
pub async fn get_vault_transactions(pool: web::Data<PgPool>, pubkey: web::Path<String>) -> impl Responder {
    match db::get_transactions(&pool, &pubkey).await {
         Ok(txs) => HttpResponse::Ok().json(txs),
         Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// TVL Endpoint
#[get("/tvl")]
pub async fn get_tvl(pool: web::Data<PgPool>) -> impl Responder {
    match db::get_total_tvl(&pool).await {
        Ok(tvl) => HttpResponse::Ok().json(TvlResponse { total_value_locked: tvl }),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// WebSocket endpoint
pub async fn websocket_handler(
    req: HttpRequest, 
    stream: web::Payload,
    broadcaster: web::Data<Addr<Broadcaster>>
) -> Result<HttpResponse, actix_web::Error> {
    ws::start(VaultWebSocket::new(broadcaster.get_ref().clone()), &req, stream)
}
