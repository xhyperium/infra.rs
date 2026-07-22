# natsx SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `natsx` |
| SSOT | `.agents/ssot/adapters/storage/nats/` |
| 实现 | `crates/adapters/storage/nats` |
| 审计日期 | 2026-07-22 |
| 结论 | **P0 生产默认客户端已落地** + live/bench；**未**宣称 package stable |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 生产默认面 | `NatsPool / NatsEventBus / NatsSubscription` |
| contracts | contracts::EventBus（at-most-once） |
| 环境变量 | `FOUNDATIONX_NATS_{URL,USER,PASSWORD} 或 FOUNDATIONX_NATSX_*` |
| live | `tests/live_event_bus.rs`（`#[ignore]`） |
| bench | `benches/hot_path.rs（3s 有界）` |
| DEFER | JetStream 全量 / NKey / TLS 默认开启策略 |

## 对齐矩阵

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| NATSX-1 | workspace member | PASS | `cargo metadata -p natsx` |
| NATSX-2 | 生产默认导出 | PASS | `crates/adapters/storage/nats/src/lib.rs` |
| NATSX-3 | from_env | PASS | config · `FOUNDATIONX_NATS_{URL,USER,PASSWORD} 或 FOUNDATIONX_NATSX_*` |
| NATSX-4 | 离线测试 | PASS | `cargo test -p natsx --all-targets` |
| NATSX-5 | live 入口 | PASS | `tests/live_event_bus.rs` |
| NATSX-6 | bench 有界 | PASS | `benches/hot_path.rs（3s 有界）` |
| NATSX-7 | crate docs | PASS | docs/usage · config · operations |
| NATSX-8 | SSOT 11 层 + landing/draft | PASS | `.agents/ssot/adapters/storage/nats/` |
| NATSX-9 | package stable | OPEN | 禁止宣称 |
| NATSX-10 | DEFER 能力 | OPEN | JetStream 全量 / NKey / TLS 默认开启策略 |

## 验证

```bash
cargo test -p natsx --all-targets
cargo clippy -p natsx --all-targets -- -D warnings
test -f .agents/ssot/adapters/storage/nats/plan/infra-rs-landing.md
test -f .agents/ssot/adapters/storage/nats/plan/infra-rs-draft-spec-goal.md
test -f .agents/ssot/adapters/storage/nats/goal/goal.md
# optional live
# node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
# set -a; source /tmp/foundationx-live.env; set +a
# cargo test -p natsx -- --ignored
```

## 相关

- 总览：[workspace-ssot-alignment.md](./workspace-ssot-alignment.md)
- adapters 汇总：[adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- gap：[draft-gap-matrix.md](./draft-gap-matrix.md)
- SSOT 树：`.agents/ssot/adapters/storage/nats/`
