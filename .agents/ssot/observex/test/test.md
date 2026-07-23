# observex — Test

> 状态：治理修正后候选已重冻；本地独立 reviewer 已完成实现/证据审查，独立 verifier 已完成
> 技术/证据初验。本次纯状态 delta 不改变受审源码/测试；GitHub CI artifact pending。

测试覆盖控制字符/空白与 UTF-8 字节上限、容量 0/恰满/超限、批次原子拒绝、并发生命周期守恒、
计数饱和、exporter `Err` / unwind panic 诊断、flush/shutdown 错误映射、幂等关闭与真实 poison 恢复。

| 证据 | 结果 |
|---|---|
| Round 2 `cargo test -p observex --all-targets` | 34 个 Rust 测试通过；详见 round-02 findings |
| Round 2 clippy / fmt / doc | 本地退出码 0 |
| Round 3 root 串行覆盖率 | `942 / 942`、zeros 0、100.0000%、exit 0 |

覆盖率是共享工作树本地证据，不是固定提交 CI artifact。可复验命令见
[`../evidence/README.md`](../evidence/README.md)。
