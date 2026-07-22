# types SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| 域 | `/types/`（decimal + canonical） |
| 镜像 | `.agents/ssot/types/**`（R6 只读；**禁止**改镜像冒充本仓完成） |
| 审计日期 | 2026-07-21；**defer-close 复核 2026-07-22** |
| 跟进 | 2026-07-21 P0/P1 **#98**；W1–W2 证据 **#121/#124**；四包内部 GO **#159** · tag `v0.3.0-four-crates`；**defer-close**：wire schema / panicking-ops gate / envelope |
| 内部生产层级 | **decimalx = L1 Internal Ready**；**canonical = L2 committed wire subset（v1–v1.3）+ envelope** |
| 结论 | **两 crate 均已注册 workspace 并有可运行测试**；decimal 不变量已硬化；canonical committed wire 覆盖 v1–v1.3；分层内部 GO（**≠** package Production Ready / crates.io / Agent L5） |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 上游镜像 COMPLETE / Spec Approved 叙事 | 描述的是 **xhyper monorepo 战役**；**禁止**单独当作本仓交付证明 |
| 本仓 `crates/types/decimal` | **已落地**（package **`decimalx`** / lib `decimalx`）；字段私有 + 校验 serde + `DecimalError` |
| 本仓 `crates/types/canonical` | **已落地**（package **`canonical`** / lib `canonical`）；committed wire v1 / v1.1 / v1.2 / v1.3 子集冻结 + **`Envelope<T>`** |
| 内部生产 GO（声明层级） | decimalx **L1**；canonical **L2 wire 子集**；证据 [`../plans/releases/2026-07-21-four-crates-internal-release.md`](../plans/releases/2026-07-21-four-crates-internal-release.md) |
| `infra-core` | **已移除**；types 不依赖它 |
| package stable / crates.io | **未**宣称；`publish = false` |
| wire schema 版本常量 | **PASS**：`decimalx::WIRE_SCHEMA_VERSION`（见 `docs/WIRE.md`）；破坏性变更须升版本 |
| panicking 算子默认关闭 | **PASS**：feature `panicking-ops` **default off**；生产主路径 `checked_*` |
| schema_version envelope | **PASS**：`crates/types/canonical/src/envelope.rs` · `Envelope { schema_version, payload }` |
| public-api / examples / benches | **PASS**：baselines + `public_api_surface` + `examples/basic` + `hot_path` |

## 本仓可观察事实

```text
crates/types/decimal/           EXISTS · members 已注册
  package                       decimalx（-p decimalx）
  lib                           decimalx
  version                       0.1.0
  publish                       false
  生产依赖                      kernel + serde
  Active SSOT                   .agents/ssot/types/decimal/spec/spec.md
  examples / baseline           examples/basic.rs · docs/api-baselines/decimalx.txt

crates/types/canonical/         EXISTS · members 已注册
  package                       canonical（-p canonical）
  lib                           canonical
  version                       0.1.0
  publish                       false
  生产依赖                      decimalx + serde
  Active SSOT                   .agents/ssot/types/canonical/spec/spec.md
  wire 模块                     src/wire.rs（Committed v1 / v1.1 / v1.2 / v1.3 清单与策略）
  envelope                      src/envelope.rs（schema_version + payload）
  examples / baseline           examples/basic.rs · docs/api-baselines/canonical.txt
  decimal WIRE_SCHEMA_VERSION   crates/types/decimal/src/lib.rs = 1 · docs/WIRE.md
  panicking-ops                 feature default off（Cargo.toml）
```

验证（本仓权威命令）：

```bash
cargo test -p decimalx --all-targets
cargo clippy -p decimalx --all-targets -- -D warnings
cargo run -p decimalx --example basic
cargo bench -p decimalx --bench hot_path -- --quick

cargo test -p canonical --all-targets
cargo clippy -p canonical --all-targets -- -D warnings
cargo run -p canonical --example basic
cargo bench -p canonical --bench hot_path -- --quick
node scripts/quality-gates/check-canonical-align.mjs
node scripts/quality-gates/check-public-api.mjs
node scripts/quality-gates/check-decimal-no-panicking-ops.mjs
```

## 依赖方向（本仓）

```text
canonical  →  decimalx  →  kernel
```

- 禁止 `decimalx` / `kernel` 依赖 `canonical`
- 禁止 types 依赖 `testkit` 作为 normal dependency
- 金额字段必须来自 `decimalx`；禁止 `f32`/`f64` 金额

## decimalx 对齐要点

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| D-1 | 路径 `/types/decimal`；package/lib 命名 | PASS | `Cargo.toml` + workspace members |
| D-2 | Decimal 族唯一定义点；无业务政策 | PASS | `src/lib.rs` + README |
| D-3 | 生产主路径 `try_new` / `FromStr` / `checked_*` | PASS | lib + `tests/entry_checked_ops.rs` |
| D-4 | panicking ops 非生产主路径 | **PASS** | feature `panicking-ops` **default off**；`#[cfg(feature = "panicking-ops")]` 门控 `+/-/*`；rustdoc `# Panics` + AGENTS |
| D-5 | 禁止 f32/f64 金额 | PASS | AGENTS + 实现无浮点金额 API |
| D-6 | 生产依赖仅 kernel + serde | PASS | `Cargo.toml` |
| D-7 | unit / property 测试存在 | PASS | `tests/entry_checked_ops.rs`、`tests/proptest_ops.rs` |
| D-8 | wire shape 版本化 + 当前事实 | **PASS** | `WIRE_SCHEMA_VERSION = 1` · `docs/WIRE.md` · wire golden 测；**≠** 跨语言 multi-major stable 产品宣称 |
| D-9 | package stable / Spec Approved | OPEN | 未宣称；见 SSOT residual |
| D-10 | 字段私有；非法 scale 不可表示 | PASS | `Decimal`/`Currency`/`Money` 私有字段；`new` 拒 `> MAX_SCALE` |
| D-11 | 校验型 serde | PASS | `Deserialize` → `try_new`；非法 scale/currency 失败 |
| D-12 | `DecimalError` 可分类 + 中文 Display | PASS | Scale/Mantissa/DivisionByZero/Rounding/Representation… |
| D-13 | `checked_*` 对可达状态无 panic | PASS | 溢出/除零/边界单测 |
| D-14 | 中间值溢出合同显式 | PASS | 文档 + `RepresentationOverflow` / `MantissaOverflow` |
| D-15 | `[lints] workspace = true` | PASS | `Cargo.toml` |

## canonical 对齐要点

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| C-1 | 路径 `/types/canonical`；package/lib 命名 | PASS | `Cargo.toml` + workspace members |
| C-2 | 纯 DTO；无业务方法 / 无 I/O | PASS | `src/lib.rs` + AGENTS |
| C-3 | Money 复用 decimalx | PASS | 依赖 package `decimalx`；类型别名/字段 |
| C-4 | `OrderId` 类型已删；id 为 `String`；优先 `OrderRef` | PASS | lib + tests |
| C-5 | DTO `ts: i64` = Unix ns（CAN-TIME-001） | PASS | lib + `proposed_time` |
| C-6 | `shape::*` / `proposed_time::*` 公开 | PASS | 模块 + `tests/public_api.rs` |
| C-7 | 依赖仅 decimalx + serde | PASS | `Cargo.toml` |
| C-8 | align script 可跑 | PASS | `node scripts/quality-gates/check-canonical-align.mjs` |
| C-9 | 全 wire Production Ready / package stable | OPEN | **未**宣称；committed 仅限清单类型，无 package stable |
| C-10 | Committed wire 清单 | PASS | `COMMITTED_WIRE_V1` 五类型 + `V1_1` Order + `V1_2` Tick/Trade + `V1_3` Position/OrderBookSnapshot/PriceLevel/SymbolMeta |
| C-11 | committed 类型 `deny_unknown_fields` | PASS | 全部 committed DTO derive + wire 拒绝测 |
| C-12 | 双向 golden / N-1 / 拒绝样例 | PASS | `wire` 单元测 + `fixtures/market/canonical/v1{,.1,.2,.3}/` |
| C-13 | 未晋升类型诚实标注 | PASS | 公开市场 DTO 均已晋升；Money/alias 不在 committed 清单（wire SSOT 在 decimalx / alias） |
| C-14 | `[lints] workspace = true` | PASS | `Cargo.toml` |
| C-15 | `schema_version` envelope | **PASS** | `src/envelope.rs` · `Envelope::wrap/expect_version` · unit + public_api_surface |

## 与镜像文档的关系

- `.agents/ssot/types/**`：只读镜像；禁止本地改 CLOSED/COMPLETE 叙事冒充同步
- 实现 SSOT 以 **源码 + 本仓 `cargo test` / align 脚本输出** 为准
- 候选完整规范在 `20260717/`：Draft/历史战役文档，**不**自动覆盖 active `spec/spec.md`
- 生产就绪审计跟进：[docs/report/2026-07-21/core-crates-production-readiness.md](../report/2026-07-21/core-crates-production-readiness.md) §11
- 详见 `.agents/ssot/SSOT.md` R6 / R7 与根 `AGENTS.md`

## OBJECTIVE 处置（2026-07-22 defer-close）

| 项 | 前状态 | 现状态 | 证据 |
|----|--------|--------|------|
| decimal wire 跨版本 stable | DEFER | **PASS（声明层）** | `WIRE_SCHEMA_VERSION` + `docs/WIRE.md`；破坏性字段变更须升版本 |
| panicking 算子仍公开 | DEFER | **PASS** | `panicking-ops` feature default off |
| canonical schema_version envelope | DEFER | **PASS** | `crates/types/canonical/src/envelope.rs` |

## 未做（follow-up / 诚实边界）

- decimal：full productized fuzz；跨语言 multi-major wire 产品协议；package stable
- decimal：oracle / boundary 门禁 **已有**（#121）；scheduled mutants/miri **有 CI**，非每次 PR 阻断
- canonical：package stable / 跨语言 wire 产品协议；镜像 `wire-commitment-matrix.md` 与实现清单同步（上游 R6）
- 上游 SSOT 镜像措辞收口（应在 xhyper.rs 修，再删除感知同步）
- **Agent L5 / Production Ready 人签** — 未填

## 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-22 | **defer-close**：WIRE_SCHEMA_VERSION / panicking-ops gate / Envelope PASS |
| 2026-07-21 | 初版：decimal + canonical 本仓落地状态；配合移除 `infra-core` 后的 workspace 地图 |
| 2026-07-21 | 生产就绪闭合：字段私有 / DecimalError / committed wire v1 / Uncommitted 标注；同步 PR #98 |
| 2026-07-21 | PR #98 合入 main：本对齐文随主干生效 |
| 2026-07-21 | infra-asa.3：晋升 Order/Tick/Trade/Position/OrderBookSnapshot/PriceLevel/SymbolMeta 为 committed v1.1–v1.3；**≠** package Production Ready |
| 2026-07-21 | 四包内部 GO：package 名 `decimalx`/`canonical`；L1 / L2 分层；examples/surface/baseline；PR #159 · tag `v0.3.0-four-crates` |
