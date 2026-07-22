# infra.rs 本仓落地说明 — natsx

| 字段 | 值 |
|------|-----|
| package | `natsx` |
| 实现路径 | `crates/adapters/storage/nats` |
| 生产默认面 | Core NatsPool/EventBus + JetStream durable pull |
| scaffold | `feature = "scaffold"`（可选 mock） |
| live | `tests/live_event_bus.rs`；`tests/broker_conformance.rs`（默认 `#[ignore]`） |
| 凭据 | `FOUNDATIONX_*` via `scripts/live/build-foundationx-env.mjs` |
| PR | #188 · #189 · #190 · #191 |
| 对齐 | [docs/ssot/adapters-ssot-alignment.md](../../../../../docs/ssot/adapters-ssot-alignment.md) |
| package stable | **未宣称** |

## 硬限制

1. 本文件描述 **infra.rs 本仓 P0 生产入口**，不是 monorepo 战役 COMPLETE。
2. Cluster / HA / 跨账户 / 自动 DLQ / KV·ObjectStore 为 **NO-GO/DEFER**。
3. 无 live 证据不得宣称“全后端 Production Ready”。

## 验证

```bash
cargo test -p natsx --all-targets
node scripts/broker-conformance.mjs
# live:
# node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
# set -a; source /tmp/foundationx-live.env; set +a
# cargo test -p natsx -- --ignored
```
