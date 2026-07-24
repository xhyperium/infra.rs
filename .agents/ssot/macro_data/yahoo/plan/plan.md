<!-- ssot:trace=yahoo.plan.001 -->
# yahoo — 分阶段计划

| 阶段 | 交付物 | 退出条件 |
|---|---|---|
| P0 合同核验 | 来源、许可、访问权限、字段和更新语义 | 官方/合同证据经人工审查 |
| P1 fixture | quote、chart、fx、search 的脱敏样本 | 严格解析和坏输入测试通过 |
| P2 统一模型 | 时间、单位、缺失和 source identity 映射 | 与 `domain_macro` 草案一致 |
| P3 provider 实现 | 获批独立 crate 与离线/授权测试 | manifest、Cargo、门禁和证据一致 |

在 P0 完成前不创建具体 crate 路径，不运行真实网络请求；拒绝或配额响应是终止条件。
