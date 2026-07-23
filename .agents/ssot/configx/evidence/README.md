# configx — Evidence（横切）

本目录索引 `infra-2d9.9` 的可复验证据，不冒充 CI artifact、固定提交快照、签名或发布批准。

## 记录

- [第 1 轮原子性与失败语义发现](../plan/round-01-findings.md)
- [第 2 轮 reviewer 阻断修复](../plan/round-02-findings.md)
- [第 3 轮候选准备](../plan/round-03-findings.md)
- 行为测试：`crates/configx/src/**`、`crates/configx/tests/**`

最终行覆盖率由 root 串行确认为 `1166 / 1166`（100.0000%），exit 0。Round 2/3 记录了定向 test、clippy、doc、
确定性 phase hook 与覆盖率收敛过程。`f904ecd` 的关闭状态/零时限优先级回归修复在 rebase 后
等价为 `eba66fb`；先前 Codex `review --base main` 已审该实现内容且无 finding。rebased fixed HEAD
已完成完整门禁；最终独立 verifier 因治理措辞阻断，待本次纯文档修正后复核。GitHub 新 HEAD
CI artifact 与发布证据仍 pending。

```bash
cargo fmt -p configx -- --check
cargo test -p configx --all-targets
cargo clippy -p configx --all-targets -- -D warnings
cargo doc -p configx --no-deps
node scripts/quality-gates/cov-gate-100.mjs -p configx --filter crates/configx/src
cmp .agents/ssot/configx/spec/spec.md \
    .agents/ssot/configx/spec/xhyper-configx-complete-spec.md
```
