# types/canonical — Gate

| 字段 | 值 |
|------|-----|
| 状态 | **agent-safe PASS** · package stable **BLOCKED** |
| 更新 | 2026-07-21 |

| # | 门禁 | 裁决 |
|---|------|------|
| G1 | `cargo test -p xhyper-canonical -p xhyper-decimalx` | **PASS** |
| G2 | clippy `-D warnings` | **PASS** |
| G3 | fmt `--check` | **PASS** |
| G4 | dual-mirror cmp | **PASS** |
| G5 | 无 OrderId 类型 / reverse deps / f32 字段 | **PASS** |
| G6 | package stable | **BLOCKED** HUMAN S2 |
