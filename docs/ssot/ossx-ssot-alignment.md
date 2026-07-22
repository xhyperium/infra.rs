# ossx SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `ossx` |
| SSOT | `.agents/ssot/adapters/storage/oss/` |
| 实现 | `crates/adapters/storage/oss` |
| 审计日期 | 2026-07-22 |
| 结论 | **P0 生产默认客户端已落地** + live/bench；**未**宣称 package stable |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 生产默认面 | `OssClient / OssConfig + sign_v1` |
| contracts | contracts::ObjectStore |
| 环境变量 | `FOUNDATIONX_OSSX_{ENDPOINT,BUCKET,ACCESS_KEY_ID,ACCESS_KEY_SECRET,REGION}` |
| live | `tests/live_object_store.rs`（`#[ignore]`） |
| bench | `benches/put_get.rs` |
| DEFER | multipart / lifecycle / STS 临时凭证 |

## 对齐矩阵

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| OSSX-1 | workspace member | PASS | `cargo metadata -p ossx` |
| OSSX-2 | 生产默认导出 | PASS | `crates/adapters/storage/oss/src/lib.rs` |
| OSSX-3 | from_env | PASS | config · `FOUNDATIONX_OSSX_{ENDPOINT,BUCKET,ACCESS_KEY_ID,ACCESS_KEY_SECRET,REGION}` |
| OSSX-4 | 离线测试 | PASS | `cargo test -p ossx --all-targets` |
| OSSX-5 | live 入口 | PASS | `tests/live_object_store.rs` |
| OSSX-6 | bench 有界 | PASS | `benches/put_get.rs` |
| OSSX-7 | crate docs | PASS | docs/usage · config · operations |
| OSSX-8 | SSOT 11 层 + landing/draft | PASS | `.agents/ssot/adapters/storage/oss/` |
| OSSX-9 | package stable | OPEN | 禁止宣称 |
| OSSX-10 | DEFER 能力 | OPEN | multipart / lifecycle / STS 临时凭证 |

## 验证

```bash
cargo test -p ossx --all-targets
cargo clippy -p ossx --all-targets -- -D warnings
test -f .agents/ssot/adapters/storage/oss/plan/infra-rs-landing.md
test -f .agents/ssot/adapters/storage/oss/plan/infra-rs-draft-spec-goal.md
test -f .agents/ssot/adapters/storage/oss/goal/goal.md
# optional live
# node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
# set -a; source /tmp/foundationx-live.env; set +a
# cargo test -p ossx -- --ignored
```

## 相关

- 总览：[workspace-ssot-alignment.md](./workspace-ssot-alignment.md)
- adapters 汇总：[adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- gap：[draft-gap-matrix.md](./draft-gap-matrix.md)
- SSOT 树：`.agents/ssot/adapters/storage/oss/`
