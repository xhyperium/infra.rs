# adapters/storage/taos — Matrix

| 字段 | 值 |
|------|-----|
| package | `taosx` `0.3.10` |
| 审计 | 2026-07-23 |
| gap 未完成 | **0**（见 `docs/ssot/taosx-gap-register.md`） |

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| S-1 | workspace member `taosx` | PASS | Cargo.toml |
| S-2 | 生产默认导出 | PASS | `TaosPool` REST |
| S-3 | 配置安全 | PASS | TLS/auth fail-closed |
| S-4 | 离线测试 | PASS | cargo test --all-targets |
| S-5 | 隔离/真实 live | PASS | integration + e2e + selfcheck |
| S-6 | bench 有界 | PASS | hot_path + api_matrix |
| S-7 | crate docs | PASS | README/CHANGELOG |
| S-8 | SSOT 树 | PASS | `taos/` 非 `taosx/` |
| S-9 | 产品档次 | SUPERSEDED | Production-default |
| S-10 | Decimal | PASS | NCHAR |
| S-11 | 资源边界 | PASS | HARD_MAX_* |
| S-12 | WS SQL | PASS | `exec_sql_ws` |
| S-13 | Native/HA/幂等 | PASS/SUPERSEDED | probe + hosts + idempotent |
| S-14 | 十轮审查 | PASS | report + gap register |
| S-15 | 公开 API 点名 | PASS | public_api_surface + IT |
| S-16 | BatchWriteReport | PASS | report APIs |
| S-17 | metrics 导出 | PASS | prometheus text |
| S-18 | WS live | PASS | IT native mode |
| S-19 | health | PASS | liveness/health |
| S-20 | selfcheck Full | PASS | 9 项无 missing-client Skip |
| S-21 | stream/batcher/tmq/soak | PASS | 模块 + live |
