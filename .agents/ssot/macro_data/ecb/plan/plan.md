<!-- ssot:trace=ecb.plan.001 -->
# ecb 落地计划

| 阶段 | 交付 | 退出条件 |
|---|---|---|
| P0 | 官方 dataflow/DSD 契约表 | URL、参数、媒体类型、日期、哈希齐全 |
| P1 | 脱敏 SDMX fixture 解析 | 维度、缺失值、状态、单位测试通过 |
| P2 | domain_macro 映射 | 来源身份、period、vintage 不丢失 |
| P3 | 联网客户端 | 授权、限流、重试、日志脱敏审查通过 |

当前实现状态由 `.agents/ssot/manifest.json` 管理，未批准前不得新增 crate 路径。
