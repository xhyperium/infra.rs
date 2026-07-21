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

## 生产误用警示（infra-s9t.14）

**默认实现是进程内 scaffold/mock，不是生产客户端。**

- 禁止把 `*Adapter` 类型名当成已对接真实 Binance/Postgres/Redis/…
- 真实入口须有显式 feature（如 redisx `live`）与文档/CI 证据
- 详见 `docs/plans/artifacts/prod-consume-surface.md`
