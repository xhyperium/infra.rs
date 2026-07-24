# kafkax 0.4.0 对齐报告快照

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-23 |
| 版本 | 0.4.0 |
| PR | [#354](https://github.com/xhyperium/infra.rs/pull/354) |
| 提交 | `e5a0aee0` |
| 状态 | **已合并** |

## 可交付面闭合

kafkax 0.3.x 九轮 PATCH 迭代完结，0.4.0 为 MINOR 里程碑版本。

| 版本 | 能力 | 证据 |
|------|------|------|
| 0.3.1 | 应用层 ALO/PTC、offset store | cargo test |
| 0.3.2 | Broker conformance、TLS/CA、SASL PLAIN | broker_conformance |
| 0.3.3 | 三轮加固 fail-closed | lib test |
| 0.3.4 | ConfigBuilder、timestamp、十轮矩阵 | lib + behavior |
| 0.3.5 | 生产测试矩阵、prod_reliability、benchmarks | kafka-prod-matrix |
| 0.3.6 | selfcheck §6.2（9 catalog items） | live_selfcheck |
| 0.3.7 | gap 清零（headers、key、stats） | e2e_api_roundtrip |
| 0.3.8 | skeptic ALO/with_group/with_config | api_surface_offline |
| 0.3.9 | G-STATS-01 严格计数 | prod_offline |
| 0.4.0 | 里程碑闭合 | 全量 CI |

## NO-GO / OOS 边界（不变）

| ID | 条款 | 状态 |
|----|------|------|
| GROUP | consumer group coordinator | NO-GO |
| REB | rebalance / generation fencing | NO-GO |
| RECON | 自动重连 | NO-GO |
| EOS-N | native transactional EOS | NO-GO |
| SCHEMA | schema registry | NO-GO |
| SCRAM | SCRAM / OAuth / mTLS | NO-GO |
| HA | multi-broker / HA | NO-GO |
| STABLE | package stable / crates.io | NO-GO |
| P2-* | Part2 量化栈 | OOS |

## 全量版本对齐矩阵

| 层面 | 文件 | 版本 | 状态 |
|------|------|------|------|
| 域-goal | `goal/goal.md` | 0.4.0 | 已同步 |
| 域-spec | `spec/spec.md` | 0.4.0 | 已同步 |
| 域-spec | `xhyper-kafkax-complete-spec.md` | 0.4.0 | dual spec |
| 域-matrix | `matrix/matrix.md` | S-21 (0.4.0) | 已同步 |
| 域-tasks | `tasks/tasks.md` | 0.4.0 节 | 已同步 |
| 域-evidence | `kafkax-10pass-matrix.md` | 0.4.0 增量 | 已同步 |
| 对齐 | `docs/ssot/adapters-ssot-alignment.md` | 0.4.0 | L56,L63 |
| 对齐 | `docs/ssot/kafkax-ssot-alignment.md` | 0.4.0 | L9 |
| 对齐 | `docs/ssot/gap-matrix.md` | 0.4.0 | L10 |
| 对齐 | `docs/ssot/workspace-ssot-alignment.md` | 0.4.0 | L29 |
| 源码 | `Cargo.toml` | 0.4.0 | 已更新 |
| 源码 | `CHANGELOG.md` | 0.4.0 | 已更新 |
| 源码 | `releases/0.4.0.md` | 0.4.0 | 新建 |
| 源码 | `docs/标准.md` | v0.4.0 | 已更新 |
| 治理 | `docs/constitution/06-governance.md` | §6.0.7 | 已同步 |

## CI 验证（#354 合并前）

全部 6 个必需检查 PASS：rust-fmt / rust-clippy / rust-test / Constitution / Template / 安全基线

## 文件清单（16 个文件，+192/-26）

```text
.agents/ssot/adapters/storage/kafka/evidence/kafkax-10pass-matrix.md
.agents/ssot/adapters/storage/kafka/goal/goal.md
.agents/ssot/adapters/storage/kafka/matrix/matrix.md
.agents/ssot/adapters/storage/kafka/spec/spec.md
.agents/ssot/adapters/storage/kafka/spec/xhyper-kafkax-complete-spec.md
.agents/ssot/adapters/storage/kafka/tasks/tasks.md
crates/adapters/storage/kafka/CHANGELOG.md
crates/adapters/storage/kafka/Cargo.toml
crates/adapters/storage/kafka/docs/标准.md
crates/adapters/storage/kafka/releases/0.4.0.md
crates/adapters/storage/kafka/src/lib.rs
docs/constitution/06-governance.md
docs/ssot/adapters-ssot-alignment.md
docs/ssot/gap-matrix.md
docs/ssot/kafkax-ssot-alignment.md
docs/ssot/workspace-ssot-alignment.md
```
