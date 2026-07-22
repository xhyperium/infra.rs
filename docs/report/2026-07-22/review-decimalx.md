# Review: decimalx v0.1.1 — 2026-07-22

| 字段 | 值 |
| --- | --- |
| 目标 crate | `decimalx` |
| 路径/层级 | `crates/types/decimal` / Types |
| SSOT | `.agents/ssot/types/decimal/` |
| 对齐文档 | `docs/ssot/types-ssot-alignment.md`、`crates/types/decimal/docs/WIRE.md` |
| 审查者 | AI Agent |

## 1. 概览

decimalx 的 checked 构造、scale、舍入、Money/Price/Qty newtype、serde 校验和 wire v1 证据较强。默认关闭 `panicking-ops`，专项门禁扫描 102 个文件 0 hits；内部资金路径可有条件 GO，但调用方仍需遵守 checked-only 纪律。

## 2. 通用维度评估

| 维度 | 评分 | 证据 |
| --- | ---: | --- |
| D1 公开 API | 5 | public surface、doc、checked constructors |
| D2 类型与不变量 | 5 | MAX_SCALE、currency、newtype 和 overflow 校验 |
| D3 错误处理 | 5 | DecimalError 映射 kernel ErrorKind |
| D4 并发安全 | 4 | 值类型无共享可变状态，D4 主要 N/A |
| D5 Trait | 5 | 算术/serde 边界按类型分层 |
| D6 依赖与版本 | 5 | workspace dependency gate 通过 |
| D7 SSOT 对齐 | 5 | WIRE_SCHEMA_VERSION=1、feature 纪律和 WIRE 文档一致 |
| D8 测试覆盖 | 5 | LCOV 878/878、property/oracle/adversarial/golden 通过 |
| D9 可观测性 | 1 | 纯类型 crate 不适用 tracing |

## 3. 专项与发现

- `checked_add/sub/mul/div/rescale` 覆盖 overflow、除零、非法 scale 和 rounding；serde 拒绝未知字段/非法 scale。
- `panicking-ops` 可选 feature 仍存在，必须保持默认关闭并继续运行 no-panicking gate。
- P2：将“内部库有条件 GO”与“所有上游输入都被校验”分开；adapter metadata 不应先转 f64 再回到 Decimal。

## 4. SSOT 对齐

| 条目 | 状态 | 结论 |
| --- | --- | --- |
| checked arithmetic | fully | PASS |
| wire v1 / serde shape | fully | PASS |
| panicking operator feature | fully | PASS（默认关闭） |
| N-1 / package stable | partial | OPEN；当前只证明 committed wire subset |

## 5. 质量门禁与判定

build/test/fmt/clippy/doc、decimal no-panicking、LCOV 均通过；L1 有条件 GO，S=33/35，QT-3 为 Conditional/接线后可用，其余由上层决定。

> 本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计。

## 6. 生产就绪判定

本 crate 的层级、S1–S7 与 QT 判定以本报告上文和 workspace 综合报告为准；不能外推为 L5。

## 7. 综合建议

按本报告 P0/P1/P2 顺序补齐能力边界，并在对应真实后端或交易所环境中留下可复现实证。

## 8. 变更记录

2026-07-22：按 `review-prompt.md` v1.0 补充逐 package 审查报告。

## 9. 限制声明

本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计；历史、mock、fixture 和 ignored live 入口不等同于 live PASS。
