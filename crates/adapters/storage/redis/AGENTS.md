# redisx

- **生产默认**：`RedisPool` / `RedisClient` 实现 `contracts::KeyValueStore`
- 扩展：`delete` / `exists` / `expire` / `ttl` / `mget` / `mset`
- feature `pubsub`：`RedisPubSub` / `RedisPubSubFacade`
- feature `scaffold`：旧 `RedisAdapter` / `InMemoryRedis` / `MockRedisAdapter`
- 依赖 workspace `redis`（`tokio-comp` + `connection-manager`）
- 禁止提交密码；`Debug` 必须脱敏
- TTL `Some(0)` → `Invalid`
