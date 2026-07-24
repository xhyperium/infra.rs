# ENVIRONMENT.md — market_data SSOT 环境信息

## 来源

| 项目 | 值 |
|------|----|
| **外部仓库** | `/home/workspace/market_data.rs` |
| **本仓路径** | `.agents/ssot/market_data/` |
| **导入方式** | `cp -rf` 全文复制 |
| **导入日期** | 2026-07-24 |

## 域分类

| 层级 | 域 | 类型 | 位置 |
|------|----|------|------|
| L1 领域模型 | `domainx` | 共享交易值对象 | `→ core/domainx/` |
| L1 领域模型 | `domain_market` | 市场数据域模型 | `→ core/domain_market/` |
| L1 领域模型 | `domain_exchange` | 交易所抽象 | `→ core/domain_exchange/` |
| L2 适配器 | `binance` | 交易所适配器 | 本目录 |
| L2 适配器 | `okx` | 交易所适配器 | 本目录 |
| L2 适配器 | `coinbase` | 交易所适配器 | 本目录 |
| L2 适配器 | `hyperliquid` | 交易所适配器 | 本目录 |
| L2 适配器 | `coinglass` | 数据源适配器 (REST-only) | 本目录 |
| L2 引擎 | `orderbook` | 通用订单簿内核 | 本目录 |

> 领域规格 (domainx / domain_market / domain_exchange) 已移至 `.agents/ssot/core/`，详见 `core/AGENTS.md`。

## 门禁总览 (适配器层)

| 指标 | 值 |
|------|----|
| 总门禁 (适配器+引擎) | 50 |
| verified | 3 |
| pending | 35 |
| specified | 8 |
| deferred | 4 |

## 与本仓的关系

- market_data SSOT 是 **规格层镜像**，不包含 Rust 实现代码
- 实现路径在 `market_data.rs` 仓库的 `crates/` 下
- 领域模型规格统一在 `core/` 维护
- CONTRACT.md §5 规定了跨域公共语义
