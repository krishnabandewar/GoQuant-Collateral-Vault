# SPL Token Integration Guide

## Overview
The GoQuant Vault system integrates with Solana's SPL Token Program to manage collateral deposits and withdrawals securely.

## Architecture

### Cross-Program Invocations (CPI)
The vault uses CPI to interact with the SPL Token Program:

```rust
// Deposit: User signs the transfer
let cpi_accounts = Transfer {
    from: user_token_account,
    to: vault_token_account,
    authority: user (signer),
};
token::transfer(cpi_ctx, amount)?;

// Withdraw: Vault PDA signs the transfer
let seeds = &[b"vault", owner_key, &[bump]];
let signer = &[&seeds[..]];
token::transfer(cpi_ctx_with_signer, amount)?;
```

## Token Account Setup

### 1. Create Token Mint
```bash
spl-token create-token
# Returns: <MINT_ADDRESS>
```

### 2. Create User Token Account
```bash
spl-token create-account <MINT_ADDRESS>
# Returns: <USER_TOKEN_ACCOUNT>
```

### 3. Create Vault Token Account
```bash
# Derive vault PDA
solana-keygen grind --starts-with vault:1

# Create associated token account for vault PDA
spl-token create-account <MINT_ADDRESS> --owner <VAULT_PDA>
```

### 4. Mint Tokens for Testing
```bash
spl-token mint <MINT_ADDRESS> 10000 <USER_TOKEN_ACCOUNT>
```

## Integration Flow

### Deposit Flow
1. User approves token transfer
2. Smart contract validates amount > 0
3. CPI call to SPL Token Program
4. Update vault state (total_collateral)
5. Emit DepositEvent

### Withdraw Flow
1. Validate available balance (total - locked)
2. Generate PDA signer seeds
3. CPI call with PDA authority
4. Update vault state
5. Emit WithdrawEvent

## Security Considerations

### 1. Authority Validation
- Deposits require user signature
- Withdrawals use PDA-derived authority
- No external authority can move funds

### 2. Balance Tracking
```rust
// Separate tracking of locked vs available
pub struct Vault {
    pub total_collateral: u64,
    pub locked_collateral: u64,  // For open positions
}

// Available = Total - Locked
let available = total_collateral.checked_sub(locked_collateral)?;
```

### 3. Overflow Protection
All arithmetic uses checked operations:
```rust
vault.total_collateral.checked_add(amount).ok_or(VaultError::Overflow)?;
```

## Testing

### Unit Tests
```bash
cd anchor-program/collateral_vault
cargo test
```

### Integration Tests
```bash
anchor test
```

## Common Issues

### Issue: "Insufficient funds"
**Cause**: Trying to withdraw more than available balance
**Solution**: Check `total_collateral - locked_collateral`

### Issue: "Invalid authority"
**Cause**: PDA seeds mismatch
**Solution**: Verify seeds = [b"vault", owner.key(), &[bump]]

### Issue: "Token account not found"
**Cause**: Vault token account not created
**Solution**: Create associated token account for vault PDA
