# bootstrap — Test

> 状态：候选已重冻；本地独立 reviewer 已完成实现/证据审查，独立 verifier 已完成技术/证据初验。
> 本次纯状态 delta 不改变受审源码/测试；GitHub 固定提交 CI artifact 仍 pending。

行为测试覆盖四条 build 路径、signal-before-drain、ownerless fail-closed、外部预触发、批内 LIFO、
步骤错误后继续、所有权转移、register/drain 快照与真实 mutex poison 错误映射。

| 证据 | 结果 |
|---|---|
| Round 2 `cargo test -p bootstrap --all-targets` | 57 个 Rust 测试通过；详见 round-02 findings |
| Round 2 clippy / doc | 本地退出码 0 |
| Round 3 最终 `cargo test -p bootstrap --all-targets` | 60 passed + 1 ignored；退出码 0 |
| Round 3 root 串行覆盖率 | DependencyUnavailable 顶层脱离 source 文本后 exit 0；`963 / 963`，zeros 0，100.0000% |

此前 `975 / 975` 与 `961 / 961` 分别是 thiserror 修复前、最终错误文本修复前的中间树基线，
均不是当前候选结论。

覆盖率是共享工作树本地证据，不是固定提交 CI artifact。可复验命令与边界见
[`../evidence/README.md`](../evidence/README.md)。
