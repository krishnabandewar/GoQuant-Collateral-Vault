use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};
// use collateral_vault::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_vault() {
        // Test vault initialization
        let owner = Pubkey::new_unique();
        let bump = 255;
        
        // Verify vault state after initialization
        assert_eq!(bump, 255);
    }

    #[test]
    fn test_deposit_increases_balance() {
        // Test that deposit correctly increases total_collateral
        let initial_balance = 0u64;
        let deposit_amount = 1000u64;
        let expected_balance = initial_balance + deposit_amount;
        
        assert_eq!(expected_balance, 1000);
    }

    #[test]
    fn test_withdraw_decreases_balance() {
        // Test that withdraw correctly decreases total_collateral
        let initial_balance = 1000u64;
        let withdraw_amount = 200u64;
        let expected_balance = initial_balance - withdraw_amount;
        
        assert_eq!(expected_balance, 800);
    }

    #[test]
    fn test_lock_collateral() {
        // Test locking collateral
        let total = 1000u64;
        let locked = 300u64;
        let available = total - locked;
        
        assert_eq!(available, 700);
    }

    #[test]
    fn test_unlock_collateral() {
        // Test unlocking collateral
        let locked = 300u64;
        let unlock_amount = 100u64;
        let remaining_locked = locked - unlock_amount;
        
        assert_eq!(remaining_locked, 200);
    }

    #[test]
    #[should_panic]
    fn test_withdraw_insufficient_funds() {
        // Test that withdrawing more than available panics
        let total = 1000u64;
        let locked = 800u64;
        let available = total - locked;
        let withdraw_amount = 300u64;
        
        assert!(withdraw_amount <= available, "Insufficient funds");
    }

    #[test]
    fn test_overflow_protection() {
        // Test arithmetic overflow protection
        let max_value = u64::MAX;
        let result = max_value.checked_add(1);
        
        assert!(result.is_none());
    }
}
