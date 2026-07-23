# Gap Matrix — .cargo/draft → infra.rs (2026-07-23)

> **权威快照**：storage×7 OBJECTIVE DEFER 已闭合（生产默认就绪）。  
> package stable / crates.io **仍未**宣称。

| Domain | Draft DoD P0 | Current | Deferred (not OBJECTIVE / not stable) |
|--------|--------------|---------|----------------------------------------|
| postgresx | Pool+query+tx+TLS | **done** + Repository + 远程 Require live（CA+SNI）+ deadline + Migrator+COPY+mTLS+**selfcheck §6.1**+live (`0.3.12`) | 无限流式 COPY / read-replica / package stable / 服务端强制 mTLS live / down migration |
| redisx | Pool+KV+structures+Streams+tx+live | **done** 全公开 API + selfcheck + live/E2E/bench (`0.3.15`) | Cluster/Sentinel/TLS live（无 env）/ package stable / PubSub NO-GO |
| postgresx | Pool+query+tx+TLS | **done** + Repository + 远程 Require live（CA+SNI）+ deadline + Migrator+COPY+mTLS+**selfcheck §6.1**+live + 合同文档对齐 (`0.3.13`) | 无限流式 COPY / read-replica / package stable / 服务端强制 mTLS live / down migration / channel binding |
| kafkax | Producer pool + EventBus | **done** + headers/key/stats + selfcheck + 生产矩阵 (`0.3.9`) | SCRAM 成功路径/group/rebalance/native EOS/DLQ/Part2 OOS/24h 默认 soak |
| natsx | Core NATS EventBus | **done** + JetStream durable pull/显式确认 + 同客户端重启恢复 3/3 (`0.3.2`) | 断线窗口无回放 / NKey / Cluster/HA / 自动 DLQ / KV·Object 全量 |
| ossx | ObjectStore put/get | **done** + 有界 multipart/retry/orphan 补偿 (`0.3.2`) | lifecycle / STS / TB 流式对象 |
| clickhousex | Analytics insert+select | **done** + HTTPS/PEM CA + insert_batch + 有界池 (`0.3.2`) | 真实集群 TLS / mTLS / native 9000 / cluster 运维 |
| taosx | TimeSeries write+query | **done** Production-default 全 API + selfcheck Full + e2e/bench (`0.3.10`)；gap 未完成=0 | —（见 taosx-gap-register SUPERSEDED 行） |
| goalctl | Goal→Contract digest | **done** | full multi-module authority plane |
| verifyctl | plan+execute+run-result | **done** | full V0–V3 gate matrix |

Freeze: production-default path per domain; scaffold behind `scaffold` feature; no secrets in git; **no** package-stable claim.

## Live 入口

```bash
scripts/live/export-foundationx-env.sh --env dev -- \
  cargo test -p redisx -p postgresx -p kafkax -p natsx \
    -p ossx -p clickhousex -p taosx -- --ignored
```

## 相关 PR

| PR | 说明 |
|----|------|
| #188–#191 | storage 生产客户端 + live 凭据 |
| #195 | storage×7 SSOT layers |
| #211 | storage×7 OBJECTIVE DEFER 闭合 → `0.3.1`/`0.3.2` (redis/postgres) |
| #219 | redisx/postgresx SSOT version 行对齐 `0.3.2` |

## postgresx 专项 Gap 追踪（2026-07-23）

> 审计日期：2026-07-23 · 文件：`crates/adapters/storage/postgres/` · 版本 `0.3.13`

### 已修复

| ID | Gap | 状态 | 提交 | 证据 |
|----|-----|------|------|------|
| G-7 | `tracing` optional feature gate 未接线 | **FIXED** 2026-07-23 | `16b77e8c` | 75 行 instrumentation 分布于 pool(14)、conn(5)、tx(4)、migration(2)；`--features tracing` 编译通过 |
| T-1 | `runner.rs` 无内联单元测试 | **FIXED** 2026-07-23 | `08e980fe` | 10 项测试（Send+Sync、构造、双 commit/rollback 拒绝、run_tx_lifecycle 边界） |
| T-2 | `batch_execute` 未在 crate-root 文档提及 | **FIXED** 2026-07-23 | `08e980fe` | `lib.rs:9` 更新为 `` [`PgConnection`]（`batch_execute`） `` |

### 仍 OPEN

| ID | Gap | 说明 |
|----|-----|------|
| G-1 | package stable / crates.io | `publish = false`；禁止宣称 |
| G-2 | 无限流式 COPY / cursor | 仅有界 `copy_in_bytes`/`copy_out_bytes`（16 MiB） |
| G-3 | down migration | Migrator 仅 forward verify/apply |
| G-4 | read-replica 路由 | 无 multi-host / target_session_attrs |
| G-5 | 服务端强制 mTLS live | 客户端已落地，服务端强制依赖部署侧 |
| G-6 | channel binding / SCRAM-PLUS | `CHANNEL_BINDING_ENABLED = false`（编译期锚定） |

### 仍 DEPRECATED（待迁移周期结束）

| ID | 符号 | 文件 |
|----|------|------|
| D-1 | `TxState` 枚举 | `tx.rs:22` → 改用 `TxStatus` |
| D-2 | `PostgresConfig.database_url` | `config.rs:109` → 改用结构化字段 |
| D-3 | `PgConnection::client()`/`client_mut()` | `conn.rs:256,265` |
| D-4 | `PostgresPool::inner()` | `pool.rs:290` |
| D-5 | `run_tx_commit_on_ok`（contracts 侧） | `contracts/src/lib.rs:209` |

### 测试覆盖（2026-07-23 快照）

| 测试文件 | 离线 | Live（ignored） |
|----------|------|-----------------|
| `src/` 内联测试 | 66 | 0 |
| `tests/functional_api.rs` | 57 | 14 |
| `tests/e2e_workflow.rs` | 1 | 8 |
| `tests/integration_stress.rs` | 2 | 5 |
| `tests/edge_cases.rs` | 2 | 8 |
| `tests/live_postgres.rs`（已有） | 0 | 12 |
| `tests/live_selfcheck.rs`（已有） | 0 | 2 |
| `tests/deadline_conformance.rs`（已有） | 0 | 1 |
| **总计** | **128** | **50** |

### Benchmarks（2026-07-23 新增）

| 文件 | 基准组 | 覆盖维度 |
|------|--------|----------|
| `benches/query_hot_path.rs` | 增强 5 维度 | 不同参数数（1/5/10/20）、不同行数（1/10/100/500/1000）、不同数据类型、预编译 vs 简单查询 |
| `benches/pool_operations.rs` | 9 项 | acquire/health/execute/query/copy/transaction commit/repository save-find |
| `benches/pool_concurrency.rs` | 3 项 | 并行 acquire 吞吐、混合读写并发、池饱和恢复 |
