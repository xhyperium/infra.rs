# configx — Design

> 当前设计对应 `configx 0.1.2`；只提供调用方驱动的进程内配置能力。

## 数据与提交边界

- 批量输入先在锁外完整准备，再以单写锁提交；reload 完成所有 source load 与 key 校验后替换整张 map。
- `ConfigWatch` 以 mutation mutex 排序 `notify / reload / close`；reload 等待 store 时不持有 state mutex。
- store 替换是配置线性化点，generation 在 mutation mutex 释放前发布；两者不是一条联合原子读 API。
- timed wait 以总 deadline 和 state `try_lock` 限制锁竞争与伪通知造成的等待扩张。

## 兼容与安全边界

兼容 Option / 折叠 API 保留；生产新路径使用 Result 快照、`try_*` secret/subset 与显式 wait outcome。
`secret:` 只控制诊断脱敏，不提供加密、权限控制或 secret 托管。File / Env 只在显式调用时加载，
没有自动 watcher 或后台生命周期。

第三轮不扩展能力；代码 owner 已完成并发测试确定性加强。`f904ecd` 的关闭状态/零时限优先级回归
修复在 rebase 后等价为 `eba66fb`；先前 Codex `review --base main` 已审该实现内容且无 finding。
rebased fixed HEAD 已完成完整门禁；最终独立 verifier 因治理措辞阻断，待本次纯文档修正后复核。
GitHub 新 HEAD CI artifact、PR、维护者审批、合并、tag/发布仍 pending。
