use crate::common::SolanaRpcClient;
use solana_sdk::pubkey::Pubkey;

/// Bonk protocol specific parameters
/// Configuration parameters specific to Bonk trading protocol
#[derive(Clone, Default)]
pub struct BonkParams {
    pub virtual_base: u128,
    pub virtual_quote: u128,
    pub real_base: u128,
    pub real_quote: u128,
    pub pool_state: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    /// Token program ID
    pub mint_token_program: Pubkey,
    pub platform_config: Pubkey,
    pub platform_associated_account: Pubkey,
    pub creator_associated_account: Pubkey,
    pub global_config: Pubkey,
}

impl BonkParams {
    pub fn immediate_sell(
        mint_token_program: Pubkey,
        platform_config: Pubkey,
        platform_associated_account: Pubkey,
        creator_associated_account: Pubkey,
        global_config: Pubkey,
    ) -> Self {
        Self {
            mint_token_program,
            platform_config,
            platform_associated_account,
            creator_associated_account,
            global_config,
            ..Default::default()
        }
    }
    pub fn from_trade(
        virtual_base: u64,
        virtual_quote: u64,
        real_base_after: u64,
        real_quote_after: u64,
        pool_state: Pubkey,
        base_vault: Pubkey,
        quote_vault: Pubkey,
        base_token_program: Pubkey,
        platform_config: Pubkey,
        platform_associated_account: Pubkey,
        creator_associated_account: Pubkey,
        global_config: Pubkey,
    ) -> Self {
        Self {
            virtual_base: virtual_base as u128,
            virtual_quote: virtual_quote as u128,
            real_base: real_base_after as u128,
            real_quote: real_quote_after as u128,
            pool_state: pool_state,
            base_vault: base_vault,
            quote_vault: quote_vault,
            mint_token_program: base_token_program,
            platform_config: platform_config,
            platform_associated_account: platform_associated_account,
            creator_associated_account: creator_associated_account,
            global_config: global_config,
        }
    }

    pub fn from_dev_trade(
        is_exact_in: bool,
        amount_in: u64,
        amount_out: u64,
        pool_state: Pubkey,
        base_vault: Pubkey,
        quote_vault: Pubkey,
        base_token_program: Pubkey,
        platform_config: Pubkey,
        platform_associated_account: Pubkey,
        creator_associated_account: Pubkey,
        global_config: Pubkey,
    ) -> Self {
        const DEFAULT_VIRTUAL_BASE: u128 = 1073025605596382;
        const DEFAULT_VIRTUAL_QUOTE: u128 = 30000852951;
        let _amount_in = if is_exact_in {
            amount_in
        } else {
            crate::instruction::utils::bonk::get_amount_in(
                amount_out,
                crate::instruction::utils::bonk::accounts::PROTOCOL_FEE_RATE,
                crate::instruction::utils::bonk::accounts::PLATFORM_FEE_RATE,
                crate::instruction::utils::bonk::accounts::SHARE_FEE_RATE,
                DEFAULT_VIRTUAL_BASE,
                DEFAULT_VIRTUAL_QUOTE,
                0,
                0,
                0,
            )
        };
        let real_quote = crate::instruction::utils::bonk::get_amount_in_net(
            amount_in,
            crate::instruction::utils::bonk::accounts::PROTOCOL_FEE_RATE,
            crate::instruction::utils::bonk::accounts::PLATFORM_FEE_RATE,
            crate::instruction::utils::bonk::accounts::SHARE_FEE_RATE,
        ) as u128;
        let _amount_out = if is_exact_in {
            crate::instruction::utils::bonk::get_amount_out(
                amount_in,
                crate::instruction::utils::bonk::accounts::PROTOCOL_FEE_RATE,
                crate::instruction::utils::bonk::accounts::PLATFORM_FEE_RATE,
                crate::instruction::utils::bonk::accounts::SHARE_FEE_RATE,
                DEFAULT_VIRTUAL_BASE,
                DEFAULT_VIRTUAL_QUOTE,
                0,
                0,
                0,
            ) as u128
        } else {
            amount_out as u128
        };
        let real_base = _amount_out;
        Self {
            virtual_base: DEFAULT_VIRTUAL_BASE,
            virtual_quote: DEFAULT_VIRTUAL_QUOTE,
            real_base: real_base,
            real_quote: real_quote,
            pool_state: pool_state,
            base_vault: base_vault,
            quote_vault: quote_vault,
            mint_token_program: base_token_program,
            platform_config: platform_config,
            platform_associated_account: platform_associated_account,
            creator_associated_account: creator_associated_account,
            global_config: global_config,
        }
    }

    pub async fn from_mint_by_rpc(
        rpc: &SolanaRpcClient,
        mint: &Pubkey,
        usd1_pool: bool,
    ) -> Result<Self, anyhow::Error> {
        let pool_address = crate::instruction::utils::bonk::get_pool_pda(
            mint,
            if usd1_pool {
                &crate::constants::USD1_TOKEN_ACCOUNT
            } else {
                &crate::constants::WSOL_TOKEN_ACCOUNT
            },
        )
        .unwrap();
        let pool_data =
            crate::instruction::utils::bonk::fetch_pool_state(rpc, &pool_address).await?;
        let token_account = rpc.get_account(&pool_data.base_mint).await?;
        let platform_associated_account =
            crate::instruction::utils::bonk::get_platform_associated_account(
                &pool_data.platform_config,
            );
        let creator_associated_account =
            crate::instruction::utils::bonk::get_creator_associated_account(&pool_data.creator);
        let platform_associated_account = platform_associated_account.unwrap();
        let creator_associated_account = creator_associated_account.unwrap();
        Ok(Self {
            virtual_base: pool_data.virtual_base as u128,
            virtual_quote: pool_data.virtual_quote as u128,
            real_base: pool_data.real_base as u128,
            real_quote: pool_data.real_quote as u128,
            pool_state: pool_address,
            base_vault: pool_data.base_vault,
            quote_vault: pool_data.quote_vault,
            mint_token_program: token_account.owner,
            platform_config: pool_data.platform_config,
            platform_associated_account,
            creator_associated_account,
            global_config: pool_data.global_config,
        })
    }
}
