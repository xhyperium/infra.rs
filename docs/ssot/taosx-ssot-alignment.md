# taosx SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `taosx` |
| SSOT | `.agents/ssot/adapters/storage/taos/` |
| 实现 | `crates/adapters/storage/taos` |
| 审计日期 | 2026-07-23 |
| version | `0.3.2` |
| 结论 | **受限 REST SQL + WS reachability 已落地**；Native SQL / HA / 幂等重试 / package stable **NO-GO** |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 生产默认面 | `TaosPool / TaosClient` REST（6041） |
| batch write | `write_batch` / `write_batch_chunked` / `build_insert_sql_chunks` |
| WS | `TransportMode::NativeWs` + `connect_native_ws`（仅握手/关闭探测；SQL 始终 REST） |
| Decimal | NCHAR(64+) 文本往返；DESCRIBE 拒绝存量 DOUBLE schema |
| 资源 | response / SQL batch / query rows / in-flight / close drain 均有硬上限 |
| 安全 | 远程 TLS/auth fail-closed；strict host；REST redirect 禁止；密码 Debug 脱敏 |
| contracts | `TimeSeriesStore`（ts 纳秒 epoch） |
| 环境变量 | `FOUNDATIONX_TAOSX_{HOST,PORT,USER,PASSWORD,DATABASE,TLS,PRECISION,TRANSPORT,...}` |
| live | 2026-07-23 固定 digest 隔离 runner：2 passed / 0 failed / 0 ignored，exit 0 |
| 仍 NO-GO | Native SQL / FFI / WS auth 长会话 / 自动幂等重试 / HA / package stable |

## 对齐矩阵

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| TAOSX-1–8 | member…SSOT | PASS | — |
| TAOSX-9 | package stable | OPEN | 禁止宣称 |
| TAOSX-10 | 批量写 | PASS | `write_batch*` / `build_insert_sql_chunks` |
| TAOSX-11 | WS reachability | PARTIAL | `src/native.rs`；不执行 SQL、不证明认证 |
| TAOSX-12 | 资源/close | PASS | 硬上限 + RAII/drain/取消测试 |
| TAOSX-13 | Decimal 无损 | PASS | NCHAR schema gate + scale=18 离线/live 场景 |
| TAOSX-14 | Native SQL / FFI / HA | NO-GO | 无实现、无证据 |
| TAOSX-15 | 幂等自动重试 | NO-GO | 多 chunk 部分成功语义未闭合 |

## 诚实边界

- Native WS 当前仅为 **握手与关闭可达性探测**；所有 SQL 都走 REST。
- 本轮只运行固定 digest 的动态 loopback 隔离容器，未运行 prod；失败与成功尝试均归档。
- 存量 DOUBLE stable 必须迁移后才能使用；本 crate 不自动迁移。

## 验证

```bash
cargo test -p taosx --all-targets
cargo clippy -p taosx --all-targets -- -D warnings
node scripts/taos-live-conformance.mjs  # 可选隔离 live；非默认 CI
cmp .agents/ssot/adapters/storage/taos/spec/spec.md \
  .agents/ssot/adapters/storage/taos/spec/xhyper-taosx-complete-spec.md
```

## 相关

- [adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- [gap-matrix.md](./gap-matrix.md)
