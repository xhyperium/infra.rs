# postgresx

Postgres 存储适配：**生产连接池 / 参数化 SQL 为默认导出**。

| 面 | 说明 |
|----|------|
| 生产默认 | `PostgresConfig` + `PostgresPool` + `PgConnection` / `PgTransaction` |
| contracts | `PgTxRunner`（真实 BEGIN/COMMIT/ROLLBACK **边界**；SQL 请用 `with_transaction`） |
| deadline | pool acquire 与 SQL/事务终结内部有界；调用侧超时丢弃连接 |
| scaffold | feature `scaffold`：`PostgresAdapter` / `ObservingPostgresAdapter`（内存，非生产） |

## 快速开始

```rust
use postgresx::{PostgresConfig, PostgresPool};

# async fn demo() -> kernel::XResult<()> {
let pool = PostgresPool::connect(&PostgresConfig::from_env()?).await?;
let row = pool.query_one("SELECT 1 AS n", &[]).await?;
let n: i32 = row.get("n");
assert_eq!(n, 1);
pool.close();
# Ok(())
# }
```

环境变量见 [docs/config.md](./docs/config.md)。也可用 `DATABASE_URL` 覆盖。

## 文档

- [docs/usage.md](./docs/usage.md)
- [docs/config.md](./docs/config.md)
- [docs/operations.md](./docs/operations.md)

## 测试

```bash
cargo test -p postgresx
cargo test -p postgresx --features scaffold
cargo test -p postgresx --test live_postgres -- --ignored
node scripts/postgres-deadline-conformance.mjs
cargo bench -p postgresx --bench query_hot_path
```

## 依赖

- `deadpool-postgres` + `tokio-postgres`（workspace）
- `kernel` / `contracts` / `async-trait` / `tokio`
- 可选 `tracing`

**禁止**将密码或完整 DSN 提交到 git。
