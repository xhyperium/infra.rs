# 审计报告索引 — 2026-07-22

十轮 `crates/` SSOT 完整性 · 生产就绪 · 量化场景审查。

| 文档 | 说明 |
|------|------|
| [production-readiness-criteria.md](./production-readiness-criteria.md) | **生产条件标准**（L1–L5 · S1–S7 · QT · QT-Ship） |
| [crate-inventory.md](./crate-inventory.md) | 22 package ↔ SSOT/对齐映射 |
| [review-prompt.md](./review-prompt.md) | **全模块代码审查执行提示**（D1–D9、专项清单、门禁） |
| [review-workspace.md](./review-workspace.md) | **本轮全 workspace 集成审查**、门禁结果、P0–P3、QT-Ship 裁定 |
| [round-01/review.md](./round-01/review.md) | R1 基线扫描 |
| [round-02/review.md](./round-02/review.md) | R2 正确性 |
| [round-03/review.md](./round-03/review.md) | R3 契约 |
| [round-04/review.md](./round-04/review.md) | R4 兼容性 |
| [round-05/review.md](./round-05/review.md) | R5 可运维 |
| [round-06/review.md](./round-06/review.md) | R6 安全性 |
| [round-07/review.md](./round-07/review.md) | R7 量化场景 |
| [round-08/review.md](./round-08/review.md) | R8 集成风险 |
| [round-09/review.md](./round-09/review.md) | R9 DEFER 累积 |
| [round-10/review.md](./round-10/review.md) | R10 对抗与终裁 |
| [synthesis/go-nogo-synthesis.md](./synthesis/go-nogo-synthesis.md) | **综合 Go/No-Go + 补齐 backlog** |

## 逐 package 报告

| 分组 | 报告 |
| --- | --- |
| L0 / Types / L1 | [kernel](./review-kernel.md) · [testkit](./review-testkit.md) · [decimalx](./review-decimalx.md) · [canonical](./review-canonical.md) · [bootstrap](./review-bootstrap.md) · [configx](./review-configx.md) · [schedulex](./review-schedulex.md) · [evidence](./review-evidence.md) · [observex](./review-observex.md) · [resiliencx](./review-resiliencx.md) · [transportx](./review-transportx.md) |
| Contracts / Test Support | [contracts](./review-contracts.md) · [contract-testkit](./review-contract-testkit.md) |
| Exchange adapters | [binancex](./review-binancex.md) · [okxx](./review-okxx.md) |
| Storage adapters | [redisx](./review-redisx.md) · [postgresx](./review-postgresx.md) · [kafkax](./review-kafkax.md) · [natsx](./review-natsx.md) · [ossx](./review-ossx.md) · [clickhousex](./review-clickhousex.md) · [taosx](./review-taosx.md) |
| Tools | [goalctl](./review-goalctl.md) · [verifyctl](./review-verifyctl.md) |

> **总裁定：workspace 生产发布 = NO-GO；量化端到端 = NO-GO；13 包声明层 code+test = GO；局部内部库语义 = 有条件 GO。**
> **本轮执行补充：** 工程门禁均通过，但 live backend 测试未运行；交易、消息可靠语义、真实组合根和 L5 仍不能宣称完成。
> **defer-close 增量：** [`../2026-07-22-defer-close/`](../2026-07-22-defer-close/)（OBJECTIVE DEFER 关闭复核；archgate OOS-Accept）
> 前序：[`../2026-07-21/`](../2026-07-21/)
> 完成度看板：[`../../../STATUS.md`](../../../STATUS.md)（结构完成度 ≠ 生产就绪）
