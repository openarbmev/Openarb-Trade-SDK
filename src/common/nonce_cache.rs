use crate::common::SolanaRpcClient;
use solana_hash::Hash;
use solana_sdk::pubkey::Pubkey;
use tracing::error;

/// DurableNonceInfo structure to store durable nonce-related information
#[derive(Clone)]
pub struct DurableNonceInfo {
    /// Nonce account address
    pub nonce_account: Option<Pubkey>,
    /// Current nonce value
    pub current_nonce: Option<Hash>,
}

/// Fetch nonce information using RPC
pub async fn fetch_nonce_info(
    rpc: &SolanaRpcClient,
    nonce_account: Pubkey,
) -> Option<DurableNonceInfo> {
    match rpc.get_account(&nonce_account).await {
        Ok(account) => {
            // Parse nonce account manually: first 4 bytes is version, then 4 bytes authority type
            // For initialized nonce: version=0, authority_type=0, then authority (32 bytes), then blockhash (32 bytes), then fee_calculator
            if account.data.len() >= 80 {
                // Skip version (4) + authority_type (4) + authority (32) = 40 bytes
                // Then blockhash is at offset 40
                let blockhash_bytes: [u8; 32] = account.data[40..72].try_into().ok()?;
                return Some(DurableNonceInfo {
                    nonce_account: Some(nonce_account),
                    current_nonce: Some(Hash::from(blockhash_bytes)),
                });
            } else {
                error!("Nonce account data too short");
            }
        }
        Err(e) => {
            error!("Failed to get nonce account information: {:?}", e);
        }
    }
    None
}
