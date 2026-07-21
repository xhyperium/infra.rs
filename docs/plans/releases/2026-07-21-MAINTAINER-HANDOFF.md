# Maintainer 签核交接（PLAN-CORE-PROD-002）

> **一页纸** · 2026-07-21 · Agent 准备 · **不构成批准**

## 你现在要做什么

1. 打开 [2026-07-21-signoff-DRAFT.md](./2026-07-21-signoff-DRAFT.md)
2. 快速过 L1–L4 证据（Agent 已跑本地：`test` / `clippy` / `deny` / `public-api` / panicking 门禁 / canonical-align）
3. **必读 Accept 风险**：真实后端 mock-only、二期 trait、仅 Linux、无完整 cargo-fuzz
4. 在 **Maintainer 签核区** 填写：
   - 推荐结论：**GO-with-Accepts**（实现闭环已合入；Accept 风险已知）
   - 或 **NO-GO**（若你认为 mock-L3 不足以进入签核）
5. 将 DRAFT 复制为正式文件（建议名）并提交 PR：

```text
docs/plans/releases/0.3.0-signoff.md   # 或 v0.3.0-signoff.md
```

6. 勾选计划 §10.6 DEFER-7；关闭计划 §15 DONE（可选 follow-up docs PR）

## 不要做什么

- 不要让 Agent 代写 `Signed-off-by` / 结论字段
- 不要在未签核时把 README 改成「Production Ready」
- 不要把 mock adapter 表述为已对接生产后端

## 关键链接

| 文档 | 用途 |
|------|------|
| [signoff DRAFT](./2026-07-21-signoff-DRAFT.md) | 证据 + 签核区 |
| [计划 §15](../2026-07-21-core-crates-production-readiness.md) | DONE 条件（差人签） |
| [DEFER 处置](../artifacts/defer-disposition.md) | 8 项 Close/Accept |
| [审计 §12](../../report/2026-07-21/core-crates-production-readiness.md) | post-W5 状态 |
| [支持矩阵](../../governance/support-matrix.md) | Linux + MSRV 1.85 |

## 建议结论口径（供参考，非代签）

```text
GO-with-Accepts：
  - 五核心 W0–W5 实现已合入 main
  - L1/L2/mock-L3/L4 门禁证据充分
  - Accept：真实后端、二期 trait、非 Linux、轻量 fuzz
  - 禁止整体 Production Ready 对外营销直至本签核落盘
```
