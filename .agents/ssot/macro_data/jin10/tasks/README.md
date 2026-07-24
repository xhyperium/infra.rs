<!-- ssot:trace=jin10.task.001 -->
# jin10 — 离线任务

当前 `not_started`，所有任务都必须在脱敏 fixture 和本地 Mock 范围内完成；未获批路径不是实现证据。

| 任务 | 范围 | 退出条件 |
|---|---|---|
| JIN10-T01 | 事件、日历、行情的离线结构与身份校验 | 合法/缺失/未知/重复 fixture 可重放 |
| JIN10-T02 | 时间、语言、单位和缺失原因映射 | 不丢失来源身份，不静默降级 |
| JIN10-T03 | 坏 JSON、坏数值、冲突身份和未知权限错误 | 错误稳定且无原始输入泄露 |
| JIN10-T04 | secret sentinel 脱敏回归 | Debug、Display、Serialize、URL、错误、tracing 和原始响应均不可见 |
| JIN10-T05 | Rust/Node/SSOT 质量门禁 | 命令、退出码、测试 ID 和 fixture 摘要写入证据 |

## 未来任务门

供应商合同和授权核验完成前，不创建认证、HTTP、WebSocket、缓存、限流、重试或真实 E2E 任务。未来若获批，必须新建独立任务并同步 manifest 的 `authorized_standard_client`、`approved`、Cargo member 和回滚证据。
