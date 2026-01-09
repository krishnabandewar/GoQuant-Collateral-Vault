# WebSocket Streams Implementation

## Overview
The Vault Manager Service now includes a real-time WebSocket interface to broadcast vault updates to connected clients (frontend dashboards, trading bots, etc.).

## Connection
- **URL**: `ws://localhost:8080/ws`
- **Protocol**: Standard WebSocket (RFC 6455)

## Message Protocol

### Server-to-Client Messages
The server broadcasts JSON messages for key events:

1. **Deposit Event**
   ```json
   {
     "type": "deposit",
     "vault_pubkey": "...",
     "amount": 1000000,
     "signature": "...",
     "timestamp": 1678900000
   }
   ```

2. **Withdrawal Event**
   ```json
   {
     "type": "withdrawal",
     "vault_pubkey": "...",
     "amount": 500000,
     "signature": "...",
     "timestamp": 1678900000
   }
   ```

3. **Balance Update**
   ```json
   {
     "type": "balance_update",
     "vault_pubkey": "...",
     "balance": 1500000,
     "timestamp": 1678900000
   }
   ```

### Client-to-Server Messages
- **Ping/Pong**: Clients should handle standard WebSocket Ping/Pong/Close frames.
- Clients can send `{"type": "ping"}` to receive a `{"type": "pong"}` JSON response for application-level heartbeats if needed.

## Implementation Details
- **Architecture**: Implemented using `actix-web-actors`.
- **Broadcaster**: A central `Broadcaster` actor manages all active WebSocket sessions.
- **Integration**: The REST API endpoints for `/vaults/deposit` and `/vaults/withdraw` automatically inject messages into the `Broadcaster`, ensuring real-time synchronization between the transactional API and the streaming interface.
