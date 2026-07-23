# adapters/storage/taos — Review（0.3.10）

| 维度 | 结论 | 说明 |
|------|------|------|
| P0 生产默认路径 | **PASS** | REST `TaosPool` |
| health/readiness | **PASS** | `liveness` 本地；`health` 轻量 SQL |
| offline/live 测试 | **PASS** | 57 unit + 11 conformance + live `#[ignore]` |
| selfcheck（9 项 Full） | **PASS** | REST/WRITE/QUERY/SCHEMA/METRICS/NATIVE/BATCHER/STREAM/TMQ |
| metrics（Prometheus） | **PASS** | `TaosMetricsSnapshot` + 计数器 |
| 流式查询 | **PASS** | `TaosQueryStream` |
| 异步批写 | **PASS** | `WriteBatcher` |
| TMQ 消费 | **PASS** | `TmqConsumer` subscribe/poll |
| 幂等重试 | **PASS** | `RetryPolicy` + `write_batch_idempotent` |
| HA-lite | **PASS** | `hosts` 故障转移 |
| package stable | **NOT CLAIMED** | 禁止 |
| DEFER | 记录 | Native SQL / HA Cluster 全矩阵 / OTLP 导出 |

**Verdict（P0）**：`ready with follow-ups`
