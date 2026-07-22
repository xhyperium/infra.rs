# Gap Matrix — .cargo/draft → infra.rs (2026-07-22)

> **权威快照**：storage×7 OBJECTIVE DEFER 已闭合（生产默认就绪）。  
> package stable / crates.io **仍未**宣称。

| Domain | Draft DoD P0 | Current | Deferred (not OBJECTIVE / not stable) |
|--------|--------------|---------|----------------------------------------|
| redisx | Pool+KV+timeout+live | **done** + Cluster/Sentinel/TLS/resiliencx (`0.3.2`) | Streams full / pubsub 默认关 / package stable |
| postgresx | Pool+query+tx+TLS | **done** + Repository + remote Require 实现 + deadline/隔离证据 (`0.3.3`) | 真实 TLS 实验 / 自定义 CA/mTLS / COPY / migrations / read-replica |
| kafkax | Producer pool + EventBus | **done** + checkpoint/ALO + TLS/CA/PLAIN 证据 (`0.3.2`) | SCRAM/OAuth/mTLS / group/rebalance / native EOS / DLQ |
| natsx | Core NATS EventBus | **done** + JetStream durable pull/显式确认 (`0.3.2`) | 同客户端自动恢复 NO-GO / NKey / Cluster/HA / 自动 DLQ / KV·Object 全量 |
| ossx | ObjectStore put/get | **done** + multipart + resiliencx retry (`0.3.1`) | lifecycle / STS |
| clickhousex | Analytics insert+select | **done** + HTTPS/PEM CA + insert_batch + 有界池 (`0.3.2`) | 真实集群 TLS / mTLS / native 9000 / cluster 运维 |
| taosx | TimeSeries write+query | **done** + batch write + Native WS 探测 + 有界池 (`0.3.1`) | 完整 WS SQL 会话 / 超表治理 |
| goalctl | Goal→Contract digest | **done** | full multi-module authority plane |
| verifyctl | plan+execute+run-result | **done** | full V0–V3 gate matrix |

Freeze: production-default path per domain; scaffold behind `scaffold` feature; no secrets in git; **no** package-stable claim.

## Live 入口

```bash
scripts/live/export-foundationx-env.sh --env dev -- \
  cargo test -p redisx -p postgresx -p kafkax -p natsx \
    -p ossx -p clickhousex -p taosx -- --ignored
```

## 相关 PR

| PR | 说明 |
|----|------|
| #188–#191 | storage 生产客户端 + live 凭据 |
| #195 | storage×7 SSOT layers |
| #211 | storage×7 OBJECTIVE DEFER 闭合 → `0.3.1`/`0.3.2` (redis/postgres) |
| #219 | redisx/postgresx SSOT version 行对齐 `0.3.2` |
