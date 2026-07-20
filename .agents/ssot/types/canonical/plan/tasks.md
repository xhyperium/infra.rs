# Tasks — PLAN-TYPES-CANONICAL-002

> **2026-07-21 标签对齐**：与 todo/residual 一致；T-10X=DEFERRED；T-APPR-002=HUMAN 历史；S1 DONE。

| 字段 | 值 |
|------|-----|
| Plan | `PLAN-TYPES-CANONICAL-002-v1` |
| Status | `TODO` · `IN_PROGRESS` · `DONE` · `BLOCKED` · `DEFER` · `CANCELLED` |
| Baseline | `main@4fe8e988` |

> DONE 必须有可复核输出；禁止无证据勾选。

---

## W0 — 台账 / Plan 包

| Task ID | 内容 | AC | Status |
|---------|------|-----|--------|
| T-PLAN-001 | 落盘 plan.md | 含深度分析、M0–M3、禁止项、门禁 | **DONE** |
| T-PLAN-002 | 落盘 gap-matrix.md | Goal/Spec 条款映射 | **DONE** |
| T-PLAN-003 | 落盘 spec-inventory.md | I-API/DEP/OPEN/WIRE/CONS/GATE/FORBID | **DONE** |
| T-PLAN-004 | 落盘 residual-open.md | OPEN/HUMAN/DEFER 全登记 | **DONE** |
| T-PLAN-005 | 落盘 approval-packet.md | 人审闸门清晰；AI 不独断 Approved | **DONE** |
| T-TODO-001 | 创建 todo.md | 全量 ID + 终态标签 | **DONE** |
| T-BRANCH-001 | feature 分支/worktree | 非 main：`docs/types-canonical-002-closure` | **DONE** |

---

## W1 — M0 事实闭合（agent-safe）

| Task ID | 内容 | AC | Status |
|---------|------|-----|--------|
| T-DOC-001 | 修 active/Goal/Spec 交叉链接 | 指向 `20260717/`；无 draft/ 死链 | **DONE** |
| T-DOC-002 | active 与源码 API 再核对；更新 snapshot tip | 16 类型表一致；Verified tip | **DONE** |
| T-DOC-003 | README 补齐 OrderRef/Cancel/Venue/Instrument | 与源码一致；非职责含禁 codec | **DONE** |
| T-DOC-004 | Goal §7 / Spec §8 完成勾选诚实化 | agent-safe 可勾；OPEN 不勾 | **DONE** |
| T-CONS-001 | consumer inventory 写入 inventory/todo | I-CONS-01…10 有证据 | **DONE** |
| T-TEST-001 | 全公开 DTO/枚举 serde round-trip（可序列化者） | 每类型至少 1 RT | **DONE** |
| T-TEST-002 | 全部 OrderStatus variants 可构造+serde | 6 variants | **DONE** |
| T-TEST-003 | OrderRef Client+Exchange 双向 | 两变体 | **DONE** |
| T-TEST-004 | cancel fixture 正反保持 | include_str fixture | **DONE** |
| T-TEST-005 | Money 与 decimalx::Money 类型同一 | 编译级/赋值断言 | **DONE** |
| T-TEST-006 | 无 f32/f64 金融字段静态检查（测试或文档+grep 证据） | 源码 grep 空 | **DONE** |
| T-CHG-001 | CHANGELOG [Unreleased] 记实 | 不宣称 Approved/stable | **DONE** |

---

## W2 — M2 wire 文档边界（agent-safe）

| Task ID | 内容 | AC | Status |
|---------|------|-----|--------|
| T-WIRE-001 | 文档标明：仅 cancel + legacy ack 有固定 wire 证据 | active §4 明确 | **DONE** |
| T-WIRE-002 | 其余类型 RT ≠ 跨版本 wire 承诺 | 文档一句话 | **DONE** |

---

## W3 — 门禁 / 下游 / 对齐

| Task ID | 内容 | AC | Status |
|---------|------|-----|--------|
| T-GATE-001 | `cargo test -p xhyper-canonical` | exit 0 | **DONE** |
| T-GATE-002 | check + clippy -D warnings | exit 0 | **DONE** |
| T-GATE-003 | `cargo xtl lint-deps` | exit 0 | **DONE** |
| T-GATE-004 | `cargo fmt -- --check` | exit 0 | **DONE** |
| T-DOWN-001 | 触及路径 consumers 编译（至少 contracts + binance/okx 或 domain 抽样） | 无因本改动失败 | **DONE** |
| T-ALIGN-001 | alignment-2026-07-17.md + evidence 目录 | tip + 路径列表 | **DONE** |

---

## W4 — 10x + 批准

| Task ID | 内容 | AC | Status |
|---------|------|-----|--------|
| T-10X-001 | 十轮固定命令；每轮日志；summary fail_rounds=0 | 整组重跑规则 | **DEFERRED**（本 clone 无 fresh `evidence/types-canonical-002/10x/`；= SAFE-15） |
| T-APPR-001 | PR 创建 | PR #508 | **DONE** |
| T-APPR-002 | `export LIUKONGQIANG5_APPROVE_TOKEN` + approve helper | API readback APPROVED on tip | **HUMAN_ONLY** 历史（xhyper PR；本仓不伪造；= SAFE-16） |

---

## W5 — 明确不在本战役（DEFER / HUMAN）

| Task ID | 内容 | Status |
|---------|------|--------|
| T-HUM-001 | CAN-ID/TIME/VALID 原则人审 | **DONE** T1/T2/T4；WIRE 全量仍 OPEN → residual |
| T-HUM-002 | Spec Draft → Approved S1 | **DONE**（2026-07-17；≠ package stable） |
| T-HUM-003 | package stable / crates.io | **HUMAN_ONLY** |
| T-HUM-004 | `OrderId` **类型已删**；legacy Order/OrderAck DTO 形状删除 | 类型 **DONE**；DTO 形状 **DEFER** 至 consumer=0 |
| T-HUM-005 | types/core·protocol 搬迁 RFC | **DEFER** |
| T-HUM-006 | 移除 serde | **DEFER** |
