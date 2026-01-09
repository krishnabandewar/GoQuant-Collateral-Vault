# GoQuant 🚀
**Non-Custodial Collateral Vault on Solana**

[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange)](https://www.rust-lang.org/)
[![Solana](https://img.shields.io/badge/Solana-1.18%2B-blueviolet)](https://solana.com/)
[![License](https://img.shields.io/badge/License-Private-lightgrey)]()

**GoQuant** is a high-performance custody layer for decentralized perpetual exchanges. It combines the security of on-chain Program Derived Addresses (PDAs) with the speed of a Rust/Actix backend and the reliability of PostgreSQL.

---

## 🏗️ System Architecture

| Component | Tech Stack | Responsibility |
|-----------|------------|----------------|
| **Smart Contract** | Solana / Anchor | Custody, Withdrawals, CPI Atomic Swaps |
| **Backend** | Rust / Actix / SQLx | Indexing, State Tracking, REST API |
| **Real-Time** | WebSockets | Instant Deposit/Balance Updates |
| **Storage** | PostgreSQL | Immutable Transaction History |

👉 **[Read the Full Technical Documentation](./Documentation/DOCUMENTATION.md)**

---

## ✨ Key Features

- **🔐 Trustless Security**: Funds are held in Program Derived Addresses (PDAs). Only the smart contract can authorize withdrawals—never the backend admin key.
- **⚡ Real-Time**: WebSocket integration pushes balance updates to the frontend in milliseconds.
- **🛡️ Audit Trail**: Every on-chain event is indexed and verified in a PostgreSQL database (Postgres).
- **🪙 SPL Integration**: Seamlessly handles any SPL token (USDT/USDC) via atomic Cross-Program Invocations (CPI).

---

## 🚀 Quick Start

### 1. Prerequisites
- **Rust** & **Solana CLI** installed.
- **PostgreSQL** running locally.
- **Node.js** (for demo client).

### 2. Setup (Backend)
```bash
# 1. Setup Database
createdb goquant_vaults

# 2. Configure Environment
cd backend-service/vault-manager
echo "DATABASE_URL=postgres://postgres:password@localhost/goquant_vaults" > .env

# 3. proper Run
cargo run
```

### 3. Setup (Smart Contract)
```bash
cd anchor-program/collateral_vault
anchor build
anchor test
```

### 4. Run the Demo
Simulate a full user flow (Deposit → WebSocket Alert → Withdraw):
```bash
cd backend-service/demo-client
node test_demo.js
```

---

## 📂 Repository Structure

```
├── anchor-program/             # 🧠 Solana Smart Contract
│   └── collateral_vault/       #    Anchor Framework Code
├── backend-service/            # ⚡ Off-chain Infrastructure
│   ├── vault-manager/          #    Rust API & WebSocket Service
│   └── demo-client/            #    Node.js Simulation Client
└── Documentation/              # 📚 Detailed Guides & Specs
    ├── DOCUMENTATION.md        #    Master Technical Spec
    ├── DEPLOYMENT.md           #    Devnet Deployment Guide
    └── SECURITY_ANALYSIS.md    #    Safety & Risk Report
```

---

## 🧪 Testing

We maintain rigorous testing standards:
*   **Unit Tests**: `cargo test` in `anchor-program` (Arithmetic safety).
*   **Integration**: `anchor test` (PDA lifecycles).
*   **End-to-End**: `demo-client` verifies the full stack.

---
*Generated for GoQuant Technical Assignment.*
