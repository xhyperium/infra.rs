# Maintainer 签核交接（PLAN-CORE-PROD-002）

> **一页纸** · 2026-07-21 · Agent 准备 · **不构成批准**

## 你现在要做什么

1. 阅读 [2026-07-21-signoff-DRAFT.md](./2026-07-21-signoff-DRAFT.md) 中的 **证据** 与 **Accept 风险**
2. **复制** DRAFT → 正式文件（**必须改名，禁止在 DRAFT 上终签**）。示例：

   `cp docs/plans/releases/2026-07-21-signoff-DRAFT.md docs/plans/releases/0.3.0-signoff.md`

   然后编辑 `0.3.0-signoff.md`：去掉 DRAFT 标题/状态，填写结论与签名。
3. 在正式文件中：勾选 L1–L5（你认可的项）；**自行**选择结论：`GO` / `NO-GO` / `GO-with-Accepts`（Agent **不**预选）；按模板签名：`Signed-off-by: @handle  YYYY-MM-DD`
4. 为 **MSRV 1.85 @ 当前基线** 补一条证据（CI URL 或本地 `rust-version` 复跑）— DRAFT 已标明 Agent 未复跑 1.85
5. 开 PR 合入正式 signoff；可选：勾选计划 §10.6 DEFER-7、更新 §15 DONE

## 不要做什么

- 不要在 `*-DRAFT.md` 文件上写最终 `Signed-off-by`
- 不要让 Agent 代写结论或签名
- 不要在未签核时把 README 改成「Production Ready」
- 不要把 mock adapter 表述为已对接生产后端

## 关键链接

| 文档 | 用途 |
|------|------|
| [signoff DRAFT](./2026-07-21-signoff-DRAFT.md) | 证据包（未签核） |
| [计划 §15](../2026-07-21-core-crates-production-readiness.md) | DONE 条件（差人签） |
| [DEFER 处置](../artifacts/defer-disposition.md) | 8 项 Close/Accept |
| [审计 §12](../../report/2026-07-21/core-crates-production-readiness.md) | post-W5 状态 |
| [支持矩阵](../../governance/support-matrix.md) | Linux + MSRV 1.85 |
| [签核模板](../../governance/prod-signoff-TEMPLATE.md) | 正式签名格式 SSOT |

## 结论选项（中性列举 · 非推荐）

| 结论 | 含义 |
|------|------|
| **GO** | 接受当前证据与 Accept 风险，允许内部 Release Ready 声明 |
| **GO-with-Accepts** | 同上，但必须在签核文件中列出 Accept 清单 |
| **NO-GO** | 阻塞；写明缺口（例如要求真实后端后再签） |
