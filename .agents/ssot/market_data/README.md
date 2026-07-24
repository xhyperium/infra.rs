# SSOT — 域规格单一事实源

本仓 SSOT（Single Source of Truth）目录，包含本仓库各领域模块的目标契约、当前实现状态、外部协议证据和可执行门禁。横向规则见 [`CONTRACT.md`](CONTRACT.md)。

draft 输入与主题覆盖关系见 [`draft-coverage.md`](draft-coverage.md)。
最终全仓十轮复审见 [`review-round-21-30-final.md`](review-round-21-30-final.md)。
适配器源码审计见 [`review-round-31-40-adapters.md`](review-round-31-40-adapters.md)。

## 域清单

| 域 | 路径 | 说明 |
|----|------|------|
| domainx | `.agents/ssot/domainx/` | 领域共享值对象 |
| domain_market | `.agents/ssot/domain_market/` | 市场数据域模型 |
| domain_exchange | `.agents/ssot/domain_exchange/` | 交易域模型 |
| binance | `.agents/ssot/binance/` | 币安 Binance Market Data Adapter |
| okx | `.agents/ssot/okx/` | OKX Exchange Market Data Adapter |
| coinbase | `.agents/ssot/coinbase/` | Coinbase Exchange Market Data Adapter |
| hyperliquid | `.agents/ssot/hyperliquid/` | Hyperliquid Exchange Market Data Adapter |
| coinglass | `.agents/ssot/coinglass/` | Coinglass Crypto Market Data Adapter |
| orderbook | `.agents/ssot/orderbook/` | 通用订单簿内核与物化引擎（当前尚无对应 runtime crate） |
| market_data | `crates/market_data/docs/` | L0 兼容 facade（不重复定义域模型） |

## 规范

- 各主题规格以 `spec/spec.md` 为主题 SSOT；跨主题冲突由 `CONTRACT.md` 裁决。
- `design/`、`goal/`、`evidence/`、`matrix/`、`review/` 分别记录设计、目标、外部事实、追溯和审查状态。
- `specified`、`skeleton`、`pending` 不代表运行时通过；只有有固定证据的 `verified` 才能关闭门禁。
- 规格变更走 worktree + PR 流程，并在同一提交中更新证据/矩阵/复审决议。
