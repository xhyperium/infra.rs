# postgresx

Postgres 存储适配：**生产连接池 / 参数化 SQL 为默认导出**。
当前 workspace 版本为 `0.3.10`（foundation DoD 闭合）；`publish = false`，
**未宣称** package stable / crates.io。

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

## 重试安全

- `with_budget_safe` / `with_budget_async_safe` 要求显式 `RetrySafety`，用于 generic Adapter 组合。
- 当前 `PostgresPool` **没有** budget 字段、builder 或自动重试接线。
- 任意 SQL 不能仅凭字符串证明只读或幂等；未由调用方证明时使用 `UnsafeSideEffect`，
  `max_attempts > 1` 会在首次闭包/future 前拒绝。
- `with_budget` / `with_budget_async` / `with_retry_*` 为 unchecked compatibility，不得作为新生产默认。
- `with_budget_async` 委托 resiliencx 的 unchecked generic async core：预算耗尽统一返回标准 budget
  错误，`record_retry` 记录刚失败的 attempt（从 1 起）；这不改变其不校验 `RetrySafety` 的身份。

## 测试

最终本地结果：52 passed + 6 ignored；ignored live 测试需要外部 PostgreSQL，不作为默认 CI 通过证据。

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
