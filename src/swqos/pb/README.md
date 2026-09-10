# Proto 生成的代码说明

## 概述

这个目录包含了从 `.proto` 文件预生成的 Rust 代码。

## 文件说明

- `serverpb.rs` - 从 `blockrazor.proto` 生成的 gRPC 代码
  - 消息类型: `SendRequest`, `SendResponse`, `HealthRequest`, `HealthResponse`
  - gRPC 客户端: `server_client::ServerClient`
  - gRPC 服务端: `server_server::Server`

## 用户使用

用户**不需要**安装 `protoc` 或编译 proto 文件。这些代码已经预生成好了，可以直接使用。

在 `blockrazor.rs` 中使用：
```rust
pub mod serverpb {
    include!("pb/serverpb.rs");
}
```

## 开发者如何重新生成代码

如果你修改了 `.proto` 文件并需要重新生成代码：

```bash
cd openarb-trade-sdk/proto/gen
cargo run
```

这会在 `src/swqos/pb/serverpb.rs` 生成新的代码。

## 技术细节

生成工具使用 `tonic-prost-build` crate：
- 输出目录: `src/swqos/pb`
- Proto 文件: `proto/blockrazor.proto`
- 生成工具: `proto/gen/`
- 包含完整的客户端和服务端代码
