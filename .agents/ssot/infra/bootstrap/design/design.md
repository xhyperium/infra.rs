# bootstrap — Design

> 当前设计对应 `bootstrap 0.3.3`；描述 main 已有 `ContractStoreSet` 整合、进程内 typed composition
> 与同步 drain。

## Main contracts 整合

- `ContractStoreSet` 提供固定的 `Arc<dyn KeyValueStore>` / `Arc<dyn EventBus>` 可选槽位，并由
  `Bootstrap`、`PlatformContext` 与 `AppContext` 暴露只读接线面。
- Redis/NATS 只作为固定摘要组合实验的 dev-dependencies；具体 adapter 不进入 bootstrap 生产依赖。
- 该整合是 additive typed composition，不提供动态 `register` / `resolve`、泛型 Repository 注册、
  跨资源事务或补偿编排。

## 所有权与关停路径

- `Bootstrap` 组装 typed `PlatformContext` / `AppContext`，成功 build 产物保有唯一 shutdown owner。
- `graceful_shutdown(self)` 先触发 signal，再执行本次 drain 快照；ownerless 且 signal 未预触发时
  返回 `MissingDependency("shutdown_guard")`，不执行 hook。
- `into_parts` 显式把 owner 转交 `ShutdownController`；`run_drain` 是调用方主动选择的 drain-only
  逃生面，不隐式满足 signal 前置条件。
- drain 注册与取快照由同一 mutex 线性化，hook 在锁外按批内 LIFO 同步执行；锁中毒保留下层原因。

## 设计边界

hook 可阻塞或 panic；本 crate 不提供 timeout、async drain、取消或 panic 隔离。不同批次可并发，
不保证跨批全局 LIFO。所有依赖通过类型化上下文暴露，禁止字符串 / `Any` / `TypeId` Service Locator。

第三轮候选已吸收 main 的 `ContractStoreSet` 并完成治理修正后的重冻。本地独立 reviewer 已完成
实现/证据审查，独立 verifier 已完成技术/证据初验；本次纯状态 delta 不改变受审源码/测试。
GitHub 固定提交 CI artifact、PR、维护者审批、合并、tag/发布仍 pending。
