# postgresx

- **默认生产面**：`PostgresPool` / `PostgresConfig` / `PgConnection` / `PgTransaction`
- 驱动：`deadpool-postgres` + `tokio-postgres`（参数化 SQL only）
- `PgTxRunner`：真实事务**边界**；业务 SQL 用 `with_transaction`
- feature `scaffold`：内存 `PostgresAdapter` / `ObservingPostgresAdapter`
- live 测 `#[ignore]`；密码/DSN **禁止**入仓
- 文档：`docs/{usage,config,operations}.md`
