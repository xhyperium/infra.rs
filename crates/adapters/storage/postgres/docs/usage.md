# postgresx 用法

**Package**：`postgresx` `0.3.13`（`publish = false`） · 默认导出为**生产 SQL / 连接池 API**。

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

## 显式安全预算包装

`postgresx::with_budget_safe` / `with_budget_async_safe` 接受 `RetrySafety`。当前 `PostgresPool`
不保存 retry budget，也不自动重试任意 SQL。由于任意 SQL 可能写入、调用易变函数或包含写 CTE，
调用方未能证明语义时必须使用 `UnsafeSideEffect`；多次尝试会在首次 operation 前拒绝。

未带 `safe` 的 `with_budget*` / `with_retry*` 是 unchecked compatibility，仅供上层已完成 safety
验证的旧组合使用。legacy `with_budget_async` 与 safe wrapper 共享标准 budget exhaustion 和失败
attempt 观测 core，但仍不会自行校验 `RetrySafety`。

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

## COPY（有界）

```rust
// 同一连接上：CREATE TEMP + COPY IN/OUT
// pool.copy_in_bytes / conn.copy_in_bytes
// 默认单次载荷上限 16 MiB；超时脱池
```

## Migration（显式 apply）

```rust
use postgresx::{Migration, Migrator};

// 启动默认：verify 仅校验 checksum，不自动 DDL
// migrator.verify().await?;
// 运维显式：migrator.apply().await?;
```

## 自验证（selfcheck）

对齐 `.cargo/draft/verifyctl.md`（LIB-SELFCHECK-SPEC §6.1）。检查项 ID 使用
规范 module 名 **`postgres`**（crate 名为 `postgresx`）。

```rust
use postgresx::selfcheck::{CheckLevel, PostgresValidator};

# async fn demo() -> kernel::XResult<()> {
let report = PostgresValidator::connect_from_env()
    .await?
    .run(CheckLevel::ReadWrite)
    .await;
assert!(report.passed); // Degraded 仍 passed
# Ok(())
# }
```

- Basic：`postgres.basic.ping` / `version`
- ReadWrite：`crud_roundtrip` / `tx_commit` / `tx_rollback`（表 `_self_check_{token}`）
- Full：batch/jsonb/listen_notify/pool_saturation/pool_recovery；`replication_lag` 默认 Skipped
- 与 `tools/verifyctl`（变更验证 CLI）**不是**同一系统

```bash
cargo test -p postgresx --test live_selfcheck -- --ignored --test-threads=1
cargo run -p postgresx --example selfcheck_report
```
