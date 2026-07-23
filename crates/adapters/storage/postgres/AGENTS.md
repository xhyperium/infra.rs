# postgresx

- **Package / lib / 当前版本**：`postgresx` / `postgresx` / `0.3.8`（`publish = false`；未宣称 package stable）
- **默认生产面**：`PostgresPool` / `PostgresConfig` / `PgConnection` / `PgTransaction`
- 驱动：`deadpool-postgres` + `tokio-postgres`（参数化 SQL only）
- `PgTxRunner`：真实事务**边界**；业务 SQL 用 `with_transaction`
- 远程 PostgreSQL 仅允许 `sslmode=require`；disable/prefer 仅限 loopback / Unix socket
- acquire/SQL/COMMIT/ROLLBACK 必须内部有界；调用侧超时必须丢弃连接
- feature `scaffold`：内存 `PostgresAdapter` / `ObservingPostgresAdapter`
- live 测 `#[ignore]`；密码/DSN **禁止**入仓
- 文档：`docs/{usage,config,operations}.md`
