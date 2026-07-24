<!-- ssot:trace=domain_macro.plan.001 -->
# domain_macro 落地计划

| 阶段 | 交付 | 当前状态 | 退出证据 |
|---|---|---|---|
| P0 | Goal/ADR/术语批准 | 未开始 | reviewer、日期、版本 |
| P1 | 值对象、Period、Unit、Identity | 未开始 | 构造/反序列化/属性测试 |
| P2 | RevisionChain、MacroState、Diff | 未开始 | 原子失败、重复、零值和 as-of 测试 |
| P3 | JSON envelope 与 N-1 迁移 | 未开始 | golden fixture、迁移和回滚报告 |
| P4 | provider 映射 | 未开始 | 每个来源契约、fixture、许可证和映射 evidence |

当前唯一实现路径是 `crates/macrox`；不存在的 `src/*.rs` 不得写入 evidence 或宣称已落地。
