# redisx

- **Package / lib / 当前版本**：`redisx` / `redisx` / `0.3.6`（未发布候选；`0.3.3` 为 main 历史）
- **生产默认**：`RedisPool` / `RedisClient` 实现 `contracts::KeyValueStore`
- 扩展：`delete` / `exists` / `expire` / `ttl` / `mget` / `mset`
- feature `pubsub`：`RedisPubSub` / `RedisPubSubFacade`
- feature `scaffold`：旧 `RedisAdapter` / `InMemoryRedis` / `MockRedisAdapter`
- 依赖 workspace `redis`（`tokio-comp` + `connection-manager`）
- 禁止提交密码；`Debug` 必须脱敏
- TTL `Some(0)` → `Invalid`
- budget 路由：GET/EXISTS/PTTL/MGET=`ReadOnly`；无 TTL SET/MSET=`Idempotent`；相对 TTL SET、
  DEL、PEXPIRE=`UnsafeSideEffect` 且多试前拒绝；PUBLISH 不自动重试
- `RedisOperation::Set` 因无 TTL 参数保持 `AmbiguousWrite`；client 按实际参数细分
