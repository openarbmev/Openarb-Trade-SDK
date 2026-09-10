# Openarb Trade SDK

**用于构建 Solana 交易系统的 Rust SDK。**

Openarb Trade SDK 提供统一的交易参数、DEX 指令构建以及 RPC 和中继提交接口，
可作为交易机器人、交易工具和自有基础设施的执行层。

[English](README.md) · [示例](examples/README_CN.md) ·
[问题反馈](https://github.com/openarbmev/Openarb-Trade-SDK/issues)

## 功能

- 支持 PumpFun、PumpSwap、Bonk、Raydium AMM v4、Raydium CPMM 和 Meteora DAMM v2。
- 提供买卖参数、滑点设置、账户策略和协议状态接口。
- 支持 Solana RPC 及 Jito、Temporal、Bloxroute、Astralane、Glaive 等提交服务。
- 提供优先费、小费、地址查找表、持久化 Nonce、中间件及多钱包共享基础设施。

应用需要自行提供交易策略、钱包管理、行情数据及最新协议状态。
RPC 和中继服务需要单独配置，部分服务需要 API 凭据。

## 安装

通过 Git 使用 Openarb 版本：

```toml
[dependencies]
openarb-trade-sdk = { git = "https://github.com/openarbmev/Openarb-Trade-SDK", branch = "main" }
```

Rust 中使用 `openarb_trade_sdk` 导入。需要可复现构建时，将 `branch` 改为经过
检查的 `rev`，并提交应用的 `Cargo.lock`。此安装方式不依赖 Openarb 的 crates.io 发布。

```sh
git clone https://github.com/openarbmev/Openarb-Trade-SDK.git
cd Openarb-Trade-SDK
cargo check --workspace --all-targets
```

需要当前稳定版 Rust 和 Git。CI 使用 Linux；本机依赖可能还需要 C/C++ 工具链、
`pkg-config`、OpenSSL 开发文件及 CMake。解析器与流订阅依赖使用已发布的固定版本，
无需在相邻目录克隆其他仓库。

## 使用与文档

从 [客户端配置](examples/trading_client/README_CN.md) 和
[简化交易参数](examples/simple_trading/README_CN.md) 开始。部分示例在配置后会提交
真实交易，客户端初始化也可能创建代币账户。运行前阅读各示例说明，私钥及 API
凭据应通过环境变量或密钥库提供，不应提交到 Git。

- [交易参数](docs/TRADING_PARAMETERS_CN.md)
- [低延迟集成](docs/LOW_LATENCY_BOTS_CN.md)
- [Gas 费用策略](docs/GAS_FEE_STRATEGY_CN.md)
- [地址查找表](docs/ADDRESS_LOOKUP_TABLE_CN.md)
- [持久化 Nonce](docs/NONCE_CACHE_CN.md)
- [买入前风险检查](docs/PRE_BUY_RISK_GATE_CN.md)
- [Devnet 测试说明](tests/README.md)

## 开发

```sh
cargo check --workspace --all-targets
cargo test --workspace
```

GitHub Actions 在 Linux 上检查工作区。Devnet 交易测试默认忽略，需主动启用。
构建检查不能替代真实交易行为验证。

## 许可证与来源

本项目由 Openarb 独立维护，基于
[0xfnzero/sol-trade-sdk](https://github.com/0xfnzero/sol-trade-sdk)，保留原始作者的
Git 历史和版权声明。导入版本和修改范围见 [UPSTREAM.md](UPSTREAM.md)。

采用 [MIT 许可证](LICENSE)。原始版权 © 2024 MiracleAI-Labs；
Openarb 修改部分版权 © 2026 Openarb。
