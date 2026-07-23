# adapters/storage/taos — Review

| 维度 | 结论 | 说明 |
|------|------|------|
| P0 生产默认路径 | **PASS** | `TaosPool / TaosClient REST` |
| 离线测试 | **PASS** | `cargo test -p taosx --all-targets` |
| live 入口 | **PASS** | REST 写查 + metrics + Native WS 握手（2026-07-23） |
| 批量报告 | **PASS** | `BatchWriteReport` 部分成功可定位 |
| 有界 metrics | **PASS（0.3.6）** | 进程内计数；非 OTLP |
| 文档 | **PASS** | crate docs + SSOT |
| package stable | **NOT CLAIMED** | 禁止 |
| DEFER | 记录 | Native SQL / WS 长会话 / HA / 幂等自动重试 / 远程 RED |

**Verdict（P0）**：`ready with follow-ups`
