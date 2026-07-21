# binancex

binance exchange adapter

Package：`binancex` · path：`crates/adapters/exchange/binance`

SSOT 镜像：[`.agents/ssot/adapters/exchange/binance`](../../../../.agents/ssot/adapters/exchange/binance/README.md)  
本仓对齐：[docs/adapters-ssot-alignment.md](../../../../docs/adapters-ssot-alignment.md)

## 状态

**scaffold** — 仅错误类型骨架，**未**宣称业务实现 / package stable / 真实 I/O。

## 职责

- exchange 适配器实现位
- 错误类型：`Error` / `Result`

## 非目标（当前）

- 真实网络 / SDK 集成
- 生产交易或生产 I/O
- package stable 宣称

## 最小用法

```rust
use binancex::{Error, Result};

fn demo() -> Result<()> {
    Err(Error::Internal("scaffold only".into()))
}
```

## 相关

- 父级规则：[`crates/AGENTS.md`](../../../AGENTS.md)
- 合约 trait：`xhyper-contracts`（`crates/contracts`）
