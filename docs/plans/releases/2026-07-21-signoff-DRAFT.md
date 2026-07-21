# 生产签核草稿 — PLAN-CORE-PROD-002（**已归档**）

> **状态：SUPERSEDED**  
> 正式签核已落盘：[`0.3.0-signoff.md`](./0.3.0-signoff.md)（**GO-with-Accepts** · `@ZoneCNH` · 2026-07-21）  
> 本文件仅保留草稿溯源；**勿**再当作未签核入口，**勿**双写签核结论。  
> 交接摘要（历史）：[`2026-07-21-MAINTAINER-HANDOFF.md`](./2026-07-21-MAINTAINER-HANDOFF.md)

---

## 元信息（归档）

| 字段 | 值 |
|------|----|
| 版本 / Tag | 见正式包 `0.3.0`（`publish = false`） |
| 关联 | PLAN-CORE-PROD-002 · beads `infra-asa` · PR #98 #120 #121 #124 #125 #127 #128 #138 #141 #143 |
| 支持矩阵 | [`support-matrix.md`](../../governance/support-matrix.md)：Linux x86_64 + MSRV 1.85 |
| 草稿日期 | 2026-07-21 |
| 证据填充会话 | 2026-07-21 · PR #143 · 基线 `b0154db`（历史） |
| 取代文件 | [`0.3.0-signoff.md`](./0.3.0-signoff.md) |
| 签核结论 | 见正式包：**GO-with-Accepts** |

---

## 红线（历史）

```text
Maintainer only — Agent must not sign.
草稿阶段禁止当作 Production Ready 批准。
正式签核后以 0.3.0-signoff.md 为唯一权威。
最终签核必须落在 docs/plans/releases/<version>-signoff.md（非 *DRAFT* 文件名）。
```

---

## 已合入实现证据（历史指针）

| 波次 | PR | squash | 合入摘要 |
|------|-----|--------|----------|
| 基线 | [#98](https://github.com/xhyperium/infra.rs/pull/98) | `76c56d7` | 五核心 P0/P1 可机器验证子集 |
| W0 | [#120](https://github.com/xhyperium/infra.rs/pull/120) | `3b82fe7` | 计划 + artifacts 冻结 |
| W1 | [#121](https://github.com/xhyperium/infra.rs/pull/121) | `0e01f97` | decimalx oracle / 边界 / panicking 门禁 / scheduled miri·mutants |
| W2 | [#124](https://github.com/xhyperium/infra.rs/pull/124) | `ee45d97` | canonical committed wire v1.1–v1.3 |
| W3 | [#128](https://github.com/xhyperium/infra.rs/pull/128) | `d72dcc4` | contracts 语义文档 + fakes + venue override 门禁 |
| W4 | [#125](https://github.com/xhyperium/infra.rs/pull/125) | `10954c3` | adapters **离线 mock** 验证入口 |
| W5 | [#127](https://github.com/xhyperium/infra.rs/pull/127) | `f214eeb` | support-matrix / public-api baselines / 签核模板 |
| 收尾 | [#138](https://github.com/xhyperium/infra.rs/pull/138) | `bbdb083` | 计划状态 + 本 DRAFT 初版 |
| 勾选回写 | [#141](https://github.com/xhyperium/infra.rs/pull/141) | `b0154db` | 验收勾选 + 审计报告 §12 |
| 证据填充 | [#143](https://github.com/xhyperium/infra.rs/pull/143) | 见 main | DRAFT 证据 + Maintainer 交接页 |

完整证据与勾选见 [`0.3.0-signoff.md`](./0.3.0-signoff.md)。

DEFER 表：[`../artifacts/defer-disposition.md`](../artifacts/defer-disposition.md)  
计划 DONE 状态：[`../2026-07-21-core-crates-production-readiness.md`](../2026-07-21-core-crates-production-readiness.md) §15  
审计 post-W5：[`../../report/2026-07-21/core-crates-production-readiness.md`](../../report/2026-07-21/core-crates-production-readiness.md) §12

---

## 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-21 | 初版 DRAFT（#138） |
| 2026-07-21 | 证据填充 + 交接页（#143） |
| 2026-07-21 | 正式签核 `0.3.0-signoff.md` 落盘后本文件标 SUPERSEDED |
