# Tasks — PLAN-TYPES-DECIMALX-002-agent-safe-v1

| Task ID | GAP | 描述 | Paths | Disposition | Evidence 约定 |
|---------|-----|------|-------|-------------|---------------|
| T-DOC-001 | GAP-001 | 落盘 plan/gap/tasks/residual/CURRENT-STATE/alignment | `plan/**` | AGENT_SAFE | 文件存在 |
| T-DOC-002 | — | 建立 `todo.md` 台账与 disposition 规则 | `todo.md` | AGENT_SAFE | 全行终态 |
| T-DOC-003 | GAP-002 | 修正 active spec 候选链接 → `20260717/` | `decimalx-spec.md` | AGENT_SAFE | 链接可解析 |
| T-DOC-004 | GAP-006 | README/CHANGELOG 对齐 checked 主路径与 Draft 边界 | `README.md` · `CHANGELOG.md` | AGENT_SAFE | 文案 |
| T-M0-001 | GAP-003 | Consumer/API/wire inventory 落盘 | `plan/evidence/*` | AGENT_SAFE | inventory txt |
| T-M0-002 | GAP-004 | 边界测试补强（parse/Display/cmp/Hash/Currency） | `src/lib.rs` tests · fuzz | AGENT_SAFE | cargo test |
| T-M0-003 | GAP-016 | float/panic 扫描摘要 | evidence | AGENT_SAFE | rg 输出 |
| T-M2-001 | GAP-005 | panicking API `# Panics` rustdoc | `src/lib.rs` | AGENT_SAFE | rustdoc 文本 |
| T-VER-001 | gates | cargo test/check/clippy/fmt -p xhyper-decimalx | — | AGENT_SAFE | SCRATCH logs |
| T-VER-002 | 10x | 十轮检查 fail_rounds=0 | 10x-verdict · scripts | AGENT_SAFE | 10x logs |
| T-VER-003 | approve | PR + liukongqiang5 APPROVE readback | GitHub | AGENT_SAFE* | readback JSON |
| T-HUM-001 | GAP-007 | 批准 MAX_SCALE / DecimalLimits | — | HUMAN_ONLY | residual |
| T-HUM-002 | GAP-008 | 字段私有化兼容计划 | — | HUMAN_ONLY | residual |
| T-HUM-003 | GAP-009 | DecimalError / 错误升格 | — | HUMAN_ONLY | residual |
| T-HUM-004 | GAP-011 | wire/storage 稳定范围批准 | — | HUMAN_ONLY | residual |
| T-HUM-005 | GAP-015 | Spec Approved / Goal Achieved | — | HUMAN_ONLY | residual |
| T-DEF-001 | GAP-010 | 除法 target_scale API（需 consumer） | — | DEFERRED | residual |
| T-DEF-002 | GAP-012 | panicking API deprecate/删除 + 全迁移 | — | DEFERRED | residual |
| T-DEF-003 | GAP-013 | 全 i128/u8 property/differential | — | DEFERRED | residual |
| T-POL-001 | GAP-014 | 禁止 numeric 路径/环依赖回流 | ADR-007 | POLICY | residual |

\* T-VER-003 失败则 env-limit 诚实记录，不伪造 APPROVE。
