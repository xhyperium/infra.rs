# CURRENT-STATE — decimalx agent-safe campaign

| 字段 | 值 |
|------|-----|
| Plan | `PLAN-TYPES-DECIMALX-002-agent-safe-v1` |
| Branch | `fix/types-decimalx-agent-safe-20260717` |
| Package | `xhyper-decimalx` **0.1.0** |
| Active SSOT | `.agent/SSOT/types/decimal/decimalx-spec.md` |
| Candidate | `20260717/` Draft · **Non-normative** · 相对链接 `../decimalx-spec.md` |
| Campaign | **DONE (agent-safe)** · 10x PASS · **≠** Goal ACHIEVED · **≠** Spec Approved |
| content_tip (10x first) | `1c53304a08db534fcff1ce8fe03aeacb127aa2ae` |
| PR | [#507](https://github.com/xhyperium/xhyper.rs/pull/507) |
| Approval tip SSOT | 本地 SCRATCH 指针，未入库，reviewer/未来 agent 不可见：`/tmp/grok-goal-99a109d2452b/implementer/approval/approval-readback.json`（对应当前 HEAD 的 tip-bound 读回；**非 durable 证据**，不得当作仓库内 APPROVED） |
| plan/evidence 审批文件 | `evidence/approval-readback-pr507.json` 为 **POINTER_NOT_TIP_BOUND**（禁止当作 tip-bound APPROVED） |
| Goal/Spec | **仍为 Draft** · 非 Achieved · 非 Approved |
| Wire | **未 stable** |
| Snapshot note | 对账日 2026-07-17；实现 tip 以 `git rev-parse HEAD` 为准 |

## 已固定事实

1. 路径 `crates/types/decimal`；依赖 `kernel` + `serde`；无 `canonical` 反向依赖。
2. ADR-006 五种舍入 + checked 算术已实现；operators panic on overflow。
3. 字段公开（见 T-HUM-002）；`MAX_SCALE` **治理层未批准，但代码已强制**。`crates/types/decimal/src/lib.rs:27` 定义 `pub const MAX_SCALE: u8 = 18`，且 `try_new`/`FromStr`/`checked_*` 将其作为**安全不变量强制**——任何 `scale>18` 均被拒绝（见 `:60`、`:374`、`:214`、`:256`）。这里的「未批准」指治理层对该常量取值 `18` 的正式 sign-off 仍待定（见 T-HUM-001 / `plan/residual-open.md`），**并非**代码未强制。
4. Consumers 见 `evidence/m0-consumer-inventory-2026-07-17.txt`。
5. Draft 归位 `20260717/` 后 Active 链接为 `../decimalx-spec.md`（非 `../specs/types/decimal/...`）。

## 本战役交付

- plan 包 + todo 台账
- M0 inventory + 边界测试补强
- panicking API `# Panics` + README 对齐
- 聚焦门禁 + 10x + approve 读回（tip SSOT = SCRATCH）

## 禁止宣称

- Spec Approved / Goal Achieved / package stable / wire stable
- 全量 M1–M3 生产迁移已完成
- 10x PASS = 实现全量闭合
- 把 `plan/evidence/approval-readback-pr507.json` 当成 tip-bound APPROVED

## infra.rs 落地（2026-07-21）

| 字段 | 值 |
|------|-----|
| 仓库 | `infra.rs` |
| 分支 | `feat/decimal-ssot-align`（对账回合） |
| 实现路径 | `crates/types/decimal`（workspace member；package `xhyper-decimalx` 0.1.0） |
| Active SSOT 路径 | `.agents/ssot/types/decimal/spec/spec.md`（dual mirror `cmp` exit 0） |
| 对账证据 | 本地 SCRATCH `decimal-ssot-alignment.md`（**非** Goal Achieved） |
| agent-safe 对账 | **完成**：§3 API 列表/§6 测试计数同步；entry 补强 sub/mul/rescale/newtype |
| 门禁 | `cargo test -p xhyper-decimalx` · fmt · clippy `-D warnings` · 依赖仅 kernel+serde |
| residual | T-HUM-001..005 / T-DEF-001..003 / T-POL-001 **仍开放** |

说明：上表 monorepo 战役字段（PR #507 等）指 xhyper 历史证据；**infra.rs 以本段与 `review/review.md` 为准**。
