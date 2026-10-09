# Openarb Trade SDK

**Build Solana trading systems with a unified Rust interface.**

Openarb Trade SDK provides transaction construction, DEX integrations, and RPC or
relay submission for Solana trading applications. Use it as the execution layer
for your own bots, trading tools, and infrastructure.

[Getting started](#getting-started) · [Examples](examples/README.md) ·
[Documentation](#documentation) · [中文](README_CN.md) ·
[Issues](https://github.com/openarbmev/Openarb-Trade-SDK/issues)

## What it supports

| Area | Capabilities |
| --- | --- |
| DEX integrations | PumpFun, PumpSwap, Bonk, Raydium AMM v4, Raydium CPMM, Meteora DAMM v2 |
| Trading interface | Buy and sell parameters, slippage, account policies, and protocol-specific state |
| Transaction delivery | Solana RPC and configurable relay clients including Jito, Temporal, Bloxroute, Astralane, Glaive, OrbitFlare Apex, and others |
| Transaction controls | Priority fees, relay tips, address lookup tables, durable nonces, and middleware |
| Shared infrastructure | Reuse RPC and submission clients across multiple wallets |
| Integration tools | Pool and price utilities, risk-gate hooks, and workspace examples |

The SDK builds and submits transactions. Your application supplies its trading
strategy, wallet management, market data, and current protocol state. RPC and
relay services are configured separately; some providers require credentials.

## Getting started

### Add the dependency

Use the Git repository to install the Openarb package:

```toml
[dependencies]
openarb-trade-sdk = { git = "https://github.com/openarbmev/Openarb-Trade-SDK", branch = "main" }
solana-commitment-config = "3.1.1"
```

The package name is `openarb-trade-sdk`; Rust imports use `openarb_trade_sdk`.
For reproducible application builds, replace `branch = "main"` with
`rev = "<reviewed-commit-sha>"` and commit your application's `Cargo.lock`.
These instructions use Git and do not require an Openarb crates.io release.

### Configure a client

This example prepares configuration only. It does not connect to an RPC service,
load a wallet, or submit a transaction.

```rust
use openarb_trade_sdk::{
    common::TradeConfig,
    swqos::SwqosConfig,
};
use solana_commitment_config::CommitmentConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = std::env::var("RPC_URL")?;
    let _config = TradeConfig::builder(
        rpc_url.clone(),
        vec![SwqosConfig::Default(rpc_url)],
        CommitmentConfig::processed(),
    )
    .create_wsol_ata_on_startup(false)
    .build();

    Ok(())
}
```

Continue with the [client setup guide](examples/trading_client/README.md) and
[simple trading example](examples/simple_trading/README.md) to construct a client
and buy/sell parameters. Supply real pool state and a recent blockhash or durable
nonce from your application's data layer before submitting a trade.

### Work with the source

Install a current stable [Rust toolchain](https://rustup.rs/) and Git. Linux is the
CI build target. Native dependencies may also require a C/C++ toolchain,
`pkg-config`, OpenSSL development headers, and CMake.

```sh
git clone https://github.com/openarbmev/Openarb-Trade-SDK.git
cd Openarb-Trade-SDK
cargo check --workspace --all-targets
```

The parser and streamer dependencies are pinned to published crates; sibling
repository checkouts are not required.

## Examples

Each example has its own README and configuration requirements. Start with the
[example index](examples/README.md) before running a workspace package.

| Goal | Example |
| --- | --- |
| Build high-level buy/sell parameters | [simple_trading](examples/simple_trading/README.md) |
| Configure wallets and submission providers | [trading_client](examples/trading_client/README.md) |
| Share clients across wallets | [shared_infrastructure](examples/shared_infrastructure/README.md) |
| Integrate a PumpSwap stream | [pumpswap_trading](examples/pumpswap_trading/README.md) |
| Customize transaction instructions | [middleware_system](examples/middleware_system/README.md) |
| Use durable nonces | [nonce_cache](examples/nonce_cache/README.md) |

Some examples submit transactions once configured, and client initialization can
create token accounts. Read the selected example before running it. Keep private
keys and API tokens in your environment or a keystore, outside Git.

## Documentation

- [Trading parameters and account policies](docs/TRADING_PARAMETERS.md)
- [Low-latency bot integration](docs/LOW_LATENCY_BOTS.md)
- [Gas fees and relay tips](docs/GAS_FEE_STRATEGY.md)
- [Address lookup tables](docs/ADDRESS_LOOKUP_TABLE.md)
- [Durable nonce cache](docs/NONCE_CACHE.md)
- [Pre-buy risk gate](docs/PRE_BUY_RISK_GATE.md)
- [PumpFun and PumpSwap cashback](docs/PUMP_CASHBACK_README.md)
- [Devnet integration test setup](tests/README.md)

## Development

```sh
cargo check --workspace --all-targets
cargo test --workspace
```

GitHub Actions checks the workspace on Linux. Devnet transaction tests are
ignored by default and require deliberate opt-in; see their setup guide above.
Build checks do not establish live trading correctness or provider availability.

| Directory | Purpose |
| --- | --- |
| `src/client/` | Public client and shared infrastructure |
| `src/instruction/` | DEX instruction builders |
| `src/trading/` | Execution, transaction construction, and middleware |
| `src/swqos/` | RPC and relay submission clients |
| `src/common/` | Configuration, fees, nonces, and account helpers |
| `src/utils/` | Quote and price calculations |
| `examples/` | Integration templates and guides |
| `tests/` | Integration tests |

Report reproducible bugs or propose improvements through
[GitHub Issues](https://github.com/openarbmev/Openarb-Trade-SDK/issues).
Include the commit, build environment, and sanitized error output.

## License

Licensed under [MIT](LICENSE).
