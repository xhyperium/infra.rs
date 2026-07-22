# observex — Release

> 状态：`0.1.2` 工作树候选；**BLOCKED / 未发布**。

root 已完成从 `0.1.1` 到 `0.1.2` 的 PATCH bump。该版本候选包含 sanitizer、有界 exporter、
错误/unwind 诊断、简体中文错误与 poison 恢复。本轮新树的 root 串行覆盖率为
`942 / 942`、zeros 0、100.0000%、exit 0。

治理修正后候选已重冻；本地独立 reviewer 已完成实现/证据审查，独立 verifier 已完成技术/证据初验。
本次纯状态 delta 不改变受审源码/测试。发布前仍须完成 GitHub 固定提交 CI artifact、PR、维护者审批与合并。
当前未创建 tag，未执行外部发布，也没有签名或校验和；`publish = false` 保持不变。

crate 侧候选记录见 `crates/observex/releases/round-03-2026-07-23.md`。
