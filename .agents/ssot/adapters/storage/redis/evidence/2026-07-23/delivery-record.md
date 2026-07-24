# redisx 交付记录

## P0 基线（Standalone）

| 字段 | 值 |
|------|-----|
| PR | https://github.com/xhyperium/infra.rs/pull/281 |
| merge SHA | `bad13fcb7da485513b19b32a1324a8f6e34e2ef9` |
| 合并时间 | 2026-07-23T04:32:13Z |
| package version（当时） | `0.3.6` |
| 可宣称 | Standalone P0 生产默认 KV 客户端 |
| 禁止宣称 | package stable；Cluster/Sentinel/TLS live；Draft 全文 DoD；行覆盖 100% |

## 后续里程碑（同日链）

| 版本 | PR | 要点 |
|------|-----|------|
| 0.3.7 | #285 | 覆盖率 / error_map |
| 0.3.8 | #291 | deadline / pipeline / fencing 锁 |
| 0.3.9 | #298 | metrics / result stream |
| 0.3.10 | #300 | selfcheck §6.5 |
| 0.3.11 | #305 | integration_all_api + data E2E + api_matrix |
| 0.3.12 | #306 | E2E soft-skip；budget/stream/Facade live；selfcheck cancel/JSON；CI pubsub |
| **0.3.15** | 本分支 | xadd_with_id/xread_block；全 API live+e2e+bench |

当前 crate version：**0.3.15**（以 `crates/adapters/storage/redis/Cargo.toml` 为准）。

## 本机复验命令（当前）

| 检查 | 命令 / 期望 |
|------|-------------|
| 离线 lib | `cargo test -p redisx --lib --features pubsub` → pass |
| live 集成 | `export-foundationx-env.sh --env dev -- cargo test -p redisx --features pubsub --test integration_all_api -- --ignored` → pass |
| data E2E | 同上 + `--test e2e_klines_crud`（有 `/home/workspace/data`）→ pass；无数据 soft-skip |
| clippy | `cargo clippy -p redisx --all-targets --features pubsub -- -D warnings` → pass |
| bench | `cargo bench -p redisx --bench api_matrix`（可选） |

## 证据索引

- [gap-matrix-v0.md](./gap-matrix-v0.md)
- [passes-01-05.md](./passes-01-05.md) / [passes-06-10.md](./passes-06-10.md)
- [coverage-residual.md](./coverage-residual.md)
- [ssot-path-decision.md](./ssot-path-decision.md)
- 对齐文档：[docs/ssot/redisx-ssot-alignment.md](../../../../../../docs/ssot/redisx-ssot-alignment.md)
