# Prompt — SPEC-KERNEL-002

| 字段 | 值 |
|------|-----|
| Source Spec | `SPEC-KERNEL-002` |
| Ship PR | [#235](https://github.com/xhyperium/infra.rs/pull/235) |
| Next | 人审 PR #235；Spec Approved 决策；可选 trybuild/miri/mutants |
| Residual | [residual-open.txt](../evidence/2026-07-14/residual-open.txt) |
| G2 证据 | [G2-tests](../evidence/2026-07-14/EVID-KERNEL-002-G2-tests.md) · [G2-archgate-ci](../evidence/2026-07-14/EVID-KERNEL-002-G2-archgate-ci.md) · [CLK009/TEST004](../evidence/2026-07-14/EVID-KERNEL-002-CLK009-TEST004.md) · [CI-98af7c9c](../evidence/2026-07-14/EVID-KERNEL-002-CI-98af7c9c.md) |

## 已完成（勿重复）

E1–E3 / C1–C2 / L1–L2 / G1 / G2（测试轨 + 历史 monorepo archgate 证据链 + CI loom；**infra.rs 不要求 archgate**）/
RES-CLK-009 / RES-TEST-004 / Phase D 文档对齐 / residual mid 卫生。

## 禁止回退

- `not_found` / `other`
- `Clock::monotonic` 默认 `Instant::now`
- 公开 `Component` trait
- registry `stable` 在 §18 前
- 假 Done / 手写 PASS 代替命令输出
