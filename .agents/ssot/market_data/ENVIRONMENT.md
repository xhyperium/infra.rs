# ENVIRONMENT.md — market_data SSOT 环境信息

## 来源

| 项目 | 值 |
|------|----|
| **本仓 SSOT** | `.agents/ssot/market_data/`（域规格单一事实源；`SSOT.md` R6） |
| **实现路径** | `crates/market_data/`（L1 管线）+ `crates/exchange/{binance,okx,coinbase,hyperliquid,coinglass}/`（L2 provider） |
| **领域模型** | `crates/{domainx,domain_market,domain_exchange}/`（规格在 `.agents/ssot/core/`） |
| **建立日期** | 2026-07-24 |

> market_data 规格是**本仓 SSOT**（非外部仓库镜像）；实现在本仓 `crates/` 下。外仓名字面量（`xhyper` / `market_data.rs`）不得进入本树（`SSOT.md` §5.4）。

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

- market_data SSOT 是**本仓规格 SSOT**（R6），不包含 Rust 实现代码
- 实现路径在**本仓** `crates/` 下（`crates/market_data`、`crates/exchange/*`）
- 领域模型规格统一在 `core/` 维护
- CONTRACT.md §5 规定了跨域公共语义
