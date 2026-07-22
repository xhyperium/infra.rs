# ossx SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `ossx` |
| SSOT | `.agents/ssot/adapters/storage/oss/` |
| 实现 | `crates/adapters/storage/oss` |
| 审计日期 | 2026-07-23 |
| version | `0.3.2` |
| 结论 | **ObjectStore + 有界 multipart/retry/资源治理已落地**；**未**宣称 package stable |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 生产默认面 | `OssClient / OssConfig + sign_v1` |
| multipart | initiate / upload_part / complete / abort / put_object_multipart |
| retry | `with_retry` / `with_retry_default`（resiliencx） |
| transport | 远程仅 HTTPS；HTTP 只允许 loopback |
| resources | object/buffer/error body/in-flight/part/count/retry 均有硬上界 |
| cancel/orphan | close → Cancelled；drop → 有界 registry；显式 abort 后移除；失败显式 orphan risk |
| multipart deadline | initiate/parts/complete 共享单一剩余 operation deadline |
| contracts | `ObjectStore` |
| 环境变量 | `FOUNDATIONX_OSSX_{ENDPOINT,BUCKET,ACCESS_KEY_ID,ACCESS_KEY_SECRET,REGION}` |
| live | `tests/live_object_store.rs`（`#[ignore]`） |
| 原 OBJECTIVE DEFER | **PASS**（multipart / retry） |
| 仍 OPEN（非 OBJECTIVE） | lifecycle / STS 临时凭证 / 流式 TB 对象 / package stable |

## 对齐矩阵

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| OSSX-1–8 | member/export/env/test/live/bench/docs/SSOT | PASS | — |
| OSSX-9 | package stable | OPEN | 禁止宣称 |
| OSSX-10 | multipart | PASS | `client` + `sign` subresources |
| OSSX-11 | resiliencx retry | PASS | `src/retry.rs` |
| OSSX-12 | HTTPS fail-closed | PASS | config 单测 |
| OSSX-13 | 资源与并发硬上界 | PASS | config/client 单测 |
| OSSX-14 | XML/取消/orphan/deadline | PASS | loopback HTTP 取消补偿 + 多片总 deadline 单测 |
| OSSX-15 | STS/lifecycle/streaming/package stable | OPEN | 禁止过度声明 |

## 验证

```bash
cargo test -p ossx --all-targets
cargo clippy -p ossx --all-targets -- -D warnings
cmp .agents/ssot/adapters/storage/oss/spec/spec.md \
    .agents/ssot/adapters/storage/oss/spec/xhyper-ossx-complete-spec.md
```

## 相关

- [adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- [gap-matrix.md](./gap-matrix.md)
