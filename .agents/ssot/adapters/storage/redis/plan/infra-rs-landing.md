# infra.rs 本仓落地说明 — redisx

| 字段 | 值 |
|------|-----|
| package | `redisx` |
| 实现路径 | `crates/adapters/storage/redis` |
| 生产默认面 | RedisPool/RedisClient |
| scaffold | `feature = "scaffold"`（可选 mock） |
| live | `tests/live_kv.rs + live_kv_conformance.rs`（默认 `#[ignore]`） |
| 凭据 | `FOUNDATIONX_*` via `scripts/live/build-foundationx-env.mjs` |
| PR | #188 · #189 · #190 · #191 |
| 对齐 | [docs/ssot/adapters-ssot-alignment.md](../../../../../docs/ssot/adapters-ssot-alignment.md) |
| package stable | **未宣称** |

## 硬限制

1. 本文件描述 **infra.rs 本仓 P0 生产入口**，不是 monorepo 战役 COMPLETE。
2. Cluster / Sentinel / TLS 命令代码路径不等于 live PASS；真实证据保持 **OPEN**。
3. 无 live 证据不得宣称“全后端 Production Ready”。
4. Pub/Sub 仅 Standalone，必须复用池配置；拓扑 Pub/Sub 与重连 **NO-GO**。
5. 自动重试仅限只读；写操作默认单次，响应丢失时结果可能未知。

## 验证

```bash
cargo test -p redisx --all-targets
cargo test -p redisx --all-targets --features pubsub
# live:
# node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
# set -a; source /tmp/foundationx-live.env; set +a
# cargo test -p redisx -- --ignored
```
