use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("EcE7TMxnpHknZKfn3Sb6iJrojhFFt3dFfxNM2nMZGxFp");

#[program]
pub mod collateral_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, bump: u8) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.owner = ctx.accounts.owner.key();
        vault.bump = bump;
        vault.total_collateral = 0;
        vault.locked_collateral = 0;
        
        msg!("Vault initialized for owner: {}", vault.owner);
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        require!(amount > 0, VaultError::InvalidAmount);

        // Transfer tokens from user to vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Update state
        let vault = &mut ctx.accounts.vault;
        vault.total_collateral = vault.total_collateral.checked_add(amount).ok_or(VaultError::Overflow)?;
        
        emit!(DepositEvent {
            owner: ctx.accounts.owner.key(),
            amount,
            new_balance: vault.total_collateral,
        });

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        
        require!(amount > 0, VaultError::InvalidAmount);
        
        // Calculate available balance (total - locked)
        let available_balance = vault.total_collateral.checked_sub(vault.locked_collateral).ok_or(VaultError::Overflow)?;
        require!(available_balance >= amount, VaultError::InsufficientFunds);

        // seed for signing
        let seeds = &[
            b"vault".as_ref(),
            ctx.accounts.owner.key.as_ref(),
            &[vault.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: vault.to_account_info(), 
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;

        vault.total_collateral = vault.total_collateral.checked_sub(amount).ok_or(VaultError::Overflow)?;

        emit!(WithdrawEvent {
            owner: ctx.accounts.owner.key(),
            amount,
            new_balance: vault.total_collateral,
        });

        Ok(())
    }

    // New: Lock Collateral
    pub fn lock_collateral(ctx: Context<LockCollateral>, amount: u64) -> Result<()> {
         let vault = &mut ctx.accounts.vault;
         let available_balance = vault.total_collateral.checked_sub(vault.locked_collateral).ok_or(VaultError::Overflow)?;
         require!(available_balance >= amount, VaultError::InsufficientFunds);

         vault.locked_collateral = vault.locked_collateral.checked_add(amount).ok_or(VaultError::Overflow)?;
         
         msg!("Locked {} collateral. Total Locked: {}", amount, vault.locked_collateral);
         Ok(())
    }

    // New: Unlock Collateral
    pub fn unlock_collateral(ctx: Context<UnlockCollateral>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        require!(vault.locked_collateral >= amount, VaultError::InvalidAmount);

        vault.locked_collateral = vault.locked_collateral.checked_sub(amount).ok_or(VaultError::Overflow)?;
        
        msg!("Unlocked {} collateral. Total Locked: {}", amount, vault.locked_collateral);
        Ok(())
    }

    // New: Transfer Collateral (Between vaults or programs)
    // NOTE: Simplified for this assignment to transfer from Vault -> Destination Token Account (Internal settlement)
    pub fn transfer_collateral(ctx: Context<TransferCollateral>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        
        // Logic: Transfering collateral usually implies using the 'locked' portion for settlement
        // or just available funds depending on context. Assuming available funds for now or settlement logic.
        // For safety, let's assume we can only transfer available funds.
        let available_balance = vault.total_collateral.checked_sub(vault.locked_collateral).ok_or(VaultError::Overflow)?;
        require!(available_balance >= amount, VaultError::InsufficientFunds);

        let seeds = &[
            b"vault".as_ref(),
            ctx.accounts.owner.key.as_ref(),
            &[vault.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.destination_token_account.to_account_info(),
            authority: vault.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;

        vault.total_collateral = vault.total_collateral.checked_sub(amount).ok_or(VaultError::Overflow)?;
        
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct Initialize<'info> {
    #[account(
        init, 
        seeds = [b"vault", owner.key().as_ref()], 
        bump, 
        payer = owner, 
        space = 8 + 32 + 8 + 8 + 1 // discriminator + owner + total + locked + bump
    )]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut, seeds = [b"vault", owner.key().as_ref()], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut, seeds = [b"vault", owner.key().as_ref()], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct LockCollateral<'info> {
    #[account(mut, seeds = [b"vault", owner.key().as_ref()], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub owner: Signer<'info>, // Only owner/authorized program can lock (simplified to owner for now)
}

#[derive(Accounts)]
pub struct UnlockCollateral<'info> {
    #[account(mut, seeds = [b"vault", owner.key().as_ref()], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct TransferCollateral<'info> {
    #[account(mut, seeds = [b"vault", owner.key().as_ref()], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub destination_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Vault {
    pub owner: Pubkey,
    pub total_collateral: u64,
    pub locked_collateral: u64,
    pub bump: u8,
}

#[error_code]
pub enum VaultError {
    #[msg("Insufficient funds in vault.")]
    InsufficientFunds,
    #[msg("Amount must be greater than zero.")]
    InvalidAmount,
    #[msg("Arithmetic overflow.")]
    Overflow,
}

#[event]
pub struct DepositEvent {
    pub owner: Pubkey,
    pub amount: u64,
    pub new_balance: u64,
}

#[event]
pub struct WithdrawEvent {
    pub owner: Pubkey,
    pub amount: u64,
    pub new_balance: u64,
}
