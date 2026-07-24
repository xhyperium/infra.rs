<!-- ssot:trace=eastmoney.goal.001 -->
# eastmoney — 离线目标

当前 `draft`/`not_started`；目标只描述脱敏市场观测的离线解析，实时能力、产品覆盖、访问方式和外部服务可用性均不承诺。

- 保留标的身份、观测时间、单位、市场标签、修订和缺失原因；
- 对 quote、series、index 等脱敏 fixture 执行纯数据解析与 `domain_macro` 映射；
- 对未知字段、坏数值、重复身份和排序冲突返回稳定错误；
- 以来源合同、授权、许可、人工审查和 commit-matched evidence 作为未来晋级条件。

来源、字段、端点、传输、认证、反爬、限流、缓存和再分发语义均为 `UNKNOWN`，不得在本状态写成接入目标或测试合同。
