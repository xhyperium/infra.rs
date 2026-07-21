# DEFER 处置表（W0 冻结）

| 字段 | 值 |
|------|-----|
| Plan | [PLAN-CORE-PROD-002](../2026-07-21-core-crates-production-readiness.md) §11 |
| 审计 | [core-crates-production-readiness.md](../../report/2026-07-21/core-crates-production-readiness.md) §11.2 |
| Beads | `infra-asa`（epic 已 close） |
| 冻结日期 | 2026-07-21 |
| 实现合入 | 2026-07-21 · PR #120–#128（见计划文） |
| 规则 | 每项必须为 **Close** / **Accept** / **Defer-with-sign** 之一；禁止「未分类」 |

## 处置定义

| 标签 | 含义 |
|------|------|
| **Close** | 本轮必须完成并可勾掉；对应 W 波次验收 |
| **Accept** | 本轮接受残留；写入签字包风险清单，不阻塞目标层级（须 Maintainer 知悉） |
| **Defer-with-sign** | 仅能在 L5 签字包中由 **人工** 显式接受；Agent 不可代签 |

## 表

| ID | 项 | 处置 | 波次 / 证据 | 实现状态 | 签字影响 |
|----|----|------|-------------|----------|----------|
| DEFER-1 | 真实后端验证入口 | **Close**（首批 = 离线 mock）+ **Accept** 真实云端 | W4 · PR [#125](https://github.com/xhyperium/infra.rs/pull/125) | **已合入 mock**；真实 DB/MQ/交易所仍 Accept | 真实后端不阻塞 mock-L3 |
| DEFER-2 | 全 trait 深度语义 | **Close 首批** + **Accept 二期** | W3 · PR [#128](https://github.com/xhyperium/infra.rs/pull/128) | **首批已合入**；二期见 inventory | 二期不阻塞 |
| DEFER-3 | 非 committed DTO | **Close 分批** | W2 · PR [#124](https://github.com/xhyperium/infra.rs/pull/124) | **v1.1–v1.3 已合入** | L2 扩面完成 |
| DEFER-4 | fuzz / oracle / mutants / Miri | **Close** | W1 · PR [#121](https://github.com/xhyperium/infra.rs/pull/121) | **已合入** oracle/边界/门禁 + scheduled mutants/miri | L1 证据链就绪 |
| DEFER-5 | API snapshot / semver 门禁 | **Close** | W5 · PR [#127](https://github.com/xhyperium/infra.rs/pull/127) | **已合入** baseline + CI | L4 门禁就绪 |
| DEFER-6 | 非 Linux 矩阵实测 | **Accept** | W0 + [`support-matrix.md`](../../governance/support-matrix.md) | **已声明仅 Linux** | 不阻塞 |
| DEFER-7 | §8 发布/回滚人工签字 | **Defer-with-sign** | 草稿见 `docs/plans/releases/2026-07-21-signoff-DRAFT.md` | **待 Maintainer** | **阻塞 L5** |
| DEFER-8 | VenueAdapter additive override 门禁 | **Close** | W3 · PR [#128](https://github.com/xhyperium/infra.rs/pull/128) | **已合入** runtime 门禁测 | venue 路径可测 |

## 无未分类项

- 上表 8/8 已分类。
- 新增 DEFER 必须追加本表行并走 PR，不得口头接受。

## 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-21 | W0 初冻：8 项全部分类 |
| 2026-07-21 | W0–W5 合入后更新实现状态；DEFER-7 仍待人签 |
