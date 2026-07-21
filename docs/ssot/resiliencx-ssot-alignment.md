# resiliencx SSOT 对齐（infra.rs）

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-21 |
| Active SSOT | `.agents/ssot/resiliencx/spec/spec.md` |
| 用户路径别名 | `.agent/ssot/resiliencx` → `.agents/ssot/resiliencx` |
| 实现 | `crates/resiliencx` · package `resiliencx`（文档/产品名可写 xhyper-resiliencx） |

## 结论

| 能力 | 状态 | 证据 |
|------|------|------|
| 重试 §2 | **PASS** | `retry.rs` + `tests/retry_contract.rs` |
| 熔断 | **PASS**（本仓扩展；无墙钟） | `circuit.rs` + unit/public_api |
| 限流（令牌桶） | **PASS**（本仓扩展；显式 refill） | `rate_limit.rs` + unit/public_api |
| 舱壁（bulkhead） | **PASS**（并发上限；RAII） | `bulkhead.rs` + unit/public_api |
| Instrumentation | **PASS** | re-export `contracts::Instrumentation`；禁止 observex |
| LCOV 行 100% | **PASS** | `cov-gate-100.mjs -p resiliencx` |
| async wait | **PASS**（infra-s9t.6 / #167） | `retry_async` + `AsyncWait`；feature `tokio` → `TokioSleepWait` |
| backoff / budget / stable | **DEFER** | residual OPEN |

## 熔断合同（本仓）

- 状态：`Closed` / `Open` / `HalfOpen`
- `failure_threshold` 连续失败 → Open + `record_circuit_open`
- Open 下累计拒绝 `open_to_half_open_after_rejects` 次 → HalfOpen（**非**墙钟冷却）
- HalfOpen 连续成功 `success_threshold` → Closed + `record_circuit_close`；失败 → Open
- 配置阈值为 0 → `Invalid`；Open 拒绝 → `Unavailable`

## 限流合同（本仓）

- 满桶起步；`try_acquire(n)` 不足 → `Unavailable`（不部分扣减）
- `refill(n)` 不超过 capacity；**不**按时间自动补充

## 舱壁合同（本仓）

- `max_concurrent >= 1`；否则 `Invalid`
- `try_enter` / `call`：在途达上限 → `Unavailable("bulkhead full")`
- `BulkheadPermit` drop 归还槽位（含错误路径）
- **无**排队、**无**超时等待

## 验证

```bash
cargo test -p resiliencx --all-targets
cargo test -p resiliencx --features tokio --all-targets
cargo clippy -p resiliencx --all-targets -- -D warnings
node scripts/quality-gates/cov-gate-100.mjs -p resiliencx --filter crates/resiliencx/src
cargo tree -p resiliencx -i observex  # 须无匹配
```

## 双栏落地（2026-07-22 · STATUS 100% structure）

| 标尺 | 状态 |
|------|------|
| STATUS 结构完成度 | **100%**（layout+tests+content；非 Production Ready） |
| 声明面生产硬化 | 公共 API 集成测 + 热路径 bench + `docs/` 红线；**cov-gate-100 行覆盖** |
| 非宣称 | **禁止** workspace Production Ready / Agent L5 / 扩大 SSOT DEFER 平台面 |

自验证：`cargo test -p resiliencx --all-targets`；`node scripts/quality-gates/cov-gate-100.mjs -p resiliencx`；`cargo run -p resiliencx --example …`；`cargo bench -p resiliencx --bench hot_path -- --quick`。
