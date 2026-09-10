//! 🚀 交易构建器对象池
//!
//! 预分配交易构建器,避免运行时分配:
//! - 对象池重用
//! - 零分配构建
//! - 零拷贝 I/O
//! - 内存预热

/// 预分配指令容量（单笔交易常见指令数）
const TX_BUILDER_INSTRUCTION_CAP: usize = 32;
/// 对象池最大容量
const TX_BUILDER_POOL_CAP: usize = 1000;
/// 多路提交并发数（与 async_executor SWQOS_DEDICATED_DEFAULT_THREADS 一致，保证不串行）
const PARALLEL_SENDER_COUNT: usize = 18;
/// 启动时预填充数量，必须 >= PARALLEL_SENDER_COUNT，否则 18 路并发 build 会触发分配或争抢
const TX_BUILDER_POOL_PREFILL: usize = 64;

use crate::common::TradeTransactionVersion;
use anyhow::Result;
use crossbeam_queue::ArrayQueue;
use once_cell::sync::Lazy;
use solana_message::AddressLookupTableAccount;
use solana_sdk::{
    hash::Hash,
    instruction::Instruction,
    message::{v0, v1, Message, VersionedMessage},
    pubkey::Pubkey,
};
use std::sync::Arc;
/// 预分配的交易构建器
pub struct PreallocatedTxBuilder {
    /// 预分配的指令容器
    instructions: Vec<Instruction>,
}

impl PreallocatedTxBuilder {
    fn new() -> Self {
        Self { instructions: Vec::with_capacity(TX_BUILDER_INSTRUCTION_CAP) }
    }

    /// 重置构建器 (清空但保留容量)
    #[inline(always)]
    fn reset(&mut self) {
        self.instructions.clear();
    }

    /// 🚀 零分配构建交易
    ///
    /// # 交易版本选择
    ///
    /// - `V0` 无地址查找表时构造 Legacy，有地址查找表时构造 V0，保持原有兼容行为。
    /// - `V1` 构造 `VersionedMessage::V1`，并拒绝地址查找表。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let msg = builder.build_zero_alloc(
    ///     &payer,
    ///     &ixs,
    ///     &[lookup_table],
    ///     blockhash,
    ///     TradeTransactionVersion::V0,
    ///     v1::TransactionConfig::empty(),
    /// );
    /// assert!(matches!(msg, VersionedMessage::V0(_)));
    /// ```
    #[inline(always)]
    pub fn build_zero_alloc(
        &mut self,
        payer: &Pubkey,
        instructions: &[Instruction],
        address_lookup_table_accounts: &[AddressLookupTableAccount],
        recent_blockhash: Hash,
        transaction_version: TradeTransactionVersion,
        v1_config: v1::TransactionConfig,
    ) -> Result<VersionedMessage> {
        self.reset();
        self.instructions.extend_from_slice(instructions);

        match transaction_version {
            TradeTransactionVersion::V0 => {
                if address_lookup_table_accounts.is_empty() {
                    let message = Message::new_with_blockhash(
                        &self.instructions,
                        Some(payer),
                        &recent_blockhash,
                    );
                    Ok(VersionedMessage::Legacy(message))
                } else {
                    let message = v0::Message::try_compile(
                        payer,
                        &self.instructions,
                        address_lookup_table_accounts,
                        recent_blockhash,
                    )?;
                    Ok(VersionedMessage::V0(message))
                }
            }
            TradeTransactionVersion::V1 => {
                if !address_lookup_table_accounts.is_empty() {
                    anyhow::bail!("V1 transactions do not support address lookup tables");
                }
                let message = v1::Message::try_compile_with_config(
                    payer,
                    &self.instructions,
                    recent_blockhash,
                    v1_config,
                )?;
                Ok(VersionedMessage::V1(message))
            }
        }
    }
}

/// 🚀 全局交易构建器对象池
static TX_BUILDER_POOL: Lazy<Arc<ArrayQueue<PreallocatedTxBuilder>>> = Lazy::new(|| {
    let pool = ArrayQueue::new(TX_BUILDER_POOL_CAP);
    let prefill = TX_BUILDER_POOL_PREFILL.max(PARALLEL_SENDER_COUNT);
    for _ in 0..prefill {
        let _ = pool.push(PreallocatedTxBuilder::new());
    }
    Arc::new(pool)
});

/// 🚀 从池中获取构建器
#[inline(always)]
pub fn acquire_builder() -> PreallocatedTxBuilder {
    TX_BUILDER_POOL.pop().unwrap_or_else(PreallocatedTxBuilder::new)
}

/// 🚀 归还构建器到池
#[inline(always)]
pub fn release_builder(mut builder: PreallocatedTxBuilder) {
    builder.reset();
    let _ = TX_BUILDER_POOL.push(builder);
}

/// 获取池统计
pub fn get_pool_stats() -> (usize, usize) {
    (TX_BUILDER_POOL.len(), TX_BUILDER_POOL.capacity())
}

/// 🚀 RAII 构建器包装器 (自动归还)
pub struct TxBuilderGuard {
    builder: Option<PreallocatedTxBuilder>,
}

impl TxBuilderGuard {
    pub fn new() -> Self {
        Self { builder: Some(acquire_builder()) }
    }

    pub fn get_mut(&mut self) -> &mut PreallocatedTxBuilder {
        self.builder.as_mut().unwrap()
    }
}

impl Drop for TxBuilderGuard {
    fn drop(&mut self) {
        if let Some(builder) = self.builder.take() {
            release_builder(builder);
        }
    }
}
