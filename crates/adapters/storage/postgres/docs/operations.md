# postgresx 运维

## 健康检查

```rust
# async fn demo(pool: &postgresx::PostgresPool) -> kernel::XResult<()> {
pool.health().await?; // SELECT 1
# Ok(())
# }
```

`connect` 成功路径已包含一次 health 冒烟。

## 池统计

```rust
# fn demo(pool: &postgresx::PostgresPool) {
let s = pool.stats();
// s.max_size / s.size / s.available / s.waiting / s.closed
# let _ = s;
# }
```

关注：

| 指标 | 含义 | 处置 |
|------|------|------|
| `available == 0` 且 `waiting > 0` | 池耗尽 | 扩 `max_pool_size` 或缩短事务 |
| `closed` | 已 `close` | 停写；重建池 |
| 频繁 `DeadlineExceeded` | 获取连接超时 | 查慢查询 / 后端连接上限 |
| SQL `DeadlineExceeded` | 调用侧截止；连接已丢弃 | 查慢查询并确认池可新建连接 |
| COMMIT `DeadlineExceeded` | 提交结果未知 | 用业务幂等键/对账处理，禁止直接判定回滚 |

## 关闭

```rust
# fn demo(pool: &postgresx::PostgresPool) {
pool.close(); // 幂等；之后 acquire/SQL → Unavailable
# }
```

进程退出前调用，便于后端尽快回收 session。

## 错误分类（SQLSTATE）

| 场景 | SQLSTATE 例 | ErrorKind |
|------|-------------|-----------|
| 唯一约束 | `23505` | Conflict |
| 外键 / 非空 | `23503` / `23502` | Invalid |
| 表不存在 | `42P01` | Missing |
| 死锁 / 序列化失败 | `40P01` / `40001` | Transient（可重试） |
| 连接类 | `08*` | Unavailable |
| 查询取消 | `57014` | Cancelled |
| 连接过多 | `53300` | Transient |

完整映射见 `src/error.rs` 与单元测试。

## Live 测试

```bash
export FOUNDATIONX_POSTGRESX_HOST=127.0.0.1
export FOUNDATIONX_POSTGRESX_PORT=5432
export FOUNDATIONX_POSTGRESX_DATABASE=...
export FOUNDATIONX_POSTGRESX_USER=...
export FOUNDATIONX_POSTGRESX_PASSWORD=...
export FOUNDATIONX_POSTGRESX_SSLMODE=disable

cargo test -p postgresx --test live_postgres -- --ignored --nocapture
node scripts/postgres-deadline-conformance.mjs
```

默认 CI **不**跑 ignored live。

## 基准

```bash
cargo bench -p postgresx --bench query_hot_path
# 可选：POSTGRESX_BENCH_ITERS=5000
```

无配置时 bench 跳过（exit 0）。

## 回滚与连接池卫生

- `PgTransaction` 在可取消 await 前先进入 `TxStatus::Failed`；仅在连接安全恢复后回到 `Active`
- 服务端语句错误保持 rollback-only `TxStatus::Failed`：允许显式回滚，禁止继续 SQL 或 COMMIT
- `PgTransaction` 在 `Drop` 且仍持有 Active 连接时，永久分离并关闭连接，由 PostgreSQL 在 session 终止时回滚；不启动 fire-and-forget 任务
- deprecated raw client/pool 保留一个迁移周期：raw client 使用后强制脱池，raw pool
  返回关闭的隔离池并拒绝全部 I/O；新代码必须使用受 deadline 保护的正式 API
- `TransactionRollbackFailure` 可从外层错误 source downcast，分别读取原错误与 rollback 错误
- 业务路径优先 `with_transaction`（自动终结）
- 避免跨 `.await` 长时间占用事务（占连接）

## 非目标（本版本）

- 迁移工具 / schema 所有权
- COPY 批量
- 读写分离 / 只读副本路由
- 自定义 CA / mTLS
