# Core SSOT 对齐矩阵（分层共存）

| 字段 | 值 |
|------|-----|
| 审计日期 | 2026-07-24 |
| SSOT 路径 | `.agents/ssot/core/{domainx,domain_market,domain_exchange,domain_macro}/` |
| 对齐总览 | [workspace-ssot-alignment.md](./workspace-ssot-alignment.md) |
| 架构模式 | **分层共存**（L0 类型层 / L1 管线 / L2 provider / L2' infra adapter） |

## 1. 分层共存模型

本仓库存在四层独立的领域平面，各层有自己的类型系统和 trait 契约，**通过 path 依赖单向连接，无循环、无桥接层**。

### 1.1 层次定义

| 层 | 职责 | crate | SSOT | trait 契约 |
|----|------|-------|------|-----------|
| **L0 类型层** | 纯领域类型 + 验证 + trait 抽象，零 I/O | `domainx` `domain_market` `domain_exchange` | `core/` | `VenueAdapter` |
| **L1 行情管线** | 标准化行情模型 + 数据处理内核 | `market_data` | `market_data/`（orderbook 引擎） | — |
| **L2 Provider** | 交易所具体实现（VenueAdapter impl） | `exchange/{binance,okx,coinbase,hyperliquid,coinglass}` | `market_data/{<venue>}/` | 实现 `VenueAdapter` |
| **L2' Infra Adapter** | 基础设施适配器（contracts impl） | `adapters/exchange/{binance,okx}` | `adapters/exchange/` | 实现 `contracts::Exchange` |

### 1.2 依赖图

```text
L0 类型层                          L1 管线
┌──────────┐                      ┌──────────────┐
│ domainx  │                      │ market_data  │
└──▲───▲──┘                      └──────────────┘
   │   │                           独立类型平面
 ┌─┴───┴────────┐                     (无 domain_* 依赖)
 │ domain_market│
 └──▲───────────┘
    │
 ┌──┴──────────────┐
 │ domain_exchange │ (定义 VenueAdapter trait)
 └──▲──────────────┘
    │                    │
    │ L2 Provider        │ L2' Infra Adapter
    │                    │
 ┌──┴──────────────┐    ┌┴──────────────────────┐
 │ exchange/*      │    │ adapters/exchange/*   │
 │ 实现 VenueAdapter│   │ 实现 contracts::Exchange│
 └─────────────────┘    └───────────────────────┘
    │ 依赖 domain_*          │ 依赖 kernel/canonical/
    │                        │ contracts/transportx
    └──── 无桥接层 ──────────┘
```

### 1.3 共存规则

1. **VenueAdapter ≠ Exchange**：`domain_exchange::VenueAdapter`（L2 层）与 `contracts::Exchange`（L2' 层）是两套独立 trait，不得混淆或自动派生
2. **market_data ≠ domain_market**：两者是独立类型平面；`market_data` 提供 `MarketTick`/`InstrumentType`，`domain_market` 提供 `Tick`/`Quote`/`Bar`/`OrderBook`/`InstrumentKey`
3. **依赖方向单向**：L2 → L0，L2' → infra；L2 与 L2' 之间无依赖
4. **未来桥接**：如需桥接 L2 与 L2'（如用 `exchange/*` 实现喂给 `contracts::Exchange`），需新建独立桥接 crate，不得在现有层引入双向依赖

## 2. 域矩阵

### 2.1 L0 类型层（core/）

| 域 | crate | package | 版本 | 总门禁 | verified | blocked | 标准布局 | 状态 |
|----|-------|---------|------|--------|----------|---------|----------|------|
| `domainx` | `crates/domainx` | `domainx` | 0.1.0 | 5 | 4 | 1 | 缺 | 已实现（交易值对象 + 验证） |
| `domain_market` | `crates/domain_market` | `domain_market` | 0.1.0 | 6 | 5 | 1 | 缺 | 已实现（行情模型 + 时间/簿验证） |
| `domain_exchange` | `crates/domain_exchange` | `domain_exchange` | 0.1.0 | 6 | 6 | 0 | 缺 | 已实现（VenueAdapter trait） |
| `domain_macro` | — | — | — | — | — | — | — | 规格 draft，crate 未落地 |

**关键阻塞**：`xhyper-canonical` 未引入 workspace，导致 `domainx`/`domain_market` 的 instrument 字段以 `String` 占位。

**标准布局缺口**：三个已落地 crate 当前仅有 `src/` + `tests/`，缺 `docs/` `benches/` `README.md` `review/` `releases/`（跟进中）。

### 2.2 L1 行情管线（market_data kernel）

| crate | package | 版本 | 标准布局 | 状态 |
|-------|---------|------|----------|------|
| `crates/market_data` | `market_data` | 0.1.0 | 齐全 | `MarketTick`/`InstrumentType` 等标准化行情模型已实现 |

### 2.3 L2 Exchange Provider

| crate | package | 版本 | SSOT | 状态 |
|-------|---------|------|------|------|
| `crates/exchange/binance` | `exchange-binance` | 0.1.0 | `market_data/binance/` | 骨架 stub |
| `crates/exchange/okx` | `exchange-okx` | 0.1.0 | `market_data/okx/` | 骨架 stub |
| `crates/exchange/coinbase` | `exchange-coinbase` | 0.1.0 | `market_data/coinbase/` | 骨架 stub |
| `crates/exchange/hyperliquid` | `exchange-hyperliquid` | 0.1.0 | `market_data/hyperliquid/` | 骨架 stub |
| `crates/exchange/coinglass` | `exchange-coinglass` | 0.1.0 | `market_data/coinglass/` | 骨架 stub |

所有 provider 实现 `domain_exchange::VenueAdapter` trait；依赖 `domainx` + `domain_market` + `domain_exchange`。

### 2.4 L2' Infrastructure Adapter（既有）

| crate | package | 版本 | SSOT | 状态 |
|-------|---------|------|------|------|
| `crates/adapters/exchange/binance` | `binancex` | 0.3.2 | `adapters/exchange/binance/` | 签名 REST + 公共 WS；交易 NO-GO |
| `crates/adapters/exchange/okx` | `okxx` | — | `adapters/exchange/okx/` | 签名 REST + 公共 WS；交易 NO-GO |

实现 `contracts::Exchange` trait；依赖 `kernel`/`canonical`/`decimalx`/`contracts`/`transportx`。详见 [adapters-ssot-alignment.md](./adapters-ssot-alignment.md)。

## 3. 与既有架构的关系

### 3.1 core/ 与 infra/ 的关系

| 维度 | core/（L0 类型层） | infra/（L1 平台面） |
|------|-------------------|-------------------|
| 职责 | 领域类型 + trait 抽象 | 平台基础设施（config/schedule/bootstrap/transport…） |
| 依赖 | 零 I/O，仅 serde/chrono/decimal | 依赖 kernel |
| 交叠 | 无 | 无 |

两平面**互不依赖**：`core/domain_*` 不引用 `infra/*`；`infra/*` 不引用 `core/domain_*`。

### 3.2 exchange/* 与 adapters/exchange/* 的关系

| 维度 | exchange/*（L2 Provider） | adapters/exchange/*（L2' Infra Adapter） |
|------|--------------------------|------------------------------------------|
| 实现 trait | `domain_exchange::VenueAdapter` | `contracts::Exchange` |
| 依赖 | domainx/domain_market/domain_exchange | kernel/canonical/contracts/transportx |
| 成熟度 | 骨架 stub | 签名 REST + 公共 WS（生产默认客户端） |
| 用途 | 领域层交易所抽象实现 | 基础设施层 exchange adapter（contracts 出口） |

两套 adapter **并存**，分别服务于不同的 trait 层次。如需统一，须新建桥接层（见 §1.3 规则 4）。

## 4. 验证入口

```bash
# L0 类型层
cargo test -p domainx -p domain_market -p domain_exchange --all-targets

# L1 行情管线
cargo test -p market_data --all-targets

# L2 Provider（骨架，仅编译验证）
cargo check -p exchange-binance -p exchange-okx -p exchange-coinbase \
  -p exchange-hyperliquid -p exchange-coinglass

# L2' Infra Adapter
cargo test -p binancex -p okxx --all-targets
```

## 5. 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-24 | 初始创建：声明分层共存模型；登记 core/market_data/macro_data 三平面；定义 L0–L2' 层次与依赖边界 |
