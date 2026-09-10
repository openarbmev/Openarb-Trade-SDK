//! Global account for the Pump.fun Solana Program
//!
//! This module contains the definition for the global configuration account.
//!
//! # Global Account
//!
//! The global account is used to store the global configuration for the Pump.fun program.
//!
//! # Fields
//!
//! - `discriminator`: Unique identifier for the global account
//! - `initialized`: Whether the global account has been initialized
//! - `authority`: Authority pubkey that can modify settings
//! - `fee_recipient`: Account that receives fees
//! - `initial_virtual_token_reserves`: Initial virtual token reserves for price calculations
//! - `initial_virtual_sol_reserves`: Initial virtual SOL reserves for price calculations
//! - `initial_real_token_reserves`: Initial actual token reserves available for trading
//! - `token_total_supply`: Total supply of tokens
//! - `fee_basis_points`: Fee in basis points (1/100th of a percent)
//! - `withdraw_authority`: Authority that can withdraw fees
//! - `enable_migrate`: Whether migration is enabled
//! - `pool_migration_fee`: Fee for pool migration
//! - `creator_fee`: Fee for creators
//! - `fee_recipients`: Array of fee recipient accounts

use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::instruction::utils::pumpfun::global_constants::{
    AUTHORITY, CREATOR_FEE, ENABLE_MIGRATE, FEE_BASIS_POINTS, FEE_RECIPIENT, GLOBAL_ACCOUNT,
    INITIAL_REAL_TOKEN_RESERVES, INITIAL_VIRTUAL_SOL_RESERVES, INITIAL_VIRTUAL_TOKEN_RESERVES,
    POOL_MIGRATION_FEE, PUMPFUN_AMM_FEE_1, PUMPFUN_AMM_FEE_2, PUMPFUN_AMM_FEE_3, PUMPFUN_AMM_FEE_4,
    PUMPFUN_AMM_FEE_5, PUMPFUN_AMM_FEE_6, PUMPFUN_AMM_FEE_7, TOKEN_TOTAL_SUPPLY,
    WITHDRAW_AUTHORITY,
};

/// Represents the global configuration account for token pricing and fees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalAccount {
    /// Unique identifier for the global account
    pub discriminator: u64,
    /// Pubkey of the global account  
    pub account: Pubkey,
    /// Whether the global account has been initialized
    pub initialized: bool,
    /// Authority that can modify global settings
    pub authority: Pubkey,
    /// Account that receives fees
    pub fee_recipient: Pubkey,
    /// Initial virtual token reserves for price calculations
    pub initial_virtual_token_reserves: u64,
    /// Initial virtual SOL reserves for price calculations
    pub initial_virtual_sol_reserves: u64,
    /// Initial actual token reserves available for trading
    pub initial_real_token_reserves: u64,
    /// Total supply of tokens
    pub token_total_supply: u64,
    /// Fee in basis points (1/100th of a percent)
    pub fee_basis_points: u64,
    /// Authority that can withdraw fees
    pub withdraw_authority: Pubkey,
    /// Whether migration is enabled
    pub enable_migrate: bool,
    /// Fee for pool migration
    pub pool_migration_fee: u64,
    /// Fee for creators
    pub creator_fee: u64,
    /// Array of fee recipient accounts
    pub fee_recipients: [Pubkey; 7],
}

impl GlobalAccount {
    /// Creates a new global account instance
    pub fn new() -> Self {
        Self {
            discriminator: 0,
            account: GLOBAL_ACCOUNT,
            initialized: true,
            authority: AUTHORITY,
            fee_recipient: FEE_RECIPIENT,
            initial_virtual_token_reserves: INITIAL_VIRTUAL_TOKEN_RESERVES,
            initial_virtual_sol_reserves: INITIAL_VIRTUAL_SOL_RESERVES,
            initial_real_token_reserves: INITIAL_REAL_TOKEN_RESERVES,
            token_total_supply: TOKEN_TOTAL_SUPPLY,
            fee_basis_points: FEE_BASIS_POINTS,
            withdraw_authority: WITHDRAW_AUTHORITY,
            enable_migrate: ENABLE_MIGRATE,
            pool_migration_fee: POOL_MIGRATION_FEE,
            creator_fee: CREATOR_FEE,
            fee_recipients: [
                PUMPFUN_AMM_FEE_1,
                PUMPFUN_AMM_FEE_2,
                PUMPFUN_AMM_FEE_3,
                PUMPFUN_AMM_FEE_4,
                PUMPFUN_AMM_FEE_5,
                PUMPFUN_AMM_FEE_6,
                PUMPFUN_AMM_FEE_7,
            ],
        }
    }

    /// Calculates the initial amount of tokens received for a given SOL amount
    ///
    /// # Arguments
    /// * `amount` - Amount of SOL to spend
    ///
    /// # Returns
    /// Amount of tokens that would be received
    pub fn get_initial_buy_price(&self, amount: u64) -> u64 {
        if amount == 0 {
            return 0;
        }

        let n: u128 = (self.initial_virtual_sol_reserves as u128)
            * (self.initial_virtual_token_reserves as u128);
        let i: u128 = (self.initial_virtual_sol_reserves as u128) + (amount as u128);
        let r: u128 = n / i + 1;
        let s: u128 = (self.initial_virtual_token_reserves as u128) - r;

        if s < (self.initial_real_token_reserves as u128) {
            s as u64
        } else {
            self.initial_real_token_reserves
        }
    }
}
