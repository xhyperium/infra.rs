# Gap Matrix — .cargo/draft → infra.rs (2026-07-22)

| Domain | Draft DoD P0 | Current | Target this wave | Deferred (not stable) |
|--------|--------------|---------|------------------|------------------------|
| redisx | Pool+KV+timeout+live | production RedisPool + Cluster/Sentinel/TLS/resiliencx | **done** DEFER closed | Streams full |
| postgresx | Pool+query+tx | production PostgresPool + PgRepository + SSL require + resiliencx | **done** DEFER closed | COPY/migrations |
| kafkax | Producer+EventBus | AMO + 单 owner ALO + 非原子 produce/checkpoint | **done** broker conformance PASS | TLS/group/rebalance/native EOS/DLQ |
| natsx | Core EventBus | Core AMO + JetStream durable pull/显式确认 | **done** broker conformance PASS | NKey/Cluster/HA/自动 DLQ/JS KV full |
| ossx | ObjectStore | production OssClient + multipart + retry | **done** DEFER closed | lifecycle/STS |
| clickhousex | Analytics | production HTTP + insert_batch + 有界池 | **done** DEFER closed | native/cluster |
| taosx | TimeSeries | production REST + batch + Native WS 探测 + 有界池 | **done** DEFER closed | full WS SQL session |
| goalctl | Goal→Contract | tools/goalctl member | **done** | full authority plane |
| verifyctl | plan+execute | tools/verifyctl member | **done** | full V0–V3 matrix |

## Evidence anchors

| 项 | 位置 / 命令 |
|----|-------------|
| 对齐总览 | [workspace-ssot-alignment.md](./workspace-ssot-alignment.md) |
| adapters | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| live env | `scripts/live/build-foundationx-env.mjs` |
| offline | `cargo test -p redisx -p postgresx -p kafkax -p natsx -p ossx -p clickhousex -p taosx --all-targets` |
