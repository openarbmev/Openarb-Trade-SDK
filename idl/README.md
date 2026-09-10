# IDL Sources

Unversioned IDLs track current protocol-maintained files. Explicitly versioned files such as
`_011`, `_015`, or `_090` remain compatibility snapshots. Canonical SHA-256 values use
`jq -S -c . | sha256sum`, so JSON formatting does not affect comparisons.

| Local IDL | Official upstream | Version | Canonical SHA-256 |
| --- | --- | --- | --- |
| `pump.json` | `pump-fun/pump-public-docs/idl/pump.json` | 0.1.0 | `ab1b3b5a85b2bb3aade5dc7bacb2f5e75e62a242db6f2828d9addbf09d8d864e` |
| `pump_amm.json` | `pump-fun/pump-public-docs/idl/pump_amm.json` | 0.1.0 | `b85ddba5f4611e9e2cd0695d1204a40f16db15d48b1b8ad4177abee194dc5141` |
| `pump_fees.json` | `pump-fun/pump-public-docs/idl/pump_fees.json` | 0.1.0 | `1177093f22cfef08d5474025b44ace9ba97960ecd04dc87b4530a33072194bb5` |
| `raydium_clmm.json` | `raydium-io/raydium-idl/raydium_clmm/raydium_clmm.json` | 0.1.0 | `ce860422ec1e3284e89e165d9740c9f19f861ab45e0ad5bfc7ca647b6af39496` |
| `raydium_cpmm.json` | `raydium-io/raydium-idl/raydium_cpmm/raydium_cp_swap.json` | 0.2.0 | `a4cc67efdc1374a3d6f8a1d7f16627bcda2deecdb65b2413fa336e973b97b52f` |
| `raydium_launchpad.json` | `raydium-io/raydium-idl/raydium_launchpad/raydium_launchpad.json` | 0.2.0 | `03713b809cd62272f63ebf5ca5ad06a64f6f7a3ac493a9f49d2c3b2aa43aca68` |
| `meteora_lb_clmm.json` | `MeteoraAg/dlmm-sdk/idls/dlmm.json` | 0.12.0 | `1bc4333e5702dddb51d9ad92b6e9298940c9d9ff7f92fd761b634a03cf2d7daf` |
| `meteora_damm_v2.json` | `MeteoraAg/damm-v2-sdk/src/idl/cp_amm.json` | 0.2.2 | `44cba19689609a51e3c0ab44fb04f7c85e9eacf72f05fd013f0f4c753eb42f0a` |
| `meteora_dynamic_bonding_curve.json` | `MeteoraAg/dynamic-bonding-curve-sdk/packages/dynamic-bonding-curve/src/idl/dynamic-bonding-curve/idl.json` | 0.2.0 | `b28d0683f164e8ebc510190a605be493ea2ecd39c02c8852960e41120847a822` |
| `meteora_amm.json` | `MeteoraAg/dynamic-bonding-curve/idls/dynamic_amm.json` | 0.5.2 | `21f59f98c8a593ac2a9d976889cff8d8b618fe3aa1fb5c0132d5ca598c59a833` |
| `orca_whirlpool.json` / `orca_whirlpool_v2.json` | `@orca-so/whirlpools-sdk@0.22.0/dist/artifacts/whirlpool.json` | 0.9.0 | `301335733544288d52aae316827c4e2bbf4e27edbd3d87bd1711de1251e3b1ee` |

Raydium AMM V4 is not an Anchor program and has no current protocol-maintained Anchor IDL.
`raydium_amm_v4.json` and `raydium_pool_v4.json` remain compatibility references; current
instruction and `ray_log` layouts are defined by `raydium-io/raydium-amm`.
