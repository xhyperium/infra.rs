# bootstrap — Release

> 状态：`0.3.3` 工作树候选；**BLOCKED / 未发布**。

当前 Cargo 版本为 `0.3.3`。该版本包含 main `ContractStoreSet` 整合、shutdown owner、graceful drain、
ownerless fail-closed、poison 映射与自有错误中文化加固；错误语言补丁后 root 本地串行
最终错误文本修复后覆盖率门禁 exit 0：`963 / 963`，zeros 0，100.0000%；此前 `975 / 975` 与
`961 / 961` 为中间树基线。

治理修正后候选已重冻；本地独立 reviewer 已完成实现/证据审查，独立 verifier 已完成技术/证据初验。
本次纯状态 delta 不改变受审源码/测试。GitHub 固定提交 CI artifact、PR、维护者审批与合并证据仍
pending。当前未创建 tag，
未执行外部发布，也没有签名或校验和；`publish = false` 保持不变。

crate 侧候选记录见 `crates/infra/bootstrap/releases/round-03-2026-07-23.md`。
