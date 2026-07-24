# ENVIRONMENT.md — market_data SSOT 环境信息

## 来源

| 项目 | 值 |
|------|----|
| **外部仓库** | `/home/workspace/market_data.rs` |
| **本仓路径** | `.agents/ssot/market_data/` |
| **导入方式** | `cp -rf` 全文复制 |
| **导入日期** | 2026-07-24 |

## 域分类

| 层级 | 域 | 类型 | 状态 |
|------|----|------|------|
| L1 领域模型 | `domainx` | 共享交易值对象 | 5 门禁, 4 verified |
| L1 领域模型 | `domain_market` | 市场数据域模型 | 6 门禁, 5 verified |
| L1 领域模型 | `domain_exchange` | 交易所抽象 | 6 门禁, 6 verified |
| L2 适配器 | `binance` | 交易所适配器 | 7 门禁, 1 verified |
| L2 适配器 | `okx` | 交易所适配器 | 7 门禁 |
| L2 适配器 | `coinbase` | 交易所适配器 | 7 门禁, 1 verified |
| L2 适配器 | `hyperliquid` | 交易所适配器 | 8 门禁 |
| L2 适配器 | `coinglass` | 数据源适配器 (REST-only) | 8 门禁, 1 verified |
| L2 引擎 | `orderbook` | 通用订单簿内核 | 13 门禁 (无 runtime) |

## 门禁总览

| 指标 | 值 |
|------|----|
| 总门禁 | 67 |
| verified | 18 |
| pending | 35 |
| specified | 8 |
| blocked | 2 |
| deferred | 4 |

## 跨域阻塞项

1. **`xhyper-canonical` 未引入** — `domainx` 的 instrument 仍用 `String` 占位，`domain_market` 的 `InstrumentKey` 不是 canonical owner
2. **exchange adapter runtime 空实现** — `binance`/`okx`/`coinbase` 的 `VenueAdapter` 仅 mock 级通过
3. **orderbook 无 runtime crate** — 13 门禁全部 pending/deferred

## 关键文件

| 文件 | 用途 |
|------|------|
| `CONTRACT.md` | 十个主题横向治理基线（v0.3.0，契约冻结） |
| `AGENTS.md` | Agent 工作指引、门禁统计、变更规则 |
| `draft-coverage.md` | `.cargo/draft` 到 SSOT 覆盖审查 |

## 与本仓的关系

- market_data SSOT 是 **规格层镜像**，不包含 Rust 实现代码
- 实现路径在 `market_data.rs` 仓库的 `crates/` 下
- 跨域契约引用 `infra.rs` 的 `kernel`（Timestamp/Error）和 `contracts`（trait 出口）
- CONTRACT.md §5 规定了 Timestamp = Unix 毫秒 i64、Decimal = 十进制字符串 等公共语义
