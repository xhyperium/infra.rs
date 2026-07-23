# resiliencx — Gate

> 当前发布门禁：**BLOCKED**。当前 Round 3 三包 scoped test / clippy / doc / fmt 与 spec / dependency
> gates 已通过；版本已按 R-C2 裁定；root 串行 coverage `1208 / 1208`、zeros 0、100.0000%、
> 退出码 0。候选已重冻；本地 reviewer 完成，verifier 技术/证据初验完成；本次纯状态 delta
> 不改变受审源码/测试。GitHub 固定提交 CI artifact、PR/审批/合并仍 pending。

必需门禁：fmt、all-features/all-targets test、clippy `-D warnings`、rustdoc、spec `cmp`、scoped diff check。
命令与边界见 [`../evidence/README.md`](../evidence/README.md)。任一 pending 项未闭合时不得发布。
