# postgresx 用法

**Package**：`postgresx` · 默认导出为**生产 SQL / 连接池 API**。

## 最小示例

```rust
use postgresx::{PostgresConfig, PostgresPool};

# async fn demo() -> kernel::XResult<()> {
let cfg = PostgresConfig::from_env()?;
let pool = PostgresPool::connect(&cfg).await?;

let row = pool.query_one("SELECT $1::text AS v", &[&"ok"]).await?;
let v: String = row.get("v");
assert_eq!(v, "ok");

pool.with_transaction(async |tx| {
    tx.execute(
        "CREATE TEMP TABLE t (id int PRIMARY KEY)",
        &[],
    ).await?;
    tx.execute("INSERT INTO t (id) VALUES ($1)", &[&1i32]).await?;
    Ok::<_, kernel::XError>(())
}).await?;

pool.close();
# Ok(())
# }
```

## Builder

```rust
use postgresx::{PostgresConfig, SslMode};

# fn demo() -> kernel::XResult<()> {
let cfg = PostgresConfig::builder()
    .host("127.0.0.1")
    .port(5432)
    .database("app")
    .user("app")
    .password("secret")
    .sslmode(SslMode::Disable)
    .max_pool_size(8)
    .build()?;
# let _ = cfg;
# Ok(())
# }
```

## 事务

| API | 用途 |
|-----|------|
| `PostgresPool::with_transaction` | **推荐**：闭包内多条 SQL，Ok→commit / Err→rollback |
| `PostgresPool::begin` → `PgTransaction` | 手动 `commit` / `rollback` |
| `PgTxRunner` + `contracts::TxRunner` | 仅事务**边界**；`dyn TxContext` **无** SQL 句柄 |

## contracts 集成

```rust
use std::sync::Arc;
use contracts::run_tx_commit_on_ok;
use postgresx::{PgTxRunner, PostgresPool};

# async fn demo(pool: Arc<PostgresPool>) -> kernel::XResult<()> {
let runner = PgTxRunner::new(pool);
let n = run_tx_commit_on_ok(&runner, |_ctx| async move {
    Ok::<_, kernel::XError>(1u32)
}).await?;
assert_eq!(n, 1);
# Ok(())
# }
```

业务 SQL 请走 `with_transaction`，不要指望 `dyn TxContext` 执行查询。

## scaffold（可选）

```bash
cargo test -p postgresx --features scaffold
```

启用后导出内存 `PostgresAdapter` / `ObservingPostgresAdapter`（**非**生产客户端）。

## 参数化（强制）

```rust
// ✅
# async fn ok(pool: &postgresx::PostgresPool, id: i32) -> kernel::XResult<()> {
pool.query_one("SELECT * FROM t WHERE id = $1", &[&id]).await?;
# Ok(())
# }

// ❌ 禁止
// pool.query_one(&format!("SELECT * FROM t WHERE id = {id}"), &[]).await?;
```
