-- GoQuant Vault Management System Schema
-- Compatible with PostgreSQL

-- 1. Vaults Table
CREATE TABLE IF NOT EXISTS vaults (
    id SERIAL PRIMARY KEY,
    owner TEXT NOT NULL,
    pubkey TEXT NOT NULL UNIQUE,
    balance BIGINT DEFAULT 0,
    status TEXT DEFAULT 'active'
);

-- 2. Transactions Table
CREATE TABLE IF NOT EXISTS transactions (
    id SERIAL PRIMARY KEY,
    vault_pubkey TEXT NOT NULL REFERENCES vaults(pubkey),
    tx_type TEXT NOT NULL, -- 'deposit', 'withdraw', 'lock', 'unlock'
    amount BIGINT NOT NULL,
    signature TEXT NOT NULL,
    timestamp BIGINT NOT NULL
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_vaults_owner ON vaults(owner);
CREATE INDEX IF NOT EXISTS idx_transactions_vault ON transactions(vault_pubkey);
