# Gap Matrix — .cargo/draft → infra.rs (2026-07-22)

| Domain | Draft DoD P0 | Current | Target this PR | Deferred (not stable) |
|--------|--------------|---------|----------------|------------------------|
| redisx | Pool+KV+timeout+live | production RedisPool default | done | Cluster/Sentinel/Streams full |
| postgresx | Pool+query+tx | production PostgresPool | done | COPY/migrations/TLS require |
| kafkax | Producer+EventBus | production KafkaPool | done | EOS/tx, group if coordinator down |
| natsx | Core EventBus | production NatsPool | done | JetStream full |
| ossx | ObjectStore | production OssClient | done | multipart |
| clickhousex | Analytics | production HTTP client | done | native protocol |
| taosx | TimeSeries | production REST | done | native WS full |
| goalctl | Goal→Contract | tools/goalctl member | done | full authority plane |
| verifyctl | plan+execute | tools/verifyctl member | done | full V0–V3 matrix |
