# adapters/storage/taos — Matrix

| 字段 | 值 |
|------|-----|
| package | `taosx` `0.3.5` |
| 审计 | 2026-07-23 |

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| S-1 | workspace member `taosx` | PASS | Cargo.toml |
| S-2 | 生产默认导出 | PASS | `TaosPool / TaosClient REST` |
| S-3 | 配置安全 | PASS | 远程 TLS/auth fail-closed；strict host；redirect 禁止；密码脱敏 |
| S-4 | 离线测试 | PASS | cargo test -p taosx --all-targets |
| S-5 | 隔离 live | PASS | 固定 digest runner：2 passed，exit 0 |
| S-5b | 真实 dev live | PASS | `export-foundationx-env` + `live_smoke` 2/2（2026-07-23） |
| S-6 | bench 有界 | PASS | `benches/hot_path.rs（3s 有界）` |
| S-7 | crate docs | PASS | docs/usage·config·operations |
| S-8 | SSOT 11 层 + listing | PASS | 本树 `taos/`（非 `taosx/`） |
| S-9 | package stable | OPEN | 未宣称 |
| S-10 | Decimal 无损 | PASS | NCHAR(64+) + DESCRIBE fail-closed + scale=18 测试 |
| S-11 | 资源/关闭硬边界 | PASS | response/batch/query/in-flight/close tests |
| S-12 | WS reachability | PARTIAL | 仅握手/关闭探测；SQL 始终 REST |
| S-13 | Native SQL / FFI / HA / 幂等重试 | NO-GO | 无实现或真实证据 |
| S-14 | 十轮审查矩阵 | PASS | `docs/report/2026-07-23/taosx-ten-round-review.md` |
| S-15 | 公开 API 表面 100% 点名 | PASS | `public_api_surface` + 行为测试 |
| S-16 | 批量部分成功报告 | PASS | `BatchWriteReport` / `write_batch_chunked_outcome`（无自动重试） |
