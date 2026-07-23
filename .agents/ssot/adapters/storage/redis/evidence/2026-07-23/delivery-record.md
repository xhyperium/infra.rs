# redisx 交付记录（Standalone P0）

| 字段 | 值 |
|------|-----|
| PR | https://github.com/xhyperium/infra.rs/pull/281 |
| merge SHA | `bad13fcb7da485513b19b32a1324a8f6e34e2ef9` |
| 合并时间 | 2026-07-23T04:32:13Z |
| package version | `0.3.6`（后续覆盖率补强见同目录 / 后续 patch） |
| 可宣称 | Standalone P0 生产默认 KV 客户端 |
| 禁止宣称 | package stable；Cluster/Sentinel/TLS live；Draft 全文 DoD；行覆盖 100% |

## 本机复验（main 后）

| 检查 | 结果 |
|------|------|
| `cargo test -p redisx --all-targets` | pass |
| `cargo test -p redisx --all-targets --features pubsub` | pass |
| live_kv / conformance / pubsub `--ignored` | pass（真实 Redis） |
| `cargo clippy -p redisx --all-targets --features pubsub -- -D warnings` | pass |
| bench `kv_hot_path` | pass（吞吐数字见 scratch 日志） |

## 证据索引

- [gap-matrix-v0.md](./gap-matrix-v0.md)
- [passes-01-05.md](./passes-01-05.md) / [passes-06-10.md](./passes-06-10.md)
- [coverage-residual.md](./coverage-residual.md)
- [ssot-path-decision.md](./ssot-path-decision.md)
