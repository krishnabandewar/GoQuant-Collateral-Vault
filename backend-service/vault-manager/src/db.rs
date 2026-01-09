use sqlx::{PgPool, FromRow, Error};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, FromRow, PartialEq)]
pub struct Vault {
    pub id: i32,  // SERIAL in PostgreSQL is i32
    pub owner: String,
    pub pubkey: String,
    pub balance: i64,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, PartialEq)]
pub struct Transaction {
    pub id: i32,  // SERIAL in PostgreSQL is i32
    pub vault_pubkey: String,
    pub tx_type: String, // 'deposit' or 'withdraw'
    pub amount: i64,
    pub signature: String,
    pub timestamp: i64,
}

pub async fn insert_vault(pool: &PgPool, owner: &str, pubkey: &str) -> Result<(), Error> {
    sqlx::query("INSERT INTO vaults (owner, pubkey) VALUES ($1, $2)")
        .bind(owner)
        .bind(pubkey)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn find_vault(pool: &PgPool, pubkey: &str) -> Result<Vault, Error> {
    sqlx::query_as::<_, Vault>("SELECT * FROM vaults WHERE pubkey = $1")
        .bind(pubkey)
        .fetch_one(pool)
        .await
}

pub async fn update_balance(pool: &PgPool, pubkey: &str, amount: i64) -> Result<(), Error> {
    sqlx::query("UPDATE vaults SET balance = balance + $1 WHERE pubkey = $2")
        .bind(amount)
        .bind(pubkey)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn insert_transaction(pool: &PgPool, vault_pubkey: &str, tx_type: &str, amount: i64, signature: &str) -> Result<(), Error> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    sqlx::query("INSERT INTO transactions (vault_pubkey, tx_type, amount, signature, timestamp) VALUES ($1, $2, $3, $4, $5)")
        .bind(vault_pubkey)
        .bind(tx_type)
        .bind(amount)
        .bind(signature)
        .bind(timestamp)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_transactions(pool: &PgPool, vault_pubkey: &str) -> Result<Vec<Transaction>, Error> {
    sqlx::query_as::<_, Transaction>("SELECT * FROM transactions WHERE vault_pubkey = $1 ORDER BY timestamp DESC")
        .bind(vault_pubkey)
        .fetch_all(pool)
        .await
}

pub async fn get_total_tvl(pool: &PgPool) -> Result<i64, Error> {
    let result: (i64,) = sqlx::query_as("SELECT COALESCE(SUM(balance), 0)::BIGINT FROM vaults")
        .fetch_one(pool)
        .await?;
    Ok(result.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn test_create_and_find_vault() {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            "CREATE TABLE vaults (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                owner TEXT NOT NULL,
                pubkey TEXT NOT NULL UNIQUE,
                balance INTEGER DEFAULT 0,
                status TEXT DEFAULT 'active'
            )"
        )
        .execute(&pool)
        .await
        .unwrap();

        let owner = "test_owner";
        let pubkey = "test_pubkey";

        insert_vault(&pool, owner, pubkey).await.unwrap();

        let vault = find_vault(&pool, pubkey).await.unwrap();

        assert_eq!(vault.owner, owner);
        assert_eq!(vault.pubkey, pubkey);
        assert_eq!(vault.balance, 0);
        assert_eq!(vault.status, "active");
    }
}
