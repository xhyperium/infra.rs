# taosx SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `taosx` |
| SSOT | `.agents/ssot/adapters/storage/taos/`（**禁止**平行 `taosx/` 树） |
| 实现 | `crates/adapters/storage/taos` |
| 审计日期 | 2026-07-23 |
| version | `0.3.8` |
| 结论 | **受限 REST SQL + health + metrics + selfcheck §6.7 已落地**；Native SQL / HA / 幂等重试 / TMQ / package stable **NO-GO** |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 生产默认面 | `TaosPool / TaosClient` REST（6041） |
| batch write | `write_batch*` / `write_batch_*report` / `BatchWriteReport` / `build_insert_sql_chunks` |
| WS | `TransportMode::NativeWs` + `connect_native_ws`（仅握手/关闭探测；SQL 始终 REST） |
| Decimal | NCHAR(64+) 文本往返；DESCRIBE 拒绝存量 DOUBLE schema |
| 资源 | response / SQL batch / query rows / in-flight / close drain 均有硬上限 |
| 安全 | 远程 TLS/auth fail-closed；strict host；REST redirect 禁止；密码 Debug 脱敏 |
| contracts | `TimeSeriesStore`（ts 纳秒 epoch） |
| 环境变量 | `FOUNDATIONX_TAOSX_{HOST,PORT,USER,PASSWORD,DATABASE,TLS,PRECISION,TRANSPORT,...}` |
| live | 2026-07-23：真实 dev 凭据 `live_smoke` 2 passed；隔离 Docker runner 亦有归档 |
| metrics | 进程内有界计数 `TaosMetricsSnapshot`（非 OTLP） |
| selfcheck | `taosx::selfcheck`（§6.7 共 9 项；`tmq_subscribe` Skipped） |
| 仍 NO-GO | Native SQL / FFI / WS auth 长会话 / 自动幂等重试 / HA / package stable / TMQ 客户端 / 远程 RED 导出 |

## 对齐矩阵

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| TAOSX-1–8 | member…SSOT | PASS | `.agents/ssot/adapters/storage/taos/` |
| TAOSX-9 | package stable | OPEN | 禁止宣称 |
| TAOSX-10 | 批量写 | PASS | `write_batch*` / `build_insert_sql_chunks` |
| TAOSX-10b | 部分成功报告 | PASS | `BatchWriteReport` / `write_batch_chunked_outcome`；无自动重试 |
| TAOSX-11 | WS reachability | PARTIAL | `src/native.rs`；不执行 SQL、不证明认证 |
| TAOSX-12 | 资源/close | PASS | 硬上限 + RAII/drain/取消测试 |
| TAOSX-13 | Decimal 无损 | PASS | NCHAR schema gate + scale=18 离线/live |
| TAOSX-14 | Native SQL / FFI / HA | NO-GO | 无实现、无证据 |
| TAOSX-15 | 幂等自动重试 | NO-GO | 报告可定位，但**不**自动重试 |
| TAOSX-16 | 十轮审查矩阵 | PASS | `docs/report/2026-07-23/taosx-ten-round-review.md` |
| TAOSX-17 | 公开 API 表面测试 | PASS | `src/lib.rs` `public_api_surface` |
| TAOSX-18 | 有界 metrics | PASS | `TaosMetricsSnapshot` / live metrics 冒烟 |
| TAOSX-19 | health/readiness | PASS | `TaosHealth` / `health` / `liveness` |
| TAOSX-20 | 模块自验证 | PASS | `taosx::selfcheck` catalog 9；`tmq` 诚实 Skipped |
| TAOSX-11b | WS live 握手 | PASS | `live_native_ws_handshake`（仍非 SQL 会话） |

## 诚实边界

- Native WS 当前仅为 **握手与关闭可达性探测**；所有 SQL 都走 REST。
- 真实 live 使用 `scripts/live/export-foundationx-env.sh --env dev` 注入凭据；**禁止**密钥入库。
- 存量 DOUBLE stable 必须迁移后才能使用；本 crate 不自动迁移。
- Draft §2.1「仅 HashMap scaffold」**过时**；默认路径为 REST 生产客户端。

## 验证

```bash
cargo test -p taosx --all-targets
cargo clippy -p taosx --all-targets -- -D warnings
# 真实 dev live（密钥仅进子进程）
scripts/live/export-foundationx-env.sh --env dev -- \
  cargo test -p taosx --test live_smoke -- --ignored
# 自验证 live
scripts/live/export-foundationx-env.sh --env dev -- \
  cargo test -p taosx --test live_selfcheck -- --ignored
# 隔离 Docker live（非 prod）
node scripts/taos-live-conformance.mjs
# 有界 bench
scripts/live/export-foundationx-env.sh --env dev -- \
  cargo bench -p taosx --bench hot_path -- --quick
```

## 相关

- [adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- [gap-matrix.md](./gap-matrix.md)
- [十轮审查](../report/2026-07-23/taosx-ten-round-review.md)
