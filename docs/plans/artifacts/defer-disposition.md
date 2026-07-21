# DEFER 处置表（W0 冻结）

| 字段 | 值 |
|------|-----|
| Plan | [PLAN-CORE-PROD-002](../2026-07-21-core-crates-production-readiness.md) §11 |
| 审计 | [core-crates-production-readiness.md](../../report/2026-07-21/core-crates-production-readiness.md) §11.2 |
| Beads | `infra-asa.1` |
| 冻结日期 | 2026-07-21 |
| 规则 | 每项必须为 **Close** / **Accept** / **Defer-with-sign** 之一；禁止「未分类」 |

## 处置定义

| 标签 | 含义 |
|------|------|
| **Close** | 本轮必须完成并可勾掉；对应 W 波次验收 |
| **Accept** | 本轮接受残留；写入签字包风险清单，不阻塞目标层级（须 Maintainer 知悉） |
| **Defer-with-sign** | 仅能在 L5 签字包中由 **人工** 显式接受；Agent 不可代签 |

## 表

| ID | 项 | 处置 | 波次 / 证据 | 签字影响 |
|----|----|------|-------------|----------|
| DEFER-1 | 真实后端验证入口 | **Close**（首批 trait） | W4 · `infra-asa.5`；二期 trait 可残留 scaffold | 阻塞 contracts L3 |
| DEFER-2 | 全 trait 深度语义 | **Close 首批** + **Accept 二期** | W3 · `infra-asa.4`；二期见 [prod-trait-inventory](./prod-trait-inventory.md) §3 | 首批阻塞 L3；二期不阻塞 |
| DEFER-3 | 非 committed DTO | **Close 分批** | W2 · `infra-asa.3`；按 [wire-promotion-candidates](./wire-promotion-candidates.md) | 阻塞 canonical L2 扩面 |
| DEFER-4 | fuzz / oracle / mutants / Miri | **Close** | W1 · `infra-asa.2` | 阻塞 decimalx L1 签字 |
| DEFER-5 | API snapshot / semver 门禁 | **Close** | W5 · `infra-asa.6` | 阻塞 L4/L5 |
| DEFER-6 | 非 Linux 矩阵实测 | **Accept** | W0 本表 + [support-matrix](./support-matrix.md)：仅 Linux 官方支持 | 不阻塞（已声明） |
| DEFER-7 | §8 发布/回滚人工签字 | **Defer-with-sign** | W5 · **仅 Maintainer** | 阻塞 L5；Agent 禁止伪造 |
| DEFER-8 | VenueAdapter additive override 门禁 | **Close** | W3 · `infra-asa.4` | 阻塞 venue 生产路径 |

## 无未分类项

- 上表 8/8 已分类。
- 新增 DEFER 必须追加本表行并走 PR，不得口头接受。

## 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-21 | W0 初冻：8 项全部分类 |
