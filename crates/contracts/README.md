# infra-contracts

适配器合约 trait 出口：`ExchangeAdapter` / `StorageAdapter` 及共享类型。

Package：`infra-contracts` · path：`crates/contracts`  
引入：#43

本仓对齐：[docs/adapters-ssot-alignment.md](../../docs/adapters-ssot-alignment.md)

## 职责

- 定义交易所 / 存储适配器必须实现的 trait
- 共享 `AdapterState` 与合约级 `Error` / `Result`
- **只放 trait / type，不放实现**

## 非目标

- 具体 adapter 实现（见 `crates/adapters/**`）
- 业务校验、domain 状态机
- package stable 宣称

## 最小用法

```rust
use infra_contracts::{AdapterState, exchange::ExchangeAdapter, storage::StorageAdapter};
```

## 已知缺口（OPEN）

- `Ticker` 等字段仍用 `f64` — 与 decimalx 金额禁令冲突，实现阶段须收口
- 标准布局在本 PR 补齐；业务深度测试未宣称
