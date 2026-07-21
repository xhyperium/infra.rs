# Gap Matrix — .cargo/draft → infra.rs (2026-07-22)

| Domain | Draft DoD P0 | Current | Target this PR | Deferred (not stable) |
|--------|--------------|---------|----------------|------------------------|
| redisx | Pool+KV+timeout+live | scaffold + optional RedisLiveKv | RedisPool/Client default prod, env config, KV+ext, close, live+bench | Cluster/Sentinel/Streams full |
| postgresx | Pool+query+tx+TLS | scaffold/mock only | PostgresPool acquire/query/tx, env, live+bench | COPY/migrations/read-replica |
| kafkax | Producer pool + EventBus | scaffold/mock | KafkaPool producer+consumer, SASL, EventBus, live+bench | EOS/tx producer, schema registry |
| natsx | Core NATS EventBus | scaffold/mock | NatsPool connect/pub/sub, auth, live+bench | JetStream full surface |
| ossx | ObjectStore put/get | scaffold | OssClient S3-compat Aliyun, env, live+bench | multipart/lifecycle |
| clickhousex | Analytics insert+select | scaffold | ClickHouseClient HTTP, env, live+bench | native protocol, cluster |
| taosx | TimeSeries write+query | scaffold | TaosClient REST, env, live+bench | native WS full |
| goalctl | Goal→Contract digest | tools/ untracked incomplete | workspace member CLI compile/validate | full multi-module authority plane |
| verifyctl | plan+execute+run-result | missing | workspace member CLI plan/execute | full V0–V3 gate matrix |
| verification | cargo verify + evidence | partial evidence crate | verifyctl+evidence self-verify path | archgate OOS |

Freeze: P0 production default path per domain; scaffold behind `scaffold` feature; no secrets in git.
