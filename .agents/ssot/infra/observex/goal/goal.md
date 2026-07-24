# observex — Goal

> 状态：治理修正后候选已重冻；本地 reviewer 完成，verifier 技术/证据初验完成。

## 目标

把 `observex 0.1.2` 收敛为可固定复验的 tracing + 自定义有界进程内 sink 候选：统一治理 `op`，
保持 exporter 容量和生命周期有界，隔离并诊断普通错误与 unwind panic，恢复 poisoned mutex 状态，
并让所有用户可见错误保持简体中文。

| 完成条件 | 当前状态 |
|---|---|
| sanitizer / exporter / diagnostic / poison 实现 | Round 2 已闭合 |
| 行覆盖率 100% | root 串行 `942 / 942`、zeros 0、100.0000%、exit 0 |
| 本地独立 reviewer | 已完成实现/证据审查；纯状态 delta 不改变受审源码/测试 |
| 独立 verifier | 已完成技术/证据初验 |
| active / complete spec 同构 | 本轮 writer 复验 |
| 固定 commit CI、PR、维护者审批与合并 | Pending |

完成条件不包含 OpenTelemetry API/SDK、OTLP、远端持久化、异步 exporter worker 或阻塞隔离。
