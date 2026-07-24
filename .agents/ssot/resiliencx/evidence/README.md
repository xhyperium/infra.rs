# resiliencx — Evidence（横切）

本目录记录可复验入口，不冒充 CI artifact、签名发布证据或固定提交快照。

## 当前证据

- Round 1 发现与命令：[`../plan/round-01-findings.md`](../plan/round-01-findings.md)
- Round 2 独立复审修复：[`../plan/round-02-findings.md`](../plan/round-02-findings.md)
- Round 3 候选准备：[`../plan/round-03-findings.md`](../plan/round-03-findings.md)
- 行为测试：`crates/infra/resiliencx/src/**` 的单元测试与 `crates/infra/resiliencx/tests/**`

root 于 2026-07-23 在本轮 Adapter safety 补丁前串行确认行覆盖率 `994 / 994`，100%。本轮补丁后的
首次串行结果为 `1106 / 1116`、缺失 10 行、99.1039%；缺口位于 safe Adapter validation、最终错误与
“拒绝前不调用 operation”探针。执行者补充真实行为测试，未使用 coverage 排除或空断言。root 修复后
串行复验结果为 instrumented `1156`、hit `1156`、zeros `0`、`100.0000%`、退出码 `0`。

随后固定 review 又要求修复 Redis 零 attempts 路由与 legacy async budget core；本轮再次修改
`crates/infra/resiliencx/src`，因此上述 `1156 / 1156` 仅是本次修复前基线。执行者按约束不运行 coverage，
root 最终串行重跑结果为 instrumented `1208`、hit `1208`、zeros `0`、`100.0000%`、退出码 `0`。
三包最终测试为：resiliencx 84 passed；postgresx 52 passed + 6 ignored；redisx 51 passed + 8 ignored。

## 可复验命令

```bash
cargo fmt -p resiliencx -p postgresx -p redisx -- --check
cargo test -p resiliencx -p postgresx -p redisx --all-features --all-targets
cargo clippy -p resiliencx -p postgresx -p redisx --all-features --all-targets -- -D warnings
cargo doc -p resiliencx -p postgresx -p redisx --all-features --no-deps
node scripts/quality-gates/check-workspace-deps.mjs
node scripts/quality-gates/check-crate-versions.mjs
node scripts/quality-gates/cov-gate-100.mjs -p resiliencx --filter crates/infra/resiliencx/src
cmp .agents/ssot/resiliencx/spec/spec.md \
    .agents/ssot/resiliencx/spec/xhyper-resiliencx-complete-spec.md
git diff --check e0dacd95c68a09d464dda97ed1e51e129c26a3cc -- \
    crates/infra/resiliencx crates/adapters/storage/postgres crates/adapters/storage/redis \
    .agents/ssot/resiliencx docs/ssot/resiliencx-ssot-alignment.md
```

本轮没有保存原始命令日志、CI artifact、发布签名或校验和；表内退出码是共享 worktree 的本地执行记录。
治理修正后候选已重冻；本地独立 reviewer 已完成实现/证据审查，独立 verifier 已完成技术/证据初验。
本次纯状态 delta 不改变受审源码/测试。GitHub 固定提交 CI artifact、PR、维护者审批、合并与发布证据
仍 pending。
