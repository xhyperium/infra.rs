# redisx

| 模式 | 类型 | 生产？ |
|------|------|--------|
| 默认 | `RedisAdapter` 内存 HashMap | **否**（scaffold） |
| `live` | `RedisLiveKv` 真实 redis crate | 验证入口（非完整产品） |

```bash
cargo test -p redisx --all-targets
cargo test -p redisx --features live --all-targets
# live（需 Redis）:
REDIS_URL=redis://127.0.0.1:6379 cargo test -p redisx --features live -- --ignored
```

**禁止**把默认 `RedisAdapter` 当生产 Redis 客户端。

## 生产误用警示（infra-s9t.14）

**默认实现是进程内 scaffold/mock，不是生产客户端。**

- 禁止把 `*Adapter` 类型名当成已对接真实 Binance/Postgres/Redis/…
- 真实入口须有显式 feature（如 redisx `live`）与文档/CI 证据
- 详见 `docs/plans/artifacts/prod-consume-surface.md`
