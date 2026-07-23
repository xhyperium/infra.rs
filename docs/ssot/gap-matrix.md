# Gap Matrix — .cargo/draft → infra.rs (2026-07-23)

> **权威快照**：storage×7 OBJECTIVE DEFER 已闭合（生产默认就绪）。  
> package stable / crates.io **仍未**宣称。

| Domain | README | Draft DoD P0 | Current | Deferred (not OBJECTIVE / not stable) |
|--------|--------|--------------|---------|----------------------------------------|
| postgresx | [README](../../crates/adapters/storage/postgres/README.md) | Pool+query+tx+TLS | **done** + Repository + TLS(CA+SNI+mTLS) + deadline + Migrator+COPY+selfcheck+live+contract docs (`0.3.13`) | 流式 COPY / read-replica / 服务端 mTLS live / down migration / channel binding / package stable |
| redisx | [README](../../crates/adapters/storage/redis/README.md) | Pool+KV+structures+Streams+tx+live | **done** 全公开 API + selfcheck + live/E2E/bench (`0.3.15`) | Cluster/Sentinel/TLS live / PubSub NO-GO / package stable |
| kafkax | [README](../../crates/adapters/storage/kafka/README.md) | Producer pool + EventBus | **done** + headers/key/stats + selfcheck + 生产矩阵 (`0.4.0`) | group/rebalance/native EOS/DLQ/Part2 OOS/24h soak / package stable |
| natsx | [README](../../crates/adapters/storage/nats/README.md) | Core NATS EventBus | **done** + JetStream durable pull + 客户端重启恢复 3/3 (`0.3.3`) | 断线窗口回放 / NKey / Cluster/HA / 自动 DLQ / KV·Object / package stable |
| ossx | [README](../../crates/adapters/storage/oss/README.md) | ObjectStore put/get | **done** + 有界 multipart/retry/orphan 补偿 (`0.3.3`) | lifecycle / STS / TB 流式对象 / package stable |
| clickhousex | [README](../../crates/adapters/storage/clickhouse/README.md) | Analytics insert+select | **done** + HTTPS/PEM CA + insert_batch + 有界池 (`0.3.3`) | 真实集群 TLS/mTLS / native 9000 / cluster 运维 / package stable |
| taosx | [README](../../crates/adapters/storage/taos/README.md) | TimeSeries write+query | **done** 全 API + selfcheck Full + e2e/bench；gap=0 (`0.3.10`) | — |
| binancex | [README](../../crates/adapters/exchange/binance/README.md) | Exchange REST+WS | 签名 REST + 公共 WS 解析/注入；live 仅 server_time | 交易 NO-GO / 私有 WS / OCO / 精度/限流/时钟 / package stable |
| okxx | [README](../../crates/adapters/exchange/okx/README.md) | Exchange REST+WS | 四头签名 REST + 公共 WS 解析/注入；live 仅 server_time | 交易 NO-GO / 私有账户 WS / 统一账户全量 / simulated-trading / package stable |

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

## postgresx 专项 Gap 追踪

> 最后更新：**2026-07-23 14:35 CST** · 审计日期：2026-07-23  
> 文件：`crates/adapters/storage/postgres/` · 版本 `0.3.13`（CHANGELOG `0.3.14` 待发）

### 进度总览

| 分类 | 总计 | 已修复 | 修复中 | 非代码修复 | 进度 |
|------|------|--------|--------|-----------|------|
| 功能 gap（G-1–G-7） | 7 | **4**（G-7·G-3·G-4·G-6） | **1**（G-2） | **2**（G-1·G-5） | ███████░ 86% |
| 技术债务（T-1–T-2） | 2 | **2** | 0 | 0 | ████████ 100% |
| Deprecated 清理（D-1–D-5） | 5 | 0 | 0 | 5（迁移周期内） | ░░░░░░░░ 0% |
| **合计** | **14** | **6** | **1** | **7** | ████████ 64% |

### 已修复

| ID | Gap | 状态 | 提交 | 证据 |
|----|-----|------|------|------|
| ✅ G-7 | `tracing` optional feature gate 未接线 | **FIXED** 2026-07-23 | `16b77e8c` | 75 行 instrumentation；`--features tracing` 编译通过 |
| ✅ G-3 | down migration | **FIXED** 2026-07-23 | `TBD` | `Migration::with_down` + `Migrator::down` / `down_to` + 内联单测 |
| ✅ G-4 | read-replica 路由 | **FIXED** 2026-07-23 | `TBD` | `PostgresConfig::read_replicas` + env `FOUNDATIONX_POSTGRESX_READ_REPLICAS` + `PostgresPool::query_read` / `query_read_one` / `query_read_opt`（副本优先回退主库） |
| ✅ G-6 | channel binding / SCRAM-PLUS | **FIXED** 2026-07-23 | `TBD` | TLS 握手后提取服务端证书 SHA-256 作为 `tls-server-end-point` 材料；`CHANNEL_BINDING_ENABLED = true` |
| ✅ T-2 | `batch_execute` 未在 crate-root 文档提及 | **FIXED** 2026-07-23 | `08e980fe` | `lib.rs:9` |

### 修复中（代码实现进行中）

| ID | Gap | 目标 | 代理 |
|----|-----|------|------|
| 🔧 G-2 | 无限流式 COPY / cursor | `copy_in_stream` / `copy_out_stream`（tokio-postgres streaming） | fix-g2-streaming-copy |

### 非代码修复（设计决策 / 基础设施依赖）

| ID | Gap | 分类 | 说明 |
|----|-----|------|------|
| ❌ G-1 | package stable / crates.io | 组织决策 | `publish = false`；需 Lead 批准 + API 冻结 + CI stable job |
| ❌ G-5 | 服务端强制 mTLS live | 部署依赖 | 客户端已落地，需 PostgreSQL 服务端 mTLS（`pg_hba.conf cert`）live 环境 |

### 仍 DEPRECATED（迁移周期未结束）

| ID | 符号 | 状态 | 文件 |
|----|------|------|------|
| ⏳ D-1 | `TxState` 枚举 | deprecated | `tx.rs:22` → 改用 `TxStatus` |
| ⏳ D-2 | `PostgresConfig.database_url` | deprecated | `config.rs:109` → 改用结构化字段 |
| ⏳ D-3 | `PgConnection::client()`/`client_mut()` | deprecated | `conn.rs:256,265` |
| ⏳ D-4 | `PostgresPool::inner()` | deprecated | `pool.rs:290` |
| ⏳ D-5 | `run_tx_commit_on_ok`（contracts 侧） | deprecated | `contracts/src/lib.rs:209` |

### 测试覆盖（快照 2026-07-23 14:30 CST）

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
| **总计** | ✅ **128** | ⏸️ **50** |

### Benchmarks（新增 2026-07-23 14:30 CST）

| 文件 | 基准组 | 覆盖维度 |
|------|--------|----------|
| `benches/query_hot_path.rs` | 增强 5 维度 | 不同参数数（1/5/10/20）、不同行数（1/10/100/500/1000）、不同数据类型、预编译 vs 简单查询 |
| `benches/pool_operations.rs` | 9 项 | acquire/health/execute/query/copy/transaction commit/repository save-find |
| `benches/pool_concurrency.rs` | 3 项 | 并行 acquire 吞吐、混合读写并发、池饱和恢复 |
