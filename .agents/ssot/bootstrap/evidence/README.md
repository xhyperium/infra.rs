# bootstrap — Evidence（横切）

本目录索引 `infra-2d9.9` 的可复验证据，不冒充 CI artifact、固定提交快照、签名或发布批准。

## 记录

- [第 1 轮发现与加固](../plan/round-01-findings.md)
- [第 2 轮 ownerless / poison 复审闭环](../plan/round-02-findings.md)
- [第 3 轮候选准备](../plan/round-03-findings.md)
- 行为测试：`crates/infra/bootstrap/src/**`、`crates/infra/bootstrap/tests/**`

## 已确认机器证据

root 于 2026-07-23 在 DependencyUnavailable 顶层脱离任意 source 文本后串行确认覆盖率门禁 exit 0：
`963 / 963`，zeros 0，100.0000%；最终测试为 46 + 10 + 4 = 60 passed、1 ignored。此前
`975 / 975` 与 `961 / 961` 分别是 thiserror 修复前、最终错误文本修复前的中间树基线。
Round 2 记录还包含定向 test、clippy、doc 与首次 coverage 失败的真实过程。治理修正后候选已重冻；
本地独立 reviewer 已完成实现/证据审查，独立 verifier 已完成技术/证据初验。本次纯状态 delta
不改变受审源码/测试。GitHub 固定提交 CI artifact、签名与发布证据仍 pending。

## 可复验命令

```bash
cargo fmt -p bootstrap -- --check
cargo test -p bootstrap --all-targets
cargo clippy -p bootstrap --all-targets -- -D warnings
cargo doc -p bootstrap --no-deps
node scripts/quality-gates/cov-gate-100.mjs -p bootstrap --filter crates/infra/bootstrap/src
cmp .agents/ssot/bootstrap/spec/spec.md \
    .agents/ssot/bootstrap/spec/xhyper-bootstrap-complete-spec.md
```
