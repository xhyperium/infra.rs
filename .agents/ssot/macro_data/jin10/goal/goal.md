<!-- ssot:trace=jin10.goal.001 -->
# jin10 — 离线目标

当前 `draft`/`not_started`；以下目标只描述待授权方向，不是 REST、WebSocket、认证、缓存、限流或性能合同。当前只交付脱敏消息 fixture 解析和安全边界。

- 保留事件来源身份、时间、语言、单位、修订和缺失原因；
- 对未知消息、坏数值、重复身份和时间冲突返回稳定错误；
- 以离线 fixture、人工审查、回滚目标和 commit-matched evidence 作为唯一晋级依据。
