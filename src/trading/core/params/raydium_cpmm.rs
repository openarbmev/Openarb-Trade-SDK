use crate::common::SolanaRpcClient;
use solana_sdk::pubkey::Pubkey;

/// RaydiumCpmm protocol specific parameters
/// Configuration parameters specific to Raydium CPMM trading protocol
#[derive(Clone)]
pub struct RaydiumCpmmParams {
    /// Pool address
    pub pool_state: Pubkey,
    /// Amm config address
    pub amm_config: Pubkey,
    /// Base token mint address
    pub base_mint: Pubkey,
    /// Quote token mint address
    pub quote_mint: Pubkey,
    /// Base token reserve amount in the pool
    pub base_reserve: u64,
    /// Quote token reserve amount in the pool
    pub quote_reserve: u64,
    /// Base token vault address
    pub base_vault: Pubkey,
    /// Quote token vault address
    pub quote_vault: Pubkey,
    /// Base token program ID
    pub base_token_program: Pubkey,
    /// Quote token program ID
    pub quote_token_program: Pubkey,
    /// Observation state account
    pub observation_state: Pubkey,
}

impl RaydiumCpmmParams {
    pub fn from_trade(
        pool_state: Pubkey,
        amm_config: Pubkey,
        input_token_mint: Pubkey,
        output_token_mint: Pubkey,
        input_vault: Pubkey,
        output_vault: Pubkey,
        input_token_program: Pubkey,
        output_token_program: Pubkey,
        observation_state: Pubkey,
        base_reserve: u64,
        quote_reserve: u64,
    ) -> Self {
        Self {
            pool_state: pool_state,
            amm_config: amm_config,
            base_mint: input_token_mint,
            quote_mint: output_token_mint,
            base_reserve: base_reserve,
            quote_reserve: quote_reserve,
            base_vault: input_vault,
            quote_vault: output_vault,
            base_token_program: input_token_program,
            quote_token_program: output_token_program,
            observation_state: observation_state,
        }
    }

    pub async fn from_pool_address_by_rpc(
        rpc: &SolanaRpcClient,
        pool_address: &Pubkey,
    ) -> Result<Self, anyhow::Error> {
        let pool =
            crate::instruction::utils::raydium_cpmm::fetch_pool_state(rpc, pool_address).await?;
        let (token0_balance, token1_balance) =
            crate::instruction::utils::raydium_cpmm::get_pool_token_balances(
                rpc,
                pool_address,
                &pool.token0_mint,
                &pool.token1_mint,
            )
            .await?;
        Ok(Self {
            pool_state: *pool_address,
            amm_config: pool.amm_config,
            base_mint: pool.token0_mint,
            quote_mint: pool.token1_mint,
            base_reserve: token0_balance,
            quote_reserve: token1_balance,
            base_vault: pool.token0_vault,
            quote_vault: pool.token1_vault,
            base_token_program: pool.token0_program,
            quote_token_program: pool.token1_program,
            observation_state: pool.observation_key,
        })
    }
}
