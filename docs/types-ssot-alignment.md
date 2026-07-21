# types SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| 域 | `/types/`（decimal + canonical） |
| 镜像 | `.agents/ssot/types/**`（R6 只读；**禁止**改镜像冒充本仓完成） |
| 审计日期 | 2026-07-21 |
| 结论 | **两 crate 均已注册 workspace 并有可运行测试**；wire/package stable **未**宣称 |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 上游镜像 COMPLETE / Spec Approved 叙事 | 描述的是 **xhyper monorepo 战役**；**禁止**单独当作本仓交付证明 |
| 本仓 `crates/types/decimal` | **已落地**（package `xhyper-decimalx` / lib `decimalx`） |
| 本仓 `crates/types/canonical` | **已落地**（package `xhyper-canonical` / lib `canonical`） |
| `infra-core` | **已移除**；types 不依赖它 |
| package stable / crates.io | **未**宣称；`publish = false` |
| 全量 wire stable | **未**宣称；见各 crate residual / WIRE 文档 |

## 本仓可观察事实

```text
crates/types/decimal/           EXISTS · members 已注册
  package                       xhyper-decimalx
  lib                           decimalx
  version                       0.1.0
  publish                       false
  生产依赖                      xhyper-kernel + serde
  Active SSOT                   .agents/ssot/types/decimal/spec/spec.md

crates/types/canonical/         EXISTS · members 已注册
  package                       xhyper-canonical
  lib                           canonical
  version                       0.1.0
  publish                       false
  生产依赖                      xhyper-decimalx + serde
  Active SSOT                   .agents/ssot/types/canonical/spec/spec.md
```

验证（本仓权威命令）：

```bash
cargo test -p xhyper-decimalx --all-targets
cargo clippy -p xhyper-decimalx --all-targets -- -D warnings

cargo test -p xhyper-canonical --all-targets
cargo clippy -p xhyper-canonical --all-targets -- -D warnings
node scripts/check-canonical-align.mjs
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
| D-4 | panicking ops 非生产主路径 | PASS | rustdoc `# Panics` + AGENTS |
| D-5 | 禁止 f32/f64 金额 | PASS | AGENTS + 实现无浮点金额 API |
| D-6 | 生产依赖仅 kernel + serde | PASS | `Cargo.toml` |
| D-7 | unit / property 测试存在 | PASS | `tests/entry_checked_ops.rs`、`tests/proptest_ops.rs` |
| D-8 | wire shape = 当前事实 ≠ stable 承诺 | PASS（文档） | `docs/WIRE.md` + residual |
| D-9 | package stable / Spec Approved | OPEN | 未宣称；见 SSOT residual |

## canonical 对齐要点

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| C-1 | 路径 `/types/canonical`；package/lib 命名 | PASS | `Cargo.toml` + workspace members |
| C-2 | 纯 DTO；无业务方法 / 无 I/O | PASS | `src/lib.rs` + AGENTS |
| C-3 | Money 复用 decimalx | PASS | 依赖 `xhyper-decimalx`；类型别名/字段 |
| C-4 | `OrderId` 类型已删；id 为 `String`；优先 `OrderRef` | PASS | lib + tests |
| C-5 | DTO `ts: i64` = Unix ns（CAN-TIME-001） | PASS | lib + `proposed_time` |
| C-6 | `shape::*` / `proposed_time::*` 公开 | PASS | 模块 + `tests/public_api.rs` |
| C-7 | 依赖仅 decimalx + serde | PASS | `Cargo.toml` |
| C-8 | align script 可跑 | PASS | `node scripts/check-canonical-align.mjs` |
| C-9 | 全 wire Production Ready / package stable | OPEN | 未宣称；见 plan/residual |

## 与镜像文档的关系

- `.agents/ssot/types/**`：只读镜像；禁止本地改 CLOSED/COMPLETE 叙事冒充同步
- 实现 SSOT 以 **源码 + 本仓 `cargo test` / align 脚本输出** 为准
- 候选完整规范在 `20260717/`：Draft/历史战役文档，**不**自动覆盖 active `spec/spec.md`
- 详见 `.agents/ssot/SSOT.md` R6 / R7 与根 `AGENTS.md`

## 未做（follow-up / OPEN）

- decimal：`MAX_SCALE` 治理层正式批准、字段 `pub` 收口、wire stable
- canonical：全 DTO wire 承诺矩阵闭合、package stable
- types 专用 coverage / mutants / miri CI（若需要与 kernel/testkit 同级）
- 上游 SSOT 镜像措辞收口（应在 xhyper.rs 修，再 `cp -rf` 同步）

## 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-21 | 初版：decimal + canonical 本仓落地状态；配合移除 `infra-core` 后的 workspace 地图 |
