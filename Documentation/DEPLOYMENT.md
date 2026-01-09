# Deployment Guide

## Prerequisites
- Solana CLI installed and configured
- Anchor CLI v0.32.0+
- Sufficient SOL for deployment (~2 SOL recommended)
- Wallet keypair configured

## Step 1: Configure Solana CLI

### Set Cluster
```bash
# For testing (recommended first)
solana config set --url devnet

# For production
solana config set --url mainnet-beta
```

### Check Balance
```bash
solana balance
# If insufficient, airdrop on devnet:
solana airdrop 2
```

### Set Wallet
```bash
solana config set --keypair ~/.config/solana/id.json
```

## Step 2: Build the Program

```bash
cd anchor-program/collateral_vault
anchor build
```

This generates:
- `target/deploy/collateral_vault.so` - The compiled program
- `target/idl/collateral_vault.json` - The IDL file

## Step 3: Deploy to Devnet

```bash
anchor deploy
```

**Expected Output:**
```
Deploying workspace: https://api.devnet.solana.com
Upgrade authority: <YOUR_WALLET>
Deploying program "collateral_vault"...
Program Id: <PROGRAM_ID>
Deploy success
```

**Save the Program ID** - You'll need this for the backend configuration.

## Step 4: Update Program ID

Edit `Anchor.toml`:
```toml
[programs.devnet]
collateral_vault = "<PROGRAM_ID_FROM_STEP_3>"
```

Also update `lib.rs`:
```rust
declare_id!("<PROGRAM_ID_FROM_STEP_3>");
```

Rebuild:
```bash
anchor build
anchor deploy
```

## Step 5: Verify Deployment

```bash
solana program show <PROGRAM_ID>
```

## Step 6: Deploy Backend Service

### Local Deployment

1. **Setup Database**:
```bash
# For PostgreSQL
createdb goquant_vaults
psql -d goquant_vaults -f backend-service/vault-manager/migrations/schema_postgres.sql
```

2. **Configure Environment**:
Create `.env` file:
```env
DATABASE_URL=postgres://user:password@localhost/goquant_vaults
SOLANA_RPC_URL=https://api.devnet.solana.com
PROGRAM_ID=<YOUR_PROGRAM_ID>
```

3. **Run Backend**:
```bash
cd backend-service/vault-manager
cargo run --release
```

### Cloud Deployment (Railway/Heroku)

1. **Prepare Procfile**:
```
web: cd backend-service/vault-manager && cargo run --release
```

2. **Set Environment Variables**:
- `DATABASE_URL`
- `SOLANA_RPC_URL`
- `PROGRAM_ID`

3. **Deploy**:
```bash
git push railway main
# or
git push heroku main
```

## Step 7: Initialize First Vault

```bash
# Using Anchor client
anchor run initialize-vault
```

Or via backend API:
```bash
curl -X POST http://localhost:8080/vaults \
  -H "Content-Type: application/json" \
  -d '{"owner": "<WALLET_ADDRESS>", "pubkey": "<VAULT_PDA>"}'
```

## Step 8: Test the System

Run the demo client:
```bash
node backend-service/scripts/demo_client.js
```

## Monitoring

### Check Program Logs
```bash
solana logs <PROGRAM_ID>
```

### Check Backend Health
```bash
curl http://localhost:8080/health
```

### Check TVL
```bash
curl http://localhost:8080/tvl
```

## Troubleshooting

### Issue: "Insufficient funds for deployment"
**Solution**: 
```bash
solana airdrop 2  # On devnet
```

### Issue: "Program deploy failed"
**Solution**: Ensure program size is under limit:
```bash
ls -lh target/deploy/collateral_vault.so
# Should be < 200KB
```

### Issue: "Backend can't connect to database"
**Solution**: Verify DATABASE_URL and ensure PostgreSQL is running:
```bash
pg_isready
```

## Production Checklist

- [ ] Program deployed to mainnet-beta
- [ ] Program upgrade authority secured (multisig recommended)
- [ ] Backend deployed with SSL/TLS
- [ ] Database backups configured
- [ ] Monitoring and alerting setup
- [ ] Rate limiting enabled on API
- [ ] Security audit completed
- [ ] Documentation published
