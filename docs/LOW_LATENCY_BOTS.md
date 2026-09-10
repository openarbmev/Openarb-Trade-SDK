# Low-Latency Bot Integration Checklist

Before subscription, initialize and warm `SolanaTrade`, RPC and SWQoS clients, a background blockhash cache or durable nonce pool, known ATAs, and ALTs. Restore signature/instruction deduplication and position state before accepting events.

The event hot path should be limited to:

```text
filter -> deduplicate -> reject stale event -> map post-trade state -> Simple*Params -> sign -> submit
```

Do not initialize clients, synchronously fetch a blockhash, query balances, or search for pools in this path. An RPC fallback is valid for incomplete shred data but is no longer a purely low-latency path.

## Pre-buy risk gate

Install a `TradeRiskGate` on `TradingClient` when a strategy must reject risky mints before buying. The gate runs synchronously after basic parameter validation and before transaction construction/submission.

Fetch remote risk data in a background task and atomically publish immutable local snapshots before the event reaches the buy path. Define missing and stale-cache behavior explicitly; do not call RPC or an audit API from the gate.

See the [Pre-Buy Risk Gate Guide](PRE_BUY_RISK_GATE.md) for its purpose, API behavior, complete cache example, fail-open/fail-closed policy, and hot-path requirements.

## Submit and confirmation latency

SDK timing logs separate local construction, submit, and confirmation. If `build_instructions` and `before_submit` are sub-millisecond but `submit` or `confirm` takes hundreds of milliseconds or seconds, the bottleneck is not local instruction building.

Common causes and fixes:

| Symptom | Likely cause | Action |
|---|---|---|
| `confirm` is slow but signature was returned | Commitment/RPC polling latency | Use `wait_tx_confirmed=false` and monitor signatures externally, or confirm at `processed`/`confirmed` instead of waiting for finalization |
| `submit` is slow on direct RPC | RPC endpoint latency or preflight path | Use a low-latency paid RPC/SWQoS lane and consider skip-preflight behavior through the selected sender |
| Transaction lands late under load | Insufficient priority fee or relay tip | Raise compute-unit price and provider tip within the strategy's budget |
| Direct example is slower than stream bot | Direct flow fetches pool state and balances | Preload pool params, ATAs, ALTs, blockhashes, and only refresh correctness-critical state outside the hot path |

For direct transactions, fetching pool state by RPC is expected to dominate more than SDK compute. For bots, keep direct RPC reads in a warmup/refresher task and pass fresh params plus a recent blockhash into `buy`/`sell`.

## Trade intent

| Goal | Parameter |
|---|---|
| Exact spend with minimum-output protection | `BuyAmount::ExactInput` |
| Fill-priority sniping/arbitrage with maximum-cost protection | `BuyAmount::WithMaxInput` |
| Exact token output with maximum-input protection | `BuyAmount::ExactOutput` |
| Sell an exact token amount | `SellAmount::ExactInput` |

`WithMaxInput` still enforces slippage. Never use `min_out = 0` as routine error handling.
Exact-output support is protocol- and pool-direction-specific. PumpSwap exposes exact output through its on-chain `buy` instruction, but its `sell` instruction accepts exact base input plus minimum quote output; the SDK rejects `SellAmount::ExactOutput` when that direction would require `sell`.

Use post-trade event reserves. Preserve PumpFun quote mint, creator/vault, token program, cashback, and mayhem fields. PumpSwap event integrations should use `from_trade_with_fee_basis_points`. Refresh delayed sells because the triggering trade and your own buy both change pool state. Durable nonce extends transaction validity, not quote validity.

For `BuySlippageBelowMinBaseAmountOut`, discard the old transaction, obtain newer reserves and fee rates, enforce a quote-age limit, and rebuild only within a bounded retry policy.
After a submit timeout or ambiguous relay error, reconcile the signature and position before retrying. A retry policy may rebuild quotes automatically only when the previous transaction is known not to have been submitted.

Reference examples:

- `fnzero-examples/pumpfun_grpc_sniper`
- `fnzero-examples/pumpfun_shredstream_sniper`
- `examples/pumpswap_trading`
