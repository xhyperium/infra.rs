# `.agents/ssot/core/` — 共享领域规格 (Shared Domain Core · L0 类型层)

本目录是**分层共存**架构中的 **L0 类型层**：汇聚跨子系统共享的领域模型规格，定义纯类型与 trait 契约，不含 I/O 和运行时逻辑。

> **分层共存声明**：`core/`（L0 类型层）与 `market_data/`（L2 provider + L1 管线）是两个**独立共存**的类型平面，不是迁移过渡关系。详见 [docs/ssot/core-ssot-alignment.md](../../docs/ssot/core-ssot-alignment.md)。

## 1. 分层共存模型

```text
┌─────────────────────────────────────────────────────────────┐
│  L0 类型层（本目录）                                        │
│  domainx → domain_market → domain_exchange                  │
│  domain_macro                                                │
│  （纯类型 + 验证 + trait 契约，零 I/O）                      │
└──────▲──────────────────────────▲─────────────────────┐    │
       │                          │                     │    │
┌──────┴───────────┐   ┌─────────┴──────────┐           │    │
│ L1 行情管线       │   │ L2 Exchange         │           │    │
│ market_data       │   │   Provider          │           │    │
│ (MarketTick/      │   │ exchange/{binance,  │           │    │
│  InstrumentType)  │   │  okx,coinbase,      │           │    │
│ 独立类型平面       │   │  hyperliquid,       │           │    │
│                   │   │  coinglass}         │           │    │
└───────────────────┘   │ 实现 VenueAdapter   │           │    │
                        └────────────────────┘           │    │
                                          ┌──────────────┴────┤
                                          │ L2' Infra Adapter │
                                          │ adapters/exchange │
                                          │ 实现 contracts::  │
                                          │ Exchange trait    │
                                          │ （独立 trait 层次）│
                                          └───────────────────┘
```

**关键边界**：
- `domain_exchange::VenueAdapter`（L2 provider trait）≠ `contracts::Exchange`（L2' infra adapter trait）——两套独立契约，**无桥接层**
- `market_data`（L1）与 `domain_market`（L0）是独立类型平面，当前无依赖关系
- `exchange/*`（L2）依赖 `core/domain_*`；`adapters/exchange/*`（L2'）依赖 `kernel`/`canonical`/`contracts`/`transportx`

## 2. 本层域树

| 路径 | crate | 角色 | 落地 |
|------|-------|------|------|
| `domainx/` | `crates/domainx` | 共享交易值对象：Order、Position、Trade、Portfolio + 验证 | 已实现 |
| `domain_market/` | `crates/domain_market` | 行情域模型：Tick、Quote、Bar、OrderBook + 时间/簿验证 | 已实现 |
| `domain_exchange/` | `crates/domain_exchange` | 交易所抽象：VenueAdapter trait、StreamType、AdapterError | 已实现 |
| `domain_macro/` | — | 宏观经济共享模型：Period、Vintage、RevisionChain | 规格 draft，crate 未落地 |

## 3. 标准文档结构

```text
goal/goal.md       # 目标：为什么需要、解决什么问题
design/design.md   # 设计：架构决策（ADR）、权衡
spec/spec.md       # 规格：API 契约、类型约束
review/            # 复审记录：逐轮评审结论
evidence/          # 验证证据
matrix/            # 追溯矩阵（门禁 → 实现 → 测试）
```

**Code 不在本树**：实现路径在 `crates/` 下，禁止在 SSOT 目录写实现代码。

## 4. 门禁总览

| 域 | 总门禁 | verified | pending | blocked | 关键阻塞 |
|----|--------|----------|---------|---------|---------|
| `domainx` | 5 | 4 | 0 | 1 | xhyper-canonical 未引入 |
| `domain_market` | 6 | 5 | 0 | 1 | 同上 |
| `domain_exchange` | 6 | 6 | 0 | 0 | adapter HTTP 映射待实现 |
| `domain_macro` | - | - | draft | - | 规格 draft，待落地 |

| **合计** | **17+** | **15** | **0** | **2** |

## 5. 跨域依赖

```
core/domainx ←── core/domain_market
              ←── core/domain_exchange
              ←── exchange/{binance,okx,coinbase,hyperliquid,coinglass}（L2 provider）

core/domain_macro ←── macro_data/{bea,ecb,fred,...}（L2 provider）
                  ←── macro_data/yield_curve
```

- `domainx` 是交易领域共享基础，domain_market、domain_exchange 及 exchange/* provider 均依赖其类型
- `domain_macro` 是宏观领域共享基础，所有 macro_data provider 依赖其模型
- L0 与 L1（`market_data`）之间**无直接依赖**——独立类型平面
- L2（`exchange/*`）与 L2'（`adapters/exchange/*`）之间**无桥接层**——独立 trait 层次

## 6. 变更规则

- 修改域规格必须走 worktree + PR
- 新增域需先创建 goal → design → spec → matrix 四层
- 门禁状态变更需同步更新本文件统计
- 分层共存边界变更（如新增桥接层）须先更新 [docs/ssot/core-ssot-alignment.md](../../docs/ssot/core-ssot-alignment.md) 并经 PR 审查
