# kafkax SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `kafkax` |
| SSOT | `.agents/ssot/adapters/storage/kafka/` |
| 实现 | `crates/adapters/storage/kafka` |
| 审计日期 | 2026-07-22 |
| 结论 | **P0 生产默认客户端已落地** + live/bench；**未**宣称 package stable |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 生产默认面 | `KafkaPool / KafkaProducer / KafkaConsumer / KafkaEventBus` |
| contracts | contracts::EventBus（at-most-once） |
| 环境变量 | `FOUNDATIONX_KAFKAX_{BROKERS,SASL_MECHANISM,SASL_USERNAME,SASL_PASSWORD,TLS}` |
| live | `tests/live_event_bus.rs`（`#[ignore]`） |
| bench | `benches/hot_path.rs（3s 有界）` |
| DEFER | EOS / transactional producer / schema registry / group coordinator 强依赖路径 |

## 对齐矩阵

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| KAFKAX-1 | workspace member | PASS | `cargo metadata -p kafkax` |
| KAFKAX-2 | 生产默认导出 | PASS | `crates/adapters/storage/kafka/src/lib.rs` |
| KAFKAX-3 | from_env | PASS | config · `FOUNDATIONX_KAFKAX_{BROKERS,SASL_MECHANISM,SASL_USERNAME,SASL_PASSWORD,TLS}` |
| KAFKAX-4 | 离线测试 | PASS | `cargo test -p kafkax --all-targets` |
| KAFKAX-5 | live 入口 | PASS | `tests/live_event_bus.rs` |
| KAFKAX-6 | bench 有界 | PASS | `benches/hot_path.rs（3s 有界）` |
| KAFKAX-7 | crate docs | PASS | docs/usage · config · operations |
| KAFKAX-8 | SSOT 11 层 + landing/draft | PASS | `.agents/ssot/adapters/storage/kafka/` |
| KAFKAX-9 | package stable | OPEN | 禁止宣称 |
| KAFKAX-10 | DEFER 能力 | OPEN | EOS / transactional producer / schema registry / group coordinator 强依赖路径 |

## 验证

```bash
cargo test -p kafkax --all-targets
cargo clippy -p kafkax --all-targets -- -D warnings
test -f .agents/ssot/adapters/storage/kafka/plan/infra-rs-landing.md
test -f .agents/ssot/adapters/storage/kafka/plan/infra-rs-draft-spec-goal.md
test -f .agents/ssot/adapters/storage/kafka/goal/goal.md
# optional live
# node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
# set -a; source /tmp/foundationx-live.env; set +a
# cargo test -p kafkax -- --ignored
```

## 相关

- 总览：[workspace-ssot-alignment.md](./workspace-ssot-alignment.md)
- adapters 汇总：[adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- gap：[draft-gap-matrix.md](./draft-gap-matrix.md)
- SSOT 树：`.agents/ssot/adapters/storage/kafka/`
