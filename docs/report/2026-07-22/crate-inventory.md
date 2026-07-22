# crates/ 包清单与 SSOT 映射 — 2026-07-22

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-22 |
| 来源 | `cargo metadata --no-deps` + `.agents/ssot/**` + `docs/ssot/*-ssot-alignment.md` |
| 主体 | **22** 个 `crates/**` workspace package（不含 tools） |

## 1. 映射总表

| Package | 路径 | 平面 | SSOT | 对齐文档 | 成熟度 | Spec Σ | 证据层 |
|---------|------|------|------|----------|--------|--------|--------|
| `kernel` | `crates/kernel` | L0 | `.agents/ssot/kernel/` | [`kernel-ssot-alignment.md`](../../docs/ssot/kernel-ssot-alignment.md) | active | 34/35 | L1+L4 |
| `testkit` | `crates/testkit` | T0 | `.agents/ssot/testkit/` | [`testkit-ssot-alignment.md`](../../docs/ssot/testkit-ssot-alignment.md) | active | 33/35 | L1 test-support |
| `decimalx` | `crates/types/decimal` | types | `.agents/ssot/types/decimal/` | [`types-ssot-alignment.md`](../../docs/ssot/types-ssot-alignment.md) | active | 33/35 | L1 |
| `canonical` | `crates/types/canonical` | types | `.agents/ssot/types/canonical/` | [`types-ssot-alignment.md`](../../docs/ssot/types-ssot-alignment.md) | active | 33/35 | L2 subset v1–v1.3 |
| `bootstrap` | `crates/bootstrap` | L1 | `.agents/ssot/bootstrap/` | [`bootstrap-ssot-alignment.md`](../../docs/ssot/bootstrap-ssot-alignment.md) | active | 32/35 | L1 有条件 |
| `configx` | `crates/configx` | L1 | `.agents/ssot/configx/` | [`configx-ssot-alignment.md`](../../docs/ssot/configx-ssot-alignment.md) | active | 32/35 | L1 内存合同 |
| `schedulex` | `crates/schedulex` | L1 | `.agents/ssot/schedulex/` | [`schedulex-ssot-alignment.md`](../../docs/ssot/schedulex-ssot-alignment.md) | active | 32/35 | L1 registry |
| `evidence` | `crates/evidence` | L1 | `.agents/ssot/tools/evidence/` | [`evidence-ssot-alignment.md`](../../docs/ssot/evidence-ssot-alignment.md) | active | 25/35 | L1 append |
| `observex` | `crates/observex` | L1 | `.agents/ssot/observex/` | [`observex-ssot-alignment.md`](../../docs/ssot/observex-ssot-alignment.md) | active | 32/35 | L1 + L3 Instr 入口 |
| `resiliencx` | `crates/resiliencx` | L1 | `.agents/ssot/resiliencx/` | [`resiliencx-ssot-alignment.md`](../../docs/ssot/resiliencx-ssot-alignment.md) | active | 31/35 | 接近 L1 Internal |
| `transportx` | `crates/transport` | L1 | `.agents/ssot/transport/` | [`transport-ssot-alignment.md`](../../docs/ssot/transport-ssot-alignment.md) | active | 32/35 | L1 有条件 I/O |
| `contracts` | `crates/contracts` | contracts | `.agents/ssot/contracts/` | [`contracts-ssot-alignment.md`](../../docs/ssot/contracts-ssot-alignment.md) | active | 33/35 | L3 子集 KV+Instr |
| `contract-testkit` | `crates/test-support/contracts` | T0 | `.agents/ssot/testkit/ §3.2 + contracts` | [`testkit-ssot-alignment.md`](../../docs/ssot/testkit-ssot-alignment.md) | active | 29/35 | L1 test-support |
| `binancex` | `crates/adapters/exchange/binance` | adapter | `.agents/ssot/adapters/exchange/binance/` | [`adapters-ssot-alignment.md`](../../docs/ssot/adapters-ssot-alignment.md) | scaffold+mock | 24/35 | scaffold + server_time |
| `okxx` | `crates/adapters/exchange/okx` | adapter | `.agents/ssot/adapters/exchange/okx/` | [`adapters-ssot-alignment.md`](../../docs/ssot/adapters-ssot-alignment.md) | scaffold+mock | 24/35 | scaffold + server_time |
| `redisx` | `crates/adapters/storage/redis` | adapter | `.agents/ssot/adapters/storage/redis/` | [`redisx-ssot-alignment.md`](../../docs/ssot/redisx-ssot-alignment.md) | active | 33/35 | L1 + L3-KV 入口 |
| `postgresx` | `crates/adapters/storage/postgres` | adapter | `.agents/ssot/adapters/storage/postgres/` | [`postgresx-ssot-alignment.md`](../../docs/ssot/postgresx-ssot-alignment.md) | active | 33/35 | L1 池+Tx |
| `kafkax` | `crates/adapters/storage/kafka` | adapter | `.agents/ssot/adapters/storage/kafka/` | [`kafkax-ssot-alignment.md`](../../docs/ssot/kafkax-ssot-alignment.md) | active | 31/35 | L1 AMO EventBus |
| `natsx` | `crates/adapters/storage/nats` | adapter | `.agents/ssot/adapters/storage/nats/` | [`natsx-ssot-alignment.md`](../../docs/ssot/natsx-ssot-alignment.md) | active | 31/35 | L1 Core NATS |
| `ossx` | `crates/adapters/storage/oss` | adapter | `.agents/ssot/adapters/storage/oss/` | [`ossx-ssot-alignment.md`](../../docs/ssot/ossx-ssot-alignment.md) | active | 28/35 | L1 ObjectStore |
| `clickhousex` | `crates/adapters/storage/clickhouse` | adapter | `.agents/ssot/adapters/storage/clickhouse/` | [`clickhousex-ssot-alignment.md`](../../docs/ssot/clickhousex-ssot-alignment.md) | active | 27/35 | L1 HTTP 部分 |
| `taosx` | `crates/adapters/storage/taos` | adapter | `.agents/ssot/adapters/storage/taos/` | [`taosx-ssot-alignment.md`](../../docs/ssot/taosx-ssot-alignment.md) | active | 27/35 | L1 REST 部分 |

## 2. 覆盖校验

- workspace `crates/**` 成员数：**22**
- 本表行数：**22**
- 对齐文档：每包至少一条 `docs/ssot/*-ssot-alignment.md`（decimalx/canonical 共享 `types-ssot-alignment.md`；binancex/okxx 共享 `adapters-ssot-alignment.md` + 分包对齐；contract-testkit 挂 testkit+contracts）
- evidence SSOT 物理位置：`.agents/ssot/tools/evidence/`（非 `.agents/ssot/evidence/`）

## 3. 缺失说明

| 项 | 状态 |
|----|------|
| contract-testkit 独立 SSOT 根 | **无**（设计挂靠 testkit §3.2）— S1 仍计 5（规范存在） |
| gate / testkitx / archgate | 本仓 **OOS / 非 member**，不入 22 表 |
| tools/goalctl · verifyctl | 非 `crates/` 主体 |

## 4. 证据命令

```bash
cargo metadata --no-deps --format-version 1
ls docs/ssot/*alignment*.md
find .agents/ssot -maxdepth 3 -type d
```
