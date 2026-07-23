# taosx SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `taosx` |
| SSOT | `.agents/ssot/adapters/storage/taos/`（**禁止**平行 `taosx/` 树） |
| 实现 | `crates/adapters/storage/taos` |
| 审计日期 | 2026-07-23 |
| version | `0.3.10` |
| 结论 | **Production-default REST 全 API 面已闭合**（stream/batcher/retry/TMQ/metrics 导出/soak/HA-lite/selfcheck Full）；gap register **未完成=0** |
| 档次 | Production-default（`publish=false`；非 crates.io package-stable 产品宣告） |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 生产默认面 | `TaosPool` REST + 可选 NativeWs 握手/SQL 短会话 |
| 写查合同 | `TimeSeriesStore` + `write_batch*` / `write_batch_idempotent` |
| 流/批 | `TaosQueryStream` / `WriteBatcher` |
| TMQ | `TmqConsumer`（主题 DDL + 水位轮询；CREATE TOPIC 失败可降级） |
| metrics | `TaosMetricsSnapshot` + Prometheus 文本 |
| health | `liveness` / `health` |
| selfcheck | §6.7 共 9 项 Full live Passed/Degraded |
| soak | `run_soak` → `/home/workspace/data/taosx/soak` |
| HA-lite | `config.hosts` 故障转移 |
| gap 清零表 | [taosx-gap-register.md](./taosx-gap-register.md) |

## 对齐矩阵

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| TAOSX-1–8 | member…SSOT | PASS | `.agents/ssot/adapters/storage/taos/` |
| TAOSX-9 | package 档次 | SUPERSEDED | Production-default；见 gap register O-01 |
| TAOSX-10 | 批量写 | PASS | `write_batch*` / report / idempotent |
| TAOSX-11 | WS | PASS | handshake + `exec_sql_ws` |
| TAOSX-12 | 资源/close | PASS | 硬上限 + drain |
| TAOSX-13 | Decimal | PASS | NCHAR gate |
| TAOSX-14 | Native | SUPERSEDED | `probe_native_tcp` + REST SQL |
| TAOSX-15 | 幂等重试 | PASS | `write_batch_idempotent` |
| TAOSX-16–20 | 审查/API/metrics/health/selfcheck | PASS | 测试 + live |
| TAOSX-21 | gap-zero | PASS | gap register 0 未完成 |

## 验证

```bash
cargo test -p taosx --all-targets
cargo clippy -p taosx --all-targets -- -D warnings
scripts/live/export-foundationx-env.sh --env dev -- \
  cargo test -p taosx --test integration_all_api --test e2e_klines --test live_selfcheck -- --ignored
scripts/live/export-foundationx-env.sh --env dev -- \
  cargo bench -p taosx --bench api_matrix
```

## 相关

- [taosx-gap-register.md](./taosx-gap-register.md)
- [adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- [gap-matrix.md](./gap-matrix.md)
