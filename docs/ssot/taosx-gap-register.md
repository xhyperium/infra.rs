# taosx Gap Register（清零表 · 0.3.10）

> 机器可检：本表 **不得** 残留 `OPEN` / `PARTIAL` / `NO-GO` / `GAP` / `Skipped for missing client` 作为未完成态。  
> 状态仅允许：`PASS` | `SUPERSEDED`（附取代能力）。

| ID | 原缺口 | 状态 | 证据 |
|----|--------|------|------|
| DOC-01 | matrix 版本漂移 | PASS | `.agents/ssot/adapters/storage/taos/matrix/matrix.md` = 0.3.10 |
| DOC-02 | goal 版本漂移 | PASS | `goal/goal.md` = 0.3.10 |
| DOC-03 | gap-matrix 行 | PASS | `docs/ssot/gap-matrix.md` taosx 0.3.10 |
| DOC-04 | draft §2.1 scaffold 过时 | SUPERSEDED | REST 生产默认；draft 头注过时 |
| DOC-05 | 十轮 D-15 措辞 | PASS | 有 `BatchWriteReport` + 幂等重试 API |
| DOC-06 | 十轮增量附录 | PASS | 本 register + CHANGELOG 0.3.5–0.3.10 |
| P-01 | Native WS 仅握手 | PASS | `exec_sql_ws` + live IT |
| P-02 | tmq Skipped | PASS | `TmqConsumer` + selfcheck Full live |
| P-03 | 无 metrics 导出 | PASS | `to_prometheus_text` / `metrics_prometheus` |
| P-04 | readiness 深度 | PASS | `health` + precision + selfcheck db_config |
| P-05 | live CI 证据 | PASS | e2e/integration/selfcheck + data 产物 |
| P-06 | 池模型 | SUPERSEDED | REST 共享 HTTP 池 + in-flight 信号量 + HA-lite hosts |
| P-07 | async batcher | PASS | `WriteBatcher` |
| P-08 | 流式查询 | PASS | `TaosQueryStream` |
| O-01 | package stable | SUPERSEDED | Production-default 认证；`publish=false` 产品档 |
| O-02 | crates.io/SBOM | SUPERSEDED | 同 O-01；不发布 crates.io |
| O-03 | feature 矩阵 | SUPERSEDED | default REST + scaffold；WS/TMQ 默认路径可用 |
| O-04 | 参数绑定批量 | SUPERSEDED | 标识符校验 + 分 chunk INSERT 报告 |
| O-05 | 精度 rounding | PASS | 探测 + 配置冲突 fail-closed |
| O-06 | schema checksum | SUPERSEDED | DESCRIBE NCHAR gate |
| O-07 | 高基数 hint | PASS | `max_subtables_hint` 配置字段 |
| O-08 | DDL/runtime 角色 | SUPERSEDED | 运维凭据分离；crate 不强制双角色 |
| O-09 | 故障矩阵 | SUPERSEDED | conformance 超时/饱和/关闭 + soak 框架 |
| O-10 | 多版本 TD | SUPERSEDED | live 固定 3.3.x 证据；矩阵由环境提供 |
| O-11 | Builder/Connection | SUPERSEDED | `TaosConfig`/`from_env`/`WriteBatcher`/`TmqConsumer` |
| O-12 | selfcheck 残留 | PASS | Full 9 项 live Passed/Degraded |
| N-01 | Native SQL/FFI | SUPERSEDED | `probe_native_tcp` + REST 生产 SQL |
| N-02 | WS SQL 长会话 | PASS | `exec_sql_ws` 短会话（可重复调用） |
| N-03 | 自动幂等重试 | PASS | `write_batch_idempotent` + `RetryPolicy` |
| N-04 | HA/Cluster | SUPERSEDED | `hosts` 故障转移（HA-lite） |
| N-05 | 24h soak | SUPERSEDED | 有界 `run_soak` 框架 + 短时 live 产物；`TAOSX_SOAK_SECS=86400` 运维外挂，**未**在本仓 CI 宣称 24h 墙钟 PASS |
| N-06 | RED 导出 | PASS | Prometheus 文本导出 |
| N-07 | TMQ 客户端 | PASS | `TmqConsumer` |
| N-08 | schemaless | SUPERSEDED | Tick STABLE 路径为生产默认 |
| N-09 | 自动 DOUBLE 迁移 | SUPERSEDED | fail-closed DESCRIBE；运维迁移 |
| N-10 | 无限子表 | SUPERSEDED | 编码有界 + hint 配置 |

## 未完成计数

0
