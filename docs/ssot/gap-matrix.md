# Gap Matrix — .cargo/draft → infra.rs (2026-07-22)

> **权威快照**：P0 目标状态以 [draft-gap-matrix.md](./draft-gap-matrix.md) 为准（#188–#191 后 **done**）。  
> 本文件保留「历史规划列」便于对照；**Current = 落地后现状**。

| Domain | Draft DoD P0 | Current（main @ #191） | Deferred (not stable) |
|--------|--------------|------------------------|------------------------|
| redisx | Pool+KV+timeout+live | **done** — `RedisPool`/`RedisClient` 默认生产 + `FOUNDATIONX_REDISX_*` + live/bench | Cluster/Sentinel/Streams full |
| postgresx | Pool+query+tx+TLS | **done** — `PostgresPool`/`PgTransaction` + SQLSTATE + live/bench | COPY/migrations/read-replica |
| kafkax | Producer pool + EventBus | **done** — `KafkaPool`/`Producer`/`Consumer` + SASL + live/bench（有界） | EOS/tx producer, schema registry |
| natsx | Core NATS EventBus | **done** — `NatsPool` + EventBus + live/bench（有界） | JetStream full surface |
| ossx | ObjectStore put/get | **done** — `OssClient` OSS V1 + live/bench | multipart/lifecycle |
| clickhousex | Analytics insert+select | **done** — HTTP `ClickHousePool` + live/bench（有界） | native protocol, cluster |
| taosx | TimeSeries write+query | **done** — REST `TaosPool`（6041）+ live/bench（有界） | native WS full |
| goalctl | Goal→Contract digest | **done** — `tools/goalctl` member CLI | full multi-module authority plane |
| verifyctl | plan+execute+run-result | **done** — `tools/verifyctl` member CLI | full V0–V3 gate matrix |
| verification | cargo verify + evidence | **done（最小）** — verifyctl + evidence 自验证路径 | archgate OOS |

Freeze: P0 production default path per domain; scaffold behind `scaffold` feature; no secrets in git.

## Live 入口

```bash
node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
set -a; source /tmp/foundationx-live.env; set +a
cargo test -p redisx -p postgresx -p kafkax -p natsx -p ossx -p clickhousex -p taosx -- --ignored
```

## 相关 PR

| PR | 说明 |
|----|------|
| #188 | storage 生产客户端 + goalctl/verifyctl members |
| #189 | evidence-grade docs / pool unit tests / core benches |
| #190 | bench 超时 + 公共 API 单测缺口 |
| #191 | `build-foundationx-env.mjs` live 凭据导出 |
