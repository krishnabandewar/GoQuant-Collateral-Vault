use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use sqlx::postgres::PgPool;
use std::env;

mod api;
mod db;
mod websocket;  // WebSocket enabled
// mod solana;

use actix::Actor;
use websocket::Broadcaster;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:krishna1310@localhost/goquant_vaults".to_string());

    let pool = PgPool::connect(&database_url).await.expect("Failed to connect to DB");
    
    // Run migrations
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS vaults (
            id SERIAL PRIMARY KEY,
            owner TEXT NOT NULL,
            pubkey TEXT NOT NULL UNIQUE,
            balance BIGINT DEFAULT 0,
            status TEXT DEFAULT 'active'
        )"
    )
    .execute(&pool)
    .await
    .expect("Failed to create vaults table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS transactions (
            id SERIAL PRIMARY KEY,
            vault_pubkey TEXT NOT NULL REFERENCES vaults(pubkey),
            tx_type TEXT NOT NULL,
            amount BIGINT NOT NULL,
            signature TEXT NOT NULL,
            timestamp BIGINT NOT NULL
        )"
    )
    .execute(&pool)
    .await
    .expect("Failed to create transactions table");

    println!("Server starting at http://127.0.0.1:8080");

    let broadcaster = Broadcaster::new().start();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(broadcaster.clone()))
            .service(api::health_check)
            .service(api::create_vault)
            .service(api::get_vault)
            .service(api::deposit)
            .service(api::withdraw)
            .service(api::get_tvl)
            .service(api::get_vault_transactions)
            .route("/ws", web::get().to(api::websocket_handler))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
