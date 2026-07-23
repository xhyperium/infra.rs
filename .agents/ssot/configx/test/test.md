# configx — Test

> 状态：rebase 后 fixed HEAD 完整门禁已通过；`f904ecd` 的关闭状态/零时限优先级回归修复
> 在 rebase 后等价为 `eba66fb`。最终独立 verifier 待本次纯文档修正后复核；GitHub 新 HEAD CI artifact pending。

测试覆盖批量中点不可见、reload 只见完整旧/新快照、加载/校验失败保留旧值、poison 显式失败、
secret Debug 与 parse 错误脱敏、generation 溢出、reload/state 锁边界、真实伪通知与显式 wait outcome。

| 证据 | 结果 |
|---|---|
| Round 2 `cargo test -p configx --all-targets` | 49 个 Rust 测试通过；详见 round-02 findings |
| Round 2 clippy / doc | 本地退出码 0 |
| Round 3 `cargo test -p configx --all-targets` | 50 个 Rust 测试通过；bench/examples 通过 |
| Round 3 phase-hook 竞态测试 | 连续 100 轮通过 |
| Round 3 root 串行覆盖率 | `1166 / 1166`（100.0000%），exit 0 |

并发测试已改用 per-watch phase hook + Barrier，并连续运行 100 轮；覆盖率与本地审查均已闭合。
可复验命令见
[`../evidence/README.md`](../evidence/README.md)。
