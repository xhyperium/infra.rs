# EVID-KERNEL-002-DOWN-006 — tree-external sleep timing audit

| 字段 | 值 |
|------|-----|
| Residual | **RES-DOWN-006** |
| Date | 2026-07-14 |
| Status | **CLOSED**（全量审计 + 标注 + 处置记录） |
| Spec | SPEC-KERNEL-002 §11.2 / §16.3（lifecycle 正确性 ≠ sleep） |

## 审计命令

```text
rg -n "thread::sleep|tokio::time::sleep|std::thread::sleep" --type rust
```

工作区 `**/*.{rs,toml}` 二次核对 `sleep` 字面量。

## 全量命中表

| # | 路径 | 性质 | 处置 |
|---|------|------|------|
| 1 | `crates/kernel/src/clock.rs` ≈L214 | **unit test** — SystemClock monotonic 可测间隔 | **annotate** — `interval smoke only / not correctness proof` |
| 2 | `crates/kernel/tests/public_api.rs` ≈L22 | **integration smoke** — 公共面单调推进 | **annotate** — 同上；lifecycle 正确性见 loom |
| 3 | `crates/adapters/storage/redis/src/lib.rs` ≈L399 | **集成 TTL 测**（`#[ignore]`，需 REDIS） | **annotate** — TTL interval smoke；**非** lifecycle |
| 4 | `crates/resiliencx/src/lib.rs` ≈L57 | **生产代码** — 重试退避阻塞 sleep | **accept** — 非测试；注释已说明 w4 阻塞语义 |

**无其他** workspace Rust 命中。

## Kernel 正确性证明（lifecycle）

| 资产 | 角色 |
|------|------|
| `crates/kernel/tests/lifecycle_concurrency_loom.rs` | **正确性证明**（`cfg(loom)` 模型） |
| `crates/kernel/tests/lifecycle_concurrency.rs` | std 多 waiter / 1000-cycle 回归；**明确写「不用 sleep 作正确性证明」** |
| `crates/kernel/src/lifecycle.rs` unit | 自旋 `yield_now` 等待 waiter 进入路径；注释写明非 sleep 证明 |
| CI `kernel-loom` job | RES-LC-004 / RES-TEST-012 / RES-GATE-008 已 CLOSED |

**结论**：kernel 内 sleep **仅**出现在 clock 间隔烟雾；**lifecycle 正确性不以 sleep 为证明**。

## 下游误读风险

- redis TTL：可能被误读为「用 sleep 证并发」→ 已标注 **not lifecycle correctness proof**。
- resiliencx：生产退避，不是测试；不构成 correctness claim 污染。
- 无其他 tree-external 测试把 sleep 当作 lifecycle 证明。

## 代码改动（本 residual）

- `crates/kernel/src/clock.rs` — 模块/行内标注
- `crates/kernel/tests/public_api.rs` — 文档 + 行内标注
- `crates/adapters/storage/redis/src/lib.rs` — 文档 + 行内标注

## Verdict

- 审计完整；命中少且清晰；关键路径已标注。
- **RES-DOWN-006 → CLOSED**。
- 不宣称 §18 / registry stable；不 bump 版本。
