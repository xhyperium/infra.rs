# postgresx

Postgres adapter：

- scaffold：`PostgresAdapter`（内存 + `FakeTxContext`）
- mock 验证入口：`ObservingPostgresAdapter` / `MockPostgresBackend`
  （staged 写入 + commit 边界；**非**真实 Postgres / **非** Production Ready）

```rust
use contracts::{Repository, TxRunner};
use postgresx::{ObservingPostgresAdapter, Record};

# async fn demo() -> kernel::XResult<()> {
let a = ObservingPostgresAdapter::local();
let mut tx = a.begin_mock_tx().await?;
tx.stage_save(Record { id: "1".into(), data: b"x".to_vec() })?;
tx.commit().await?;
assert!(a.find("1".into()).await?.is_some());
# Ok(())
# }
```
