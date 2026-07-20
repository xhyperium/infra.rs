> **Post-ship**：`publish` 现为 **true**（crates.io 包名 **`xhyper-kernel`**）；战役 COMPLETE。

# Retrospective — kernel 002 执行

| 字段 | 值 |
|------|-----|
| Ship PR | [#235](https://github.com/xhyperium/xhyper.rs/pull/235) |
| Date | 2026-07-14 |

## 学到的

1. E2 先 `kind()` 再 opaque，避免双重全仓 match 改写。
2. `SystemClock` 去 `Copy` 会波及 `Arc::new(SystemClock)` → 必须 `::new()`。
3. `from_clock_elapsed(now-elapsed)` 是静默正确性 bug；必须 `base.checked_add(elapsed)` + 回归测试。
4. Residual ID 必须用 **mid 原义** 登记 OPEN/CLOSED，禁止改义后假 CLOSED。
5. Cargo 字面合同（`publish=false`、`default=[]`）与「等价无 feature」不同，要显式写。
6. 十轮机械复检 + residual ledger 比散文「完成」可靠。

## 本波结果

| 项 | 结果 |
|----|------|
| 代码主路径 E1–E3/C/L/G1 | **PASS** |
| 对齐文档 + registry | **PASS** |
| 十轮 | **10/10** |
| §18 / stable | **OPEN / 禁止** |
| 下一门禁 | **G2** |
