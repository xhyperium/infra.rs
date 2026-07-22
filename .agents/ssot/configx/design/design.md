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

第三轮不扩展能力；代码 owner 已完成并发测试确定性加强。治理修正后候选已重冻，本地独立 reviewer
已完成实现/证据审查，独立 verifier 已完成技术/证据初验；本次纯状态 delta 不改变受审源码/测试。
GitHub 固定提交 CI artifact、PR、维护者审批、合并、tag/发布仍 pending。
