> **历史战役记录（2026-07-21，非当前权威）**：以下 PASS/命令绑定旧候选与旧 package 别名。当前 `canonical 0.1.2` 合同和门禁见 [../README.md](../README.md) 与 [../gate/gate.md](../gate/gate.md)。

# types/canonical — Goal

| 字段 | 值 |
|------|-----|
| Goal ID | `GOAL-TYPES-CANONICAL-002` |
| Complete | [../20260717/canonical-complete-goal.md](../20260717/canonical-complete-goal.md) |
| 状态 | **agent-safe 表面 PASS** · package stable / 全 wire **未宣称 ACHIEVED** |
| 更新 | 2026-07-21 |

## AC

| AC | 裁决 | 证据 |
|----|------|------|
| 1:1 alignment 矩阵 | **PASS** | [plan/alignment-matrix-infra-2026-07-21.md](../plan/alignment-matrix-infra-2026-07-21.md) |
| OrderId/ts/S1 无矛盾 | **PASS** | [spec/spec.md](../spec/spec.md) |
| workspace 可测 | **PASS** | `crates/types/canonical` + decimal |
| 单测/fixtures | **PASS** | `cargo test -p xhyper-canonical` |
| residual 诚实 | **PASS** | [plan/residual-open.md](../plan/residual-open.md) |
| package stable | **OPEN/HUMAN** | DEFER-STABLE |
| 全 wire 冻结 | **OPEN** | OPEN-WIRE-002 |
