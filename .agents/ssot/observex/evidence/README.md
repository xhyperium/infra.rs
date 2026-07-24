# observex — Evidence（横切）

本目录索引 `infra-2d9.9` 的可复验证据，不冒充 CI artifact、固定提交快照、签名或发布批准。

## 记录

- [第 1 轮有界 sink 发现与加固](../plan/round-01-findings.md)
- [第 2 轮 sanitizer / diagnostic / poison 闭环](../plan/round-02-findings.md)
- [第 3 轮候选准备](../plan/round-03-findings.md)
- 行为测试：`crates/infra/observex/src/**`、`crates/infra/observex/tests/**`

root 于 2026-07-23 对本轮新树串行确认行覆盖率 `942 / 942`、zeros 0、100.0000%、exit 0。
治理修正后候选已重冻；本地独立 reviewer 已完成实现/证据审查，独立 verifier 已完成技术/证据初验。
本次纯状态 delta 不改变受审源码/测试。GitHub 固定提交 CI artifact 与发布证据仍 pending。

```bash
cargo fmt -p observex -- --check
cargo test -p observex --all-targets
cargo clippy -p observex --all-targets -- -D warnings
cargo doc -p observex --no-deps
node scripts/quality-gates/cov-gate-100.mjs -p observex --filter crates/infra/observex/src
cmp .agents/ssot/observex/spec/spec.md \
    .agents/ssot/observex/spec/xhyper-observex-complete-spec.md
```
