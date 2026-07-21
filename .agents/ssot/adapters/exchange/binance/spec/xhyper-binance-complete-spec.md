# binance 交易所适配器规范

> 状态：当前 mock 实现说明，不批准真实交易。权威顺序为 `CONSTITUTION.md` → canonical spec → Approved ADR → 本文 → 代码。

## 1. 边界、范围与证据

- **Evidence**：`crates/adapters/exchange/binance` 的 `MockBinanceAdapter` 实现 `VenueAdapter` 全部 13 方法，且不执行网络 IO。
- **Inference**：真实 REST/WebSocket 实现应隐藏在同一 trait 后并按 ADR-008 从 quant 迁移，而非复制两套权威实现。
- **Unknown**：认证、限频、重连、时间同步、订单簿恢复、错误码、配置、benchmark 与 `mock` feature 未裁定。

目的：记录当前 Binance mock。非目标：批准真实下单、报文解析、网络依赖或迁移完成状态。

## 2. 位置、依赖、版本

路径 `crates/adapters/exchange/binance`，版本 `0.1.0`，无 features。依赖：`xlib_standard`、`contracts`、`canonical`、`decimalx`、`async-trait`、`futures-core`、`futures-util`；dev 为 `tokio`。符合 R2.1，当前无需 L1；缺 canonical spec 要求的 `mock` feature。独立版本更新必须恰为 `x.y.z → x.y.(z+1)`。

## 3. 当前 API 与精确行为

`pub struct MockBinanceAdapter`（unit struct，`Debug + Default`）及 `new()`。`VenueAdapter` 行为：connect/disconnect/cancel 成功；place_order 回显 id、`Pending`、`ts=0`；query_order 为 `Pending`；仓位/余额为空；三种订阅返回立即结束的空 stream；server_time 为 0；symbol_info 回显 symbol，其 base/quote 为空、tick/min_qty 为零；venue_id 为 `"binance"`。无状态、无网络、无真实撮合语义。

## 4. 错误、并发、生命周期与信任

当前方法均不产生 `XError`，实例无可变状态且可 `Send + Sync` 使用；订阅 stream 创建后即结束。生产实现必须裁定密钥隔离、签名、TLS、输入校验、限频、重连/取消、时间偏差、服务错误映射及订单簿 snapshot/delta 恢复；不得依赖 domain 类型。ADR-004 的 order-book 策略在真实流实现前仍是架构约束，不由本 mock 补全。

## 5. 测试与验收

现有 11 个测试覆盖 13 方法的 mock 结果及 trait object。运行：

```bash
cargo test -p binance
cargo check -p binance --all-targets
cargo clippy -p binance --all-targets -- -D warnings
```

缺失：真实报文 fixture/重放、认证错误、限频重试、断线恢复、订单簿一致性、热路径 benchmark、mock feature。验收要求第 3 节行为、R2.1、测试/clippy 均通过，且不得宣称生产就绪。

## 6. 开放决策与追溯

追溯 canonical spec §2 R2.1/R6、§4.3 `VenueAdapter`、§4.5.2、§5、§8；ADR-001、ADR-004、ADR-007、ADR-008。共享文档已明确当前 exchange crate 是 mock 且 `mock` feature/benchmark 尚缺；quant Binance 实现的迁移完成条件仍按 ADR-008 评审。
