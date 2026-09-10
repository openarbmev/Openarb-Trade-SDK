# Devnet integration tests

`devnet_v1.rs` contains ignored tests that submit real transactions and spend Devnet SOL. They
verify V1/wincode submission, RPC V1 retrieval, `sol-parser-sdk` parsing, and a real PumpFun buy.

The source contains a deliberately public Devnet-only keypair. Anyone can spend its balance. Never
send mainnet SOL or valuable assets to that address. For reliable runs, override it with a dedicated
Devnet-only keypair encoded as base58 or a 64-byte JSON array:

```bash
export DEVNET_TEST_KEYPAIR='[...]'
export DEVNET_RPC_URL='https://api.devnet.solana.com'
```

Run the network-independent workspace tests first, then run Devnet tests serially:

```bash
cargo test --workspace
cargo test --test devnet_v1 -- --ignored --nocapture --test-threads=1
```

The tests verify the cluster genesis hash before signing. If the fixed PumpFun curve becomes
complete, select another active Devnet mint:

```bash
export DEVNET_PUMPFUN_MINT='<active PumpFun Devnet mint>'
```

The PumpFun test disables deterministic seed-account optimization so repeated runs use the
idempotent associated-token-account creation path.
