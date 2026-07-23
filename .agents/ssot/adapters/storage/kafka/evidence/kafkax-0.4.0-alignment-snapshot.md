# kafkax 0.4.0 对齐报告快照

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-23 |
| 版本 | 0.4.0 |
| PR | [#354](https://github.com/xhyperium/infra.rs/pull/354) |
| 提交 | `e5a0aee0` |
| 状态 | **已合并** |

## 可交付面闭合

0.3.x 九轮 PATCH 完结，0.4.0 为 MINOR 里程碑。

| 版本 | 能力 | 证据 |
|------|------|------|
| 0.3.1 | ALO/PTC、offset store | cargo test |
| 0.3.2 | Broker conformance、TLS/CA、SASL PLAIN | broker_conformance |
| 0.3.3 | 三轮加固 fail-closed | lib test |
| 0.3.4 | ConfigBuilder、timestamp、十轮矩阵 | lib + behavior |
| 0.3.5 | 生产测试矩阵、prod_reliability | kafka-prod-matrix |
| 0.3.6 | selfcheck §6.2 | live_selfcheck |
| 0.3.7 | gap 清零（headers/key/stats） | e2e_api_roundtrip |
| 0.3.8 | skeptic 行为测试 | api_surface_offline |
| 0.3.9 | G-STATS-01 严格计数 | prod_offline |
| 0.4.0 | 里程碑闭合 | 全量 CI |

## NO-GO / OOS

| ID | 条款 | 状态 |
|----|------|------|
| GROUP | consumer group | NO-GO |
| REB | rebalance | NO-GO |
| RECON | 自动重连 | NO-GO |
| EOS-N | native EOS | NO-GO |
| SCHEMA | schema registry | NO-GO |
| SCRAM | SCRAM/OAuth/mTLS | NO-GO |
| HA | multi-broker | NO-GO |
| STABLE | package stable | NO-GO |
| P2-* | Part2 量化栈 | OOS |

## 全量版本对齐矩阵

| 层面 | 文件 | 版本 | 状态 |
|------|------|------|------|
| 域-goal | `goal/goal.md` | 0.4.0 | ✓ |
| 域-spec | `spec/spec.md` | 0.4.0 | ✓ |
| 域-spec | `xhyper-kafkax-complete-spec.md` | 0.4.0 | ✓ |
| 域-matrix | `matrix/matrix.md` | S-21 | ✓ |
| 域-tasks | `tasks/tasks.md` | 0.4.0 节 | ✓ |
| 域-evidence | `kafkax-10pass-matrix.md` | 0.4.0 | ✓ |
| 域-release | `release/release.md` | 0.4.0 | ✓ |
| 域-review | `review/review.md` | 0.4.0 | ✓ |
| 对齐 | `adapters-ssot-alignment.md` | 0.4.0 | ✓ |
| 对齐 | `kafkax-ssot-alignment.md` | 0.4.0 | ✓ |
| 对齐 | `gap-matrix.md` | 0.4.0 | ✓ |
| 对齐 | `workspace-ssot-alignment.md` | 0.4.0 | ✓ |
| 源码 | `Cargo.toml` | 0.4.0 | ✓ |
| 源码 | `CHANGELOG.md` | 0.4.0 | ✓ |
| 源码 | `releases/0.4.0.md` | 0.4.0 | ✓ |
| 源码 | `docs/标准.md` | v0.4.0 | ✓ |
| 治理 | `06-governance.md` | §6.0.7 | ✓ |

## CI 验证

全部 6 个必需检查 PASS（#354 合并前）。
