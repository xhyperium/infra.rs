# Gap Matrix — .cargo/draft → infra.rs (2026-07-22)

| Domain | Draft DoD P0 | Current | Target this wave | Deferred (not stable) |
|--------|--------------|---------|------------------|------------------------|
| redisx | Pool+KV+timeout+live | production RedisPool + Cluster/Sentinel/TLS/resiliencx | **done** DEFER closed | Streams full |
| postgresx | Pool+query+tx | PostgresPool + Repository + TLS + deadline/隔离 | **done** fixed-image PASS | 自定义 CA/mTLS/COPY/migrations |
| kafkax | Producer+EventBus | AMO/ALO + TLS/CA/PLAIN | **done** broker + SASL_SSL PASS | SCRAM/OAuth/mTLS/group/rebalance/native EOS/DLQ |
| natsx | Core EventBus | Core AMO + JetStream durable pull/显式确认 | **partial** broker PASS；同客户端恢复 FAIL | 自动恢复/NKey/Cluster/HA/自动 DLQ/JS KV full |
| ossx | ObjectStore | production OssClient + multipart + retry | **done** DEFER closed | lifecycle/STS |
| clickhousex | Analytics | HTTP(S)+PEM CA+insert_batch+有界池 | **done** TLS client PASS | 真实集群 TLS/native/cluster |
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
