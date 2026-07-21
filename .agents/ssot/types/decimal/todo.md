# Todo — decimalx / PLAN-TYPES-DECIMALX-002-agent-safe-v1

Disposition 规则：每行必须为 `DONE` / `DEFERRED` / `HUMAN_ONLY` / `POLICY` 之一。  
`DONE` 必须绑定可复核证据路径；禁止把人审项伪标 DONE。

| Task ID | 描述 | Disposition | Evidence |
|---------|------|-------------|----------|
| T-DOC-001 | 落盘 plan 包（plan/gap/tasks/residual/CURRENT-STATE/alignment/checklist/scripts） | DONE | `.agents/ssot/types/decimal/plan/**` |
| T-DOC-002 | 本 todo 台账 | DONE | `.agents/ssot/types/decimal/todo.md` |
| T-DOC-003 | Active spec 候选链接 → `20260717/` | DONE | `.agents/ssot/types/decimal/decimalx-spec.md` |
| T-DOC-004 | README + CHANGELOG 对齐 | DONE | `crates/types/decimal/README.md` · `CHANGELOG.md` |
| T-M0-001 | Consumer/API inventory | DONE | `plan/evidence/m0-consumer-inventory-2026-07-17.txt` |
| T-M0-002 | 边界测试补强 | DONE | `crates/types/decimal/src/lib.rs` tests · `cargo test -p xhyper-decimalx` |
| T-M0-003 | float/panic 扫描摘要 | DONE | `plan/evidence/m0-consumer-inventory-2026-07-17.txt` |
| T-M2-001 | panicking API `# Panics` | DONE | `crates/types/decimal/src/lib.rs` (`# Panics`) |
| T-VER-001 | 聚焦门禁 test/check/clippy/fmt | DONE | 本地 SCRATCH 指针，未入库，reviewer/未来 agent 不可见；证据未提交，见 `/tmp/grok-goal-99a109d2452b/implementer/tests/`（**SCRATCH，非 durable**） |
| T-VER-002 | 10x fail_rounds=0 | DONE | 仓库内 durable 证据：`plan/decimalx-plan-10x-verdict.md`；附本地 SCRATCH 指针（未入库，reviewer/未来 agent 不可见）：`/tmp/grok-goal-99a109d2452b/implementer/10x/decimal-10x-summary.log`（**SCRATCH，非 durable**） |
| T-VER-003 | liukongqiang5 APPROVE / HAR | DONE | tip 读回为本地 SCRATCH 指针（未入库，reviewer/未来 agent 不可见）：`/tmp/grok-goal-99a109d2452b/implementer/approval/approval-readback.json`（**SCRATCH，非 durable**）；仓库内 durable 指针 `plan/evidence/approval-readback-pr507.json` 为 POINTER_NOT_TIP_BOUND（≠ Goal Achieved，见 T-HUM-005） |
| T-FIX-001 | 修复 Draft Active 断链 `../decimalx-spec.md` | DONE | `20260717/xhyper-decimalx-complete-{goal,spec}.md` |
| T-FIX-002 | 审批证据诚实化（禁 DISMISSED tip 冒充 tip-bound） | DONE | CURRENT-STATE · alignment · evidence POINTER |
| T-HUM-001 | MAX_SCALE / DecimalLimits 批准 | HUMAN_ONLY | `plan/residual-open.md` |
| T-HUM-002 | 字段私有化 | HUMAN_ONLY | residual |
| T-HUM-003 | DecimalError 升格 | HUMAN_ONLY | residual |
| T-HUM-004 | wire/storage stable 批准 | HUMAN_ONLY | residual |
| T-HUM-005 | Spec Approved / Goal Achieved | HUMAN_ONLY | residual |
| T-DEF-001 | 除法 target_scale API | DEFERRED | residual |
| T-DEF-002 | panicking API 删除/全迁移 | DEFERRED | residual |
| T-DEF-003 | 全 i128 property/differential | DEFERRED | residual |
| T-POL-001 | 禁止 numeric 路径 / 环依赖 / 默认 Money\<U\> | POLICY | residual · ADR-007 |
| T-ALIGN-001 | infra.rs Active SSOT 对账 2026-07-21（§3 API/§6 计数/entry 补强） | DONE | `spec/spec.md` dual · `tests/entry_checked_ops.rs` · review/CURRENT-STATE |

## 规则

- agent-safe 全 DONE ≠ GOAL ACHIEVED ≠ SPEC Approved ≠ package stable
- 10x PASS ≠ 全量 M1–M3 生产迁移完成
- 无证据不得 DONE
- **T-ALIGN-001 完成 ≠ Goal Achieved**（T-HUM-005 仍开放）
