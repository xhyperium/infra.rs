# Gap Matrix — .cargo/draft → infra.rs (2026-07-22)

| Domain | Draft DoD P0 | Current | Target this wave | Deferred (not stable) |
|--------|--------------|---------|------------------|------------------------|
| redisx | Pool+KV+timeout+live | production RedisPool default | **done** + live 真凭据验 | Cluster/Sentinel/Streams full |
| postgresx | Pool+query+tx | production PostgresPool | **done** + live 真凭据验 | COPY/migrations/TLS require |
| kafkax | Producer+EventBus | production KafkaPool | **done** + live + 有界 bench | EOS/tx, group if coordinator down |
| natsx | Core EventBus | production NatsPool | **done** + live + 有界 bench | JetStream full |
| ossx | ObjectStore | production OssClient | **done** + live 真凭据验 | multipart |
| clickhousex | Analytics | production HTTP client | **done** + live + 有界 bench | native protocol |
| taosx | TimeSeries | production REST | **done** + live（6041）+ 有界 bench | native WS full |
| goalctl | Goal→Contract | tools/goalctl member | **done** | full authority plane |
| verifyctl | plan+execute | tools/verifyctl member | **done** | full V0–V3 matrix |

## Evidence anchors

| 项 | 位置 / 命令 |
|----|-------------|
| PR | #188 · #189 · #190 · #191 |
| 对齐总览 | [workspace-ssot-alignment.md](./workspace-ssot-alignment.md) |
| adapters | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| tools | [tools-ssot-alignment.md](./tools-ssot-alignment.md) |
| live env | `scripts/live/build-foundationx-env.mjs` |
