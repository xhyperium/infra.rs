# okx 交易所适配器规范

> 状态：当前 mock 实现说明，不批准真实交易。权威顺序为 `CONSTITUTION.md` → canonical spec → Approved ADR → 本文 → 代码。

## 1. 边界、范围与证据

- **Evidence**：`crates/adapters/exchange/okx` 的 `MockOkxAdapter` 实现 `VenueAdapter` 全部 13 方法且无网络 IO。
- **Inference**：真实实现应保留 contracts DTO 边界并隐藏于同一 trait 后。
- **Unknown**：OKX 认证/账户模式、限频、重连、时间同步、订单簿恢复、错误码、配置、benchmark 与 `mock` feature 未裁定。

目的：记录当前 OKX mock；非目标：批准真实下单、网络协议或生产可用性。

## 2. 位置、依赖、版本

路径 `crates/adapters/exchange/okx`，版本 `0.1.0`，无 features。依赖为 `xlib_standard`、`contracts`、`canonical`、`decimalx`、`async-trait`、`futures-core`、`futures-util`；dev 为 `tokio`。符合 R2.1，当前 mock 不需要 L1；缺 canonical spec 要求的 `mock` feature。独立版本每次必须恰为 `x.y.z → x.y.(z+1)`。

## 3. 当前 API 与行为

`pub struct MockOkxAdapter`（unit struct，`Debug + Default`）及 `new()`。connect/disconnect/cancel 成功；place_order 回显 id、`Pending`、`ts=0`；query_order 为 `Pending`；仓位/余额为空；三种订阅是立即结束的空 stream；server_time 为 0；symbol_info 回显 symbol，base/quote 为空且 tick/min_qty 为零；venue_id 为 `"okx"`。无状态、无网络、无真实账户/订单语义。

## 4. 错误、并发、生命周期与信任

当前不返回错误，实例无可变状态；订阅立即结束。生产边界必须裁定凭据/签名/TLS、输入与账户模式校验、限频、重连/取消、时钟偏差、服务错误到 `XError` 的映射，以及 ADR-004 的 snapshot/delta 恢复；禁止依赖 domain 类型或泄漏敏感服务响应。

## 5. 测试与验收

现有 11 个测试覆盖 13 方法的 mock 结果与 trait object。运行：

```bash
cargo test -p okx
cargo check -p okx --all-targets
cargo clippy -p okx --all-targets -- -D warnings
```

缺失：真实报文 fixture/重放、认证/账户模式、限频与断线恢复、订单簿一致性、热路径 benchmark、mock feature。验收要求第 3 节行为、R2.1、测试/clippy 通过且生产 Unknown 保持显式。

## 6. 开放决策与追溯

追溯 canonical spec §2 R2.1/R6、§4.3、§4.5.2、§5、§8；ADR-001、ADR-004、ADR-007。共享文档已明确当前 OKX 仅为 mock，且 `mock` feature/benchmark 尚缺。
