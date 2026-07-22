# contracts maintenance evidence（2026-07-23）

Baseline：`3cd29a942710c0fb42f3f6bc05e3c31570acad47`；未修改 contract-testkit 源码。

## 三轮审计索引

| 轮次 | 证据 | 结论 |
|---|---|---|
| Round 1 事实 | base `3cd29a9`；trait/profile/handles/helpers/消费者与历史核对 | 发现 profile 假阳性、helper E2E/原子性误导、active spec/API/版本追溯漂移 |
| Round 2 规格设计 | `goal/design/spec/matrix/test/gate` 与 active 双镜像；`cmp` exit 0 | 保持 trait Additive Only；profile 定位为接线意图，handles 只证明引用形状 |
| Round 3 实现验证 | 下列 Red→Green、set/commit/rollback/兼容别名矩阵与最终命令/exit | fail-closed 声明面完成；全 trait 生产语义与真实业务 live 保持 NO-GO |

## Red → Green

`cargo test -p contracts --test live_contracts` 首次 exit 101：准确命名 helper 尚不存在。最小实现并强化 `LiveHandles::validate` 后同命令 exit 0，覆盖 repo/account/venue_time fail-closed、publish failure 与 Tx begin failure。

## Final validation

| 命令 | exit | 结果 |
|---|---:|---|
| `cargo test -p contracts --all-targets` | 0 | PASS（43 tests） |
| `cargo test -p contract-testkit --all-targets` | 0 | PASS（15 tests；源码未改） |
| `cargo clippy -p transportx -p contracts --all-targets --all-features -- -D warnings` | 0 | PASS |
| `cargo doc -p transportx -p contracts --all-features --no-deps` | 0 | PASS |
| `cargo test`：bootstrap/observex/resiliencx + 9 adapters 全部 all-targets | 0 | PASS；依赖外部服务的 live tests 均诚实 ignored |
| `cov-gate-100.mjs -p contracts --filter crates/contracts/src` | 0 | PASS，547/547，100% |
| `check-crate-versions.mjs` / `check-workspace-deps.mjs` | 0 / 0 | PASS |
| `check-public-api.mjs -p contracts --update` | 0 | 机械更新 additive baseline，263 行 |
| `check-public-api.mjs -p contracts --require-tool` | 0 | PASS，当前公开面与 baseline 一致 |
| `node scripts/quality-gates/check.mjs` | 0 | PASS，Harness 44/44；`STATUS.md` 已由生成器刷新 |
| workspace fmt/clippy/test/doc + `cargo deny check` | 0 | PASS；`cargo deny` 仅报告既有 skip 配置 warning |
| 双镜像 `cmp` | 0 | PASS |

## Residual OPEN / NO-GO

- 全 trait conformance、交易业务 live、跨 backend 原子性、EventBus E2E delivery、Production Ready/L5 均 NO-GO。
- 独立 Standards 与 Spec reviewer 均已 PASS；maintainer 审批与 GitHub CI 仍为发布门禁。
