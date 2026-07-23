# configx — Release

> 状态：`0.1.2` 工作树候选；**BLOCKED / 未发布**。

root 已完成从 `0.1.1` 到 `0.1.2` 的 PATCH bump。该版本候选包含原子 reload、secret 诊断脱敏、
有界等待与显式失败语义；root 串行覆盖率为 `1166 / 1166`（100.0000%）。

并发测试确定性加强已完成。`f904ecd` 的关闭状态/零时限优先级回归修复在 rebase 后等价为
`eba66fb`；先前 Codex `review --base main` 已审该实现内容且无 finding。rebased fixed HEAD 已完成
完整门禁；最终独立 verifier 因治理措辞阻断，待本次纯文档修正后复核。GitHub 新 HEAD
CI artifact、PR、维护者审批与合并仍 pending；新 HEAD 须重跑 CI 并重新取得审批。
当前未创建 tag，未执行外部发布，也没有签名或校验和；`publish = false` 保持不变。

crate 侧候选记录见 `crates/configx/releases/round-03-2026-07-23.md`。
