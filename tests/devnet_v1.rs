//! Real Devnet integration tests for V1 transaction submission and parsing.
//!
//! These tests spend Devnet SOL and are intentionally ignored by default. The embedded keypair is
//! public, untrusted, and MUST NEVER be funded on mainnet.

use std::{str::FromStr, sync::Arc, time::Duration};

use anyhow::{anyhow, bail, Context, Result};
use sol_parser_sdk::{parse_rpc_transaction_with_cost, parse_transaction_from_rpc, DexEvent};
use openarb_trade_sdk::{
    common::{
        keypair::load_keypair_from_string, GasFeeStrategy, TradeConfig, TradeTransactionVersion,
    },
    swqos::SwqosConfig,
    trading::{
        common::build_transaction_with_version,
        core::params::{DexParamEnum, PumpFunParams},
        factory::DexType,
    },
    AccountPolicy, BuyAmount, SimpleBuyParams, SolanaTrade, TradeTokenType,
};
use solana_client::{
    nonblocking::rpc_client::RpcClient as AsyncRpcClient,
    rpc_client::RpcClient as BlockingRpcClient, rpc_config::RpcTransactionConfig,
};
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    message::VersionedMessage,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
};
use solana_system_interface::instruction as system_instruction;
use solana_transaction_status_client_types::UiTransactionEncoding;

const DEFAULT_DEVNET_RPC_URL: &str = "https://api.devnet.solana.com";
const DEVNET_GENESIS_HASH: &str = "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG";

// Public Devnet-only fixture. Anyone can spend this wallet. Never send mainnet SOL to it.
const PUBLIC_DEVNET_KEYPAIR: [u8; 64] = [
    122, 58, 155, 225, 237, 53, 10, 4, 176, 209, 246, 11, 32, 221, 245, 20, 221, 154, 0, 52, 13,
    28, 166, 197, 108, 128, 160, 166, 186, 251, 190, 215, 201, 176, 29, 218, 121, 175, 82, 158, 87,
    219, 244, 201, 65, 173, 100, 227, 142, 243, 18, 71, 140, 73, 15, 110, 0, 75, 37, 75, 3, 240,
    134, 7,
];
const PUBLIC_DEVNET_WALLET: &str = "EaJhvu4V1GxSbonRmGxHbTz6FSBMamB9VYGdY2FiBgbx";

// Active, non-complete PumpFun Devnet bonding curve at the time this fixture was added.
// Override with DEVNET_PUMPFUN_MINT if the public fixture is later completed or removed.
const DEFAULT_DEVNET_PUMPFUN_MINT: &str = "2mPrXXhioKi9me1i4o3brEnpXj4MDuMwFki5GXuuTu5R";

const MIN_TEST_BALANCE: u64 = 30_000_000;
const AIRDROP_LAMPORTS: u64 = 100_000_000;
const V1_CU_LIMIT: u32 = 250_000;
const V1_CU_PRICE_MICRO_LAMPORTS: u64 = 4_000;
const V1_PRIORITY_FEE_LAMPORTS: u64 = 1_000;
const V1_LOADED_ACCOUNTS_LIMIT: u32 = 64 * 1024 * 1024;

fn devnet_rpc_url() -> String {
    std::env::var("DEVNET_RPC_URL").unwrap_or_else(|_| DEFAULT_DEVNET_RPC_URL.to_owned())
}

fn devnet_keypair() -> Result<Keypair> {
    if let Ok(value) = std::env::var("DEVNET_TEST_KEYPAIR") {
        return load_keypair_from_string(&value).context("invalid DEVNET_TEST_KEYPAIR");
    }
    Keypair::try_from(PUBLIC_DEVNET_KEYPAIR.as_slice()).context("invalid embedded Devnet keypair")
}

async fn assert_devnet(rpc: &AsyncRpcClient) -> Result<()> {
    let genesis_hash = rpc.get_genesis_hash().await.context("getGenesisHash failed")?;
    if genesis_hash.to_string() != DEVNET_GENESIS_HASH {
        bail!(
            "refusing to use test keypair on a non-Devnet cluster: genesis hash {}",
            genesis_hash
        );
    }
    Ok(())
}

async fn wait_for_success(rpc: &AsyncRpcClient, signature: &Signature) -> Result<()> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    loop {
        if let Some(status) = rpc
            .get_signature_status_with_commitment(signature, CommitmentConfig::confirmed())
            .await
            .with_context(|| format!("getSignatureStatuses failed for {signature}"))?
        {
            status.map_err(|error| anyhow!("transaction {signature} failed: {error:?}"))?;
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            bail!("transaction {signature} was not confirmed within 60 seconds");
        }
        tokio::time::sleep(Duration::from_millis(400)).await;
    }
}

async fn ensure_funded(rpc: &AsyncRpcClient, payer: &Pubkey) -> Result<()> {
    let balance = rpc.get_balance(payer).await.context("getBalance failed")?;
    if balance >= MIN_TEST_BALANCE {
        return Ok(());
    }

    let mut errors = Vec::new();
    for attempt in 1..=5 {
        match rpc.request_airdrop(payer, AIRDROP_LAMPORTS).await {
            Ok(signature) => match wait_for_success(rpc, &signature).await {
                Ok(()) => {
                    let funded =
                        rpc.get_balance(payer).await.context("getBalance after airdrop")?;
                    if funded >= MIN_TEST_BALANCE {
                        println!("Devnet airdrop: {signature}; wallet balance: {funded} lamports");
                        return Ok(());
                    }
                    errors.push(format!(
                        "attempt {attempt}: airdrop confirmed but balance is only {funded}"
                    ));
                }
                Err(error) => errors.push(format!("attempt {attempt}: {error:#}")),
            },
            Err(error) => errors.push(format!("attempt {attempt}: {error}")),
        }
        tokio::time::sleep(Duration::from_secs(attempt * 2)).await;
    }

    bail!(
        "unable to fund public Devnet wallet {payer}; set DEVNET_TEST_KEYPAIR to a funded \n\
         Devnet-only keypair or retry after the public faucet rate limit resets: {}",
        errors.join("; ")
    )
}

async fn fetch_confirmed_transaction(
    rpc: &AsyncRpcClient,
    signature: &Signature,
) -> Result<solana_transaction_status_client_types::EncodedConfirmedTransactionWithStatusMeta> {
    let config = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::Base64),
        commitment: Some(CommitmentConfig::confirmed()),
        max_supported_transaction_version: Some(1),
    };
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        match rpc.get_transaction_with_config(signature, config).await {
            Ok(transaction) => return Ok(transaction),
            Err(error) if tokio::time::Instant::now() < deadline => {
                println!("waiting for getTransaction({signature}): {error}");
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Err(error) => return Err(error).context("getTransaction failed after confirmation"),
        }
    }
}

async fn assert_v1_rpc_and_parser(
    rpc_url: &str,
    rpc: &AsyncRpcClient,
    signature: Signature,
) -> Result<Vec<DexEvent>> {
    let rpc_transaction = fetch_confirmed_transaction(rpc, &signature).await?;
    let decoded = rpc_transaction
        .transaction
        .transaction
        .decode()
        .context("RPC Base64 transaction did not decode with wincode")?;
    assert!(matches!(decoded.message, VersionedMessage::V1(_)), "RPC did not return a V1 message");

    let parsed = parse_rpc_transaction_with_cost(&rpc_transaction, None)
        .context("sol-parser-sdk failed to parse the fetched V1 transaction")?;
    assert_eq!(parsed.signature, signature);
    assert_eq!(parsed.cost.compute_unit_limit, Some(V1_CU_LIMIT));
    assert_eq!(parsed.cost.compute_unit_price_micro_lamports, None);
    assert_eq!(parsed.cost.priority_fee_lamports, Some(V1_PRIORITY_FEE_LAMPORTS));
    assert_eq!(parsed.cost.loaded_accounts_data_size_limit, Some(V1_LOADED_ACCOUNTS_LIMIT));
    assert_eq!(parsed.cost.heap_size, None);
    assert!(parsed.cost.transaction_fee_lamports.is_some());

    let rpc_url = rpc_url.to_owned();
    let events_from_fetch_api = tokio::task::spawn_blocking(move || -> Result<Vec<DexEvent>> {
        let client = BlockingRpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        loop {
            match parse_transaction_from_rpc(&client, &signature, None) {
                Ok(events) => return Ok(events),
                Err(error) if std::time::Instant::now() < deadline => {
                    println!("waiting for parser RPC fetch({signature}): {error}");
                    std::thread::sleep(Duration::from_millis(500));
                }
                Err(error) => return Err(error).context("parse_transaction_from_rpc failed"),
            }
        }
    })
    .await
    .context("parser RPC task panicked")??;
    assert_eq!(events_from_fetch_api.len(), parsed.events.len());

    Ok(parsed.events)
}

#[tokio::test]
#[ignore = "submits a real transaction and spends Devnet SOL"]
async fn devnet_v1_system_transfer_roundtrip() -> Result<()> {
    let rpc_url = devnet_rpc_url();
    let rpc = AsyncRpcClient::new_with_commitment(rpc_url.clone(), CommitmentConfig::confirmed());
    assert_devnet(&rpc).await?;

    let payer = Arc::new(devnet_keypair()?);
    if std::env::var("DEVNET_TEST_KEYPAIR").is_err() {
        assert_eq!(payer.pubkey().to_string(), PUBLIC_DEVNET_WALLET);
    }
    ensure_funded(&rpc, &payer.pubkey()).await?;

    let recipient = Keypair::new().pubkey();
    let recent_blockhash = rpc.get_latest_blockhash().await.context("getLatestBlockhash failed")?;
    let transaction = build_transaction_with_version(
        &payer,
        V1_CU_LIMIT,
        V1_CU_PRICE_MICRO_LAMPORTS,
        TradeTransactionVersion::V1,
        &[system_instruction::transfer(&payer.pubkey(), &recipient, 1_000_000)],
        &[],
        Some(recent_blockhash),
        None,
        "DevnetSystemTransfer",
        true,
        false,
        &Pubkey::default(),
        0.0,
        None,
    )?;
    let signature = rpc.send_transaction(&transaction).await.context("sendTransaction failed")?;
    wait_for_success(&rpc, &signature).await?;

    let events = assert_v1_rpc_and_parser(&rpc_url, &rpc, signature).await?;
    assert!(events.is_empty(), "a System transfer must not produce DEX events: {events:?}");
    println!("V1 System transfer parsed successfully: {signature}");
    Ok(())
}

#[tokio::test]
#[ignore = "executes a real PumpFun buy and spends Devnet SOL"]
async fn devnet_v1_pumpfun_buy_and_parser_balances() -> Result<()> {
    let rpc_url = devnet_rpc_url();
    let rpc = Arc::new(AsyncRpcClient::new_with_commitment(
        rpc_url.clone(),
        CommitmentConfig::confirmed(),
    ));
    assert_devnet(&rpc).await?;

    let payer = Arc::new(devnet_keypair()?);
    if std::env::var("DEVNET_TEST_KEYPAIR").is_err() {
        assert_eq!(payer.pubkey().to_string(), PUBLIC_DEVNET_WALLET);
    }
    ensure_funded(&rpc, &payer.pubkey()).await?;

    let mint = Pubkey::from_str(
        &std::env::var("DEVNET_PUMPFUN_MINT")
            .unwrap_or_else(|_| DEFAULT_DEVNET_PUMPFUN_MINT.to_owned()),
    )
    .context("invalid DEVNET_PUMPFUN_MINT")?;
    let pumpfun = PumpFunParams::from_mint_by_rpc(&rpc, &mint)
        .await
        .with_context(|| format!("failed to load PumpFun Devnet curve for {mint}"))?;
    if pumpfun.bonding_curve.complete || pumpfun.bonding_curve.real_token_reserves == 0 {
        bail!(
            "PumpFun Devnet fixture {mint} is no longer tradable; set DEVNET_PUMPFUN_MINT to an \n\
             active Devnet bonding-curve mint"
        );
    }

    let config = TradeConfig::builder(
        rpc_url.clone(),
        vec![SwqosConfig::Default(rpc_url.clone())],
        CommitmentConfig::confirmed(),
    )
    .transaction_version(TradeTransactionVersion::V1)
    .use_seed_optimize(false)
    .create_wsol_ata_on_startup(false)
    .log_enabled(true)
    .build();
    let client = SolanaTrade::new(payer.clone(), config).await;

    let gas_fee_strategy = GasFeeStrategy::new();
    gas_fee_strategy.set_default_rpc_fee_strategy(
        V1_CU_LIMIT,
        V1_CU_LIMIT,
        V1_CU_PRICE_MICRO_LAMPORTS,
        V1_CU_PRICE_MICRO_LAMPORTS,
    );
    let buy = SimpleBuyParams::new(
        DexType::PumpFun,
        TradeTokenType::SOL,
        mint,
        BuyAmount::WithMaxInput { quote_amount: 100_000 },
        DexParamEnum::PumpFun(pumpfun),
        rpc.get_latest_blockhash().await.context("getLatestBlockhash failed")?,
        gas_fee_strategy,
    )
    .slippage_basis_points(1_000)
    .account_policy(AccountPolicy::Auto)
    .wait_tx_confirmed(true);

    let (success, signatures, trade_error, _) = client.buy_simple(buy).await?;
    if !success {
        bail!("PumpFun Devnet buy failed: {trade_error:?}; signatures={signatures:?}");
    }
    let signature = signatures
        .first()
        .copied()
        .context("successful trade returned no transaction signature")?;
    wait_for_success(&rpc, &signature).await?;

    let events = assert_v1_rpc_and_parser(&rpc_url, &rpc, signature).await?;
    let trade = events.iter().find_map(|event| match event {
        DexEvent::PumpFunTrade(trade)
        | DexEvent::PumpFunBuy(trade)
        | DexEvent::PumpFunBuyExactSolIn(trade)
            if trade.mint == mint && trade.is_buy =>
        {
            Some(trade)
        }
        _ => None,
    });
    let trade = trade.context("parser did not return the submitted PumpFun buy event")?;
    assert_eq!(trade.metadata.signature, signature);
    assert_eq!(trade.user, payer.pubkey());
    assert!(trade.token_amount > 0);

    let token_balance = trade.token_balance.context("missing token_balance")?;
    let sol_balance = trade.sol_balance.context("missing sol_balance")?;
    assert!(
        token_balance >= trade.token_amount,
        "final token balance {token_balance} is below bought amount {}",
        trade.token_amount
    );

    println!(
        "V1 PumpFun trade parsed successfully: {signature}; final token balance {token_balance}; \
         final SOL balance {sol_balance} lamports"
    );
    Ok(())
}
