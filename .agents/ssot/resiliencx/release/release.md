# resiliencx — Release

> 状态：`0.1.2` 工作树候选；**BLOCKED / 未发布**。

root 已在同一 PR 内完成从 `0.1.1` 到 `0.1.2` 的 PATCH bump；按 R-C2 不重复 bump。当前候选包含
`RetrySafety`、安全 generic Adapter budget、整次 deadline、seeded jitter、budget reservation / refund
与 bulkhead poison 恢复。此前覆盖率 `994 / 994` 是本轮安全补丁前基线；首次新树结果为
`1106 / 1116`、99.1039%。缺失行为测试补齐后，root 串行复验为 `1156 / 1156`、zeros 0、
100.0000%、退出码 0。

固定 review 后又新增 unchecked generic async budget core 并修复 Redis 零 attempts 路由；因此上述
`1156 / 1156` 是本次源码修复前基线。root 最终串行重跑为 `1208 / 1208`、zeros 0、100.0000%、
退出码 0。该证据已纳入本地独立 reviewer 审查与 verifier 技术/证据初验，但不替代 GitHub
固定提交 CI artifact 与发布审批。

治理修正后候选已重冻；本地独立 reviewer 已完成实现/证据审查，独立 verifier 已完成技术/证据初验。
本次纯状态 delta 不改变受审源码/测试。发布前仍须取得 GitHub 固定提交 CI artifact、PR、维护者审批
与合并证据。当前未创建 tag，
未执行外部发布，也没有签名或校验和；`publish = false` 保持不变。

crate 侧候选记录见 `crates/infra/resiliencx/releases/round-03-2026-07-23.md`。
