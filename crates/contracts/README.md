# contracts

R4 跨层 trait 出口（Additive Only）。Package `contracts` / lib `contracts`。

依赖白名单：kernel + canonical + async-trait / bytes / futures-core。

Active Spec：`.agents/ssot/contracts/spec/spec.md`

## 生产入口

- **推荐**：`ExecutionVenue`（结构化 cancel/query，无 additive default）
- **迁移 facade**：`VenueAdapter`（`cancel_order_request` / `query_order_request` 有中文 Invalid default；树内 adapter 必须覆盖）

## Live helper 边界

- `LiveContractProfile` 只是接线意图，不是 readiness attestation。
- `LiveHandles::validate` 只校验当前句柄形状；repo/account/venue_time 无对应句柄时 fail-closed。
- `publish_without_delivery_attestation` 不证明订阅/确认/E2E 交付。
- `kv_set_then_commit_separate_resources` 明确 KV 与事务上下文不是同一原子资源。

## contract-testkit（独立 crate）

Fake / Recording / per-trait suite **不在**本 crate：

| crate | path | 用途 |
|-------|------|------|
| `contract-testkit` | `crates/test-support/contracts` | Fake + `assert_*` suite（仅 **dev-dep**） |

```toml
[dev-dependencies]
contract-testkit = { path = "../test-support/contracts", version = "0.1.0" }
```

```rust
use contract_testkit::{FakeKeyValueStore, assert_key_value_store};
```

本 crate 仅保留 VenueAdapter 门禁辅助：`VENUE_*_DEFAULT_MSG` / `is_default_*`。

## 文档

- 语义合同：[`docs/contracts/`](./docs/contracts/)
- SSOT 对齐：[`docs/ssot/contracts-ssot-alignment.md`](../../docs/ssot/contracts-ssot-alignment.md)

**非**整体 Production Ready（真实后端见后续工作项）。

当前版本：`0.1.2`。

## 生产误用红线

| 禁止 | 原因 |
|------|------|
| 宣称 L3 Contract Ready | 缺 **非 scaffold** 真实后端验证入口（W4） |
| 把 Fake/*Adapter scaffold 当生产客户端 | 进程内内存，无真实 DB/MQ/交易所 |
| Agent 自签 Production Ready | 仅 Maintainer L5（`prod-signoff-TEMPLATE.md`） |

示例：`cargo run -p contracts --example fake_surface`（**仅 Fake 形状**，dev-dep）
