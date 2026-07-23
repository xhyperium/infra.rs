# `postgresx` 当前实现规范

状态：当前 `0.3.8` 实现合同（`deadpool-postgres` + `tokio-postgres` 默认真实路径）。
**未宣称 package stable。**

## 0. 权威与范围

`postgresx` 位于 `crates/adapters/storage/postgres`。默认导出生产池、参数化 SQL、事务、
`PgRepository` 与 `PgTxRunner`；旧内存实现仅在 `scaffold` feature 下导出。

非目标：迁移工具、COPY、查询 DSL、读写副本路由、嵌套事务或跨资源事务。

## 1. 公开合同

| 入口 | 当前合同 |
|---|---|
| `PostgresConfig` | env / `DATABASE_URL`、池容量、连接/获取/操作截止时间、SSL 策略 |
| `PostgresPool` | connect/acquire/参数化 SQL/事务/health/stats/close |
| `PgConnection` | 参数化 SQL；客户端超时时丢弃连接，不把未知状态连接归池 |
| `PgTransaction` | BEGIN/SQL/COMMIT/ROLLBACK 全部有界；非穷尽 `TxStatus`；终结超时视为结果未知并丢弃连接 |
| `PgRepository` | 固定表 `infra_pg_records` 的生产 Repository |
| `PgTxRunner` | 正式 `contracts::TxRunner` 事务边界；业务 SQL 使用 `with_transaction` |

公开 SQL 方法要求 `&mut self`，使同一连接上的操作串行且超时后的失效状态可见。

## 2. 安全、TLS 与截止时间

- 所有业务值使用 `$1..$N` 参数；禁止拼接不可信输入。
- loopback / Unix socket 可显式 `sslmode=disable`；远程主机的 `disable` 或 `prefer` 在连接前
  返回 `Invalid`，仅 `require` 可用。
- `DATABASE_URL` 仅接受 `postgres://` / `postgresql://`；query allowlist 只有
  `sslmode`、`application_name`、`connect_timeout`。keyword DSN 以及其他认证/会话参数
  均 fail-closed；deprecated raw URL 与其显式结构化字段必须一致。
- 建池从同一组已校验字段重建配置，禁止 TLS、认证、会话策略与执行漂移。
- `Require` 使用 rustls + webpki roots；本版未提供自定义企业 CA / 客户端证书。
- deadpool 的 wait/create/recycle 均有界且 recycle 使用 `Clean`，兼容 raw pool 返回的
  session 状态也必须清理或丢弃。
- `acquire_timeout` 独立约束池等待；`operation_timeout` 同时作为调用侧 deadline 和服务端
  `statement_timeout`。
- RAII 取消守卫覆盖内部/外层 deadline、future drop 与 task abort；未知状态连接永久移出池。
- 事务进入可取消 await 前先转为 `TxStatus::Failed`，仅在连接安全恢复后回到 Active；
  旧三态 `TxState` 与 raw client/pool 保留一个 deprecation 周期，后者通过强制脱池与
  返回关闭的独立隔离池 fail-closed，正式路径不依赖逃逸口。
- 服务端语句错误保持 rollback-only `Failed`；只允许显式回滚，禁止继续 SQL 或 COMMIT。
- 业务+rollback 双失败通过 `TransactionRollbackFailure` 结构化保留两个错误分支。
- SQL / COMMIT / ROLLBACK 超时保留 error source；COMMIT 超时不得宣称成功或失败。
- Active transaction 被 Drop 时关闭已分离 session，由 PostgreSQL 回滚，不启动无监督任务。
- 密码与完整 DSN 不写入 Debug、日志或仓库。

## 3. 错误语义

SQLSTATE 映射到 `Invalid` / `Missing` / `Conflict` / `Transient` /
`Unavailable` / `Cancelled` 等稳定类别；调用侧截止返回 `DeadlineExceeded`。
关闭或已丢弃连接返回 `Unavailable`。

## 4. 可复验证据

固定摘要 PostgreSQL 17 容器实验覆盖：

1. 最大池容量 1 时占用唯一连接，第二次 acquire 在约定期限内返回 `DeadlineExceeded`；
2. 释放后可再次查询；
3. 关闭服务端 statement timeout 后，`pg_sleep` 由调用侧截止并丢弃连接；
4. 更短的外层 deadline 分别取消普通 SQL 与事务 SQL，随后均以新连接恢复；
5. 新建连接的 `SELECT 1` 成功，证明池未复用未知状态连接。

```bash
cargo test -p postgresx --all-targets
cargo clippy -p postgresx --all-targets -- -D warnings
node scripts/postgres-deadline-conformance.mjs
cmp .agents/ssot/adapters/storage/postgres/spec/spec.md \
  .agents/ssot/adapters/storage/postgres/spec/xhyper-postgresx-complete-spec.md
```

## 5. OPEN / NO-GO

mTLS 客户端证书、迁移/COPY、隔离级别策略、read replica、跨资源事务、HA 故障切换与
package stable 均未承诺。

- **可选自定义 CA 与 SNI（0.3.8 已落地）**：`tls_ca_file` / `tls_server_name` 叠加 webpki
  公共根；远程自签 Require live 已通过（仍强制证书校验，无 insecure 旁路）。
- **channel binding（`tls-server-end-point`）与 SCRAM-PLUS：未实现。**
  `MakeRustlsConnect` 的 `TlsStream::channel_binding()` 恒返回 `ChannelBinding::none()`；
  当前仅支持普通 SCRAM-SHA-256，服务端要求 channel binding 时将无法完成认证。
- **deprecated raw client/pool 隔离**：代码路径 `CODE PASS`；deadline 固定镜像实验已通过。
  不得把 raw 逃逸面升格为 package stable 证据。

追溯：`crates/adapters/storage/postgres/{src,tests}`、
`scripts/postgres-deadline-conformance.mjs`、`docs/ssot/postgresx-ssot-alignment.md`、
`evidence/2026-07-23/`。
