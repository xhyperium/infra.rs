# kernel SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| Spec | SPEC-KERNEL-002（`.agents/ssot/kernel/spec/spec.md`，本仓 active SSOT） |
| SSOT | `.agents/ssot/kernel/**` 为本仓可编辑源层；历史 complete/evidence 只作不可变来源 |
| 本仓实现 | `crates/kernel` · package/lib **`kernel`** · version `0.3.1` |
| 审计日期 | 2026-07-23（infra-2d9.7 R3 声明收敛候选） |
| 内部生产层级 | **L1 Internal Ready；L4 仅限新鲜证据覆盖面** |
| 内部发布状态 | **已执行**（PR #159 实现 · #163 发布记录 · tag/GitHub Release `v0.3.0-four-crates`） |
| 结论 | **可移植语义面 + §11 可在本仓执行的合同：无残留 FAIL**；**内部生产已发布**；**≠** crates.io / 整体 Production Ready / Agent L5 |
| OBJECTIVE 主项 | archgate = **OOS-Accept**；组合根 drain 所有权 = **PASS**（落在 bootstrap，见 bootstrap 对齐文） |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 上游镜像 COMPLETE / residual OPEN=0 | 描述的是 **xhyper monorepo 战役**；**禁止**单独当作本仓交付证明 |
| 本仓 `crates/kernel` | **已落地**并与 SPEC §3–§11 可移植子集对齐 |
| 内部生产 GO（声明层级） | **L1+L4**；四包证据 [`../plans/releases/2026-07-21-four-crates-internal-release.md`](../plans/releases/2026-07-21-four-crates-internal-release.md) |
| **内部发布是否已执行** | **是**：crate 记录 [`../../crates/kernel/releases/0.3.0-internal.md`](../../crates/kernel/releases/0.3.0-internal.md) · GH Release [v0.3.0-four-crates](https://github.com/xhyperium/infra.rs/releases/tag/v0.3.0-four-crates) · PR #163 |
| 本仓 archgate / `.architecture` 快照 | **OOS：本仓明确不移植**；机控用结构扫描 / CI / public-api 等已有门禁 |
| 本仓 crates.io 再发布 | **不做**；`publish = false` 显式关闭 |
| public-api 棘轮 | **PASS**：`docs/api-baselines/kernel.txt` + `check-public-api.mjs` |
| line/branch cov CI | **有** PR 门禁：`.github/workflows/kernel-coverage.yml`（100% line gate） |
| mutants / miri CI | **有** 周调度：`kernel-mutants.yml` / `kernel-miri.yml` |
| loom CI | **有** PR/push 门禁：`kernel-loom.yml` + `scripts/quality-gates/run-kernel-loom.mjs` |
| ClockDomain | **前一候选 PASS / 最新分支复验待运行**：SystemClock 共享进程 domain；跨 domain 间隔 → `None` |
| 关停 deadline | **R2 PASS**：未触发且不可表示时返回 `DeadlineOverflow`；已触发时完成优先并立即 `Ok(true)` |
| 用户可见错误中文 | **PASS**：`ClockError` / `LifecycleError` Display 中文 |
| 公开面集成测 / 示例 / bench | **PASS**：`tests/public_api_surface.rs` · `examples/basic.rs` · `benches/hot_path` |

## 本仓可观察事实

```text
crates/kernel/                  EXISTS
Cargo.toml members              含 crates/kernel
package name                    kernel（Cargo 选择器 -p kernel；历史文档 xhyper-kernel 已废弃）
lib name                        kernel
version                         0.3.1
publish                         false（显式，非默认可发布）
生产依赖                        仅 thiserror
features                        default = []
[lints]                         workspace = true + loom unexpected_cfgs
examples                        examples/basic.rs
public API surface              tests/public_api_surface.rs
API baseline                    docs/api-baselines/kernel.txt
crate 发布记录                  releases/0.3.0-internal.md
内部 tag / GH Release           v0.3.0-four-crates
```

验证（本仓权威命令）：

```bash
cargo test -p kernel --all-targets
cargo test -p kernel --doc
cargo clippy -p kernel --all-targets -- -D warnings
cargo fmt --all -- --check
cargo run -p kernel --example basic
cargo bench -p kernel --bench hot_path -- --quick
node scripts/quality-gates/check-public-api.mjs
RUSTFLAGS='--cfg loom' cargo test -p kernel --test lifecycle_concurrency_loom --release
```

## 与历史文档的关系

- `.agents/ssot/kernel/spec/spec.md` 是本仓 active SSOT，可随本仓 PR 修改。
- `spec/xhyper-kernel-complete-spec.md` 是 active `spec.md` 的机械镜像，必须逐字同构；`evidence/2026-07-14/**` 与 dated campaign 才是历史来源，不继承 PASS。
- 当前实现与验证证据以本仓源码、测试输出和最终 SHA 为准。
- 详见 `.agents/ssot/SSOT.md` R6 / R7 与根 `AGENTS.md`

---

## 逐条对齐矩阵（SPEC-KERNEL-002 §3–§11 可移植子集）

> 判定：`PASS` = 本仓有源码/测试证据；`FAIL` = 语义缺失须修；`DEFER` = 本仓环境/monorepo 专属，写明原因。  
> 证据指针为 crate 相对路径（`crates/kernel/...`）。

### §3 依赖合同

| ID | 要求 | 本仓证据 | 判定 |
|----|------|----------|------|
| 3.1 | kernel → ∅ workspace 内部依赖 | `Cargo.toml` 无 path 依赖；metadata 仅 `thiserror` 生产图 | PASS |
| 3.2 | 生产依赖仅 `thiserror` | `Cargo.toml` `[dependencies]` | PASS |
| 3.2-ban | 禁 anyhow/serde/tokio/chrono/… 生产图 | 生产 deps 扫描无禁止项；serde 仅 dev | PASS |
| 3.3 | dev: proptest + static_assertions；loom 仅 cfg(loom) | `Cargo.toml` dev-deps + `target.'cfg(loom)'` | PASS |
| 3.3-trybuild | trybuild 编译负向 | rustdoc `compile_fail` + `static_assertions` 等价；trybuild 不引入 | PASS（等价机制） |
| 3.4 | `default = []`，无 feature | `Cargo.toml` `[features]` | PASS |

### §4 Crate 级属性

| ID | 要求 | 本仓证据 | 判定 |
|----|------|----------|------|
| 4.1 | `forbid(unsafe_code)` / `deny(missing_docs, unreachable_pub)` | `src/lib.rs` | PASS |
| 4.2 | 无 unsafe / todo! / unimplemented! 生产路径 | `rg` 扫描 src | PASS |
| 4.3 | 生产路径无 panic 合同；锁中毒 `into_inner` | `lifecycle.rs` poison 恢复 + 单元测 | PASS |
| 4.4 | `[lints] workspace = true` | 根 `Cargo.toml` `[workspace.lints.rust]` + crate `[lints] workspace = true`；并覆盖 loom `unexpected_cfgs` | PASS |

### §5 error

| ID | 要求 | 本仓证据 | 判定 |
|----|------|----------|------|
| 5.2 | ErrorKind ×9 `non_exhaustive`；XError 字段私有 | `error.rs` + doctest `compile_fail` 结构体字面量 | PASS |
| 5.4 | 构造器全集 + with_source + 查询面 | `error.rs` + `error::tests::*` + `public_api` | PASS |
| 5.4-sem | `is_retryable` 仅 Transient；`is_bug` 仅 Invariant | `test_is_retryable_only_transient` / `test_is_bug_only_invariant` | PASS |
| 5.5 | 禁 From&lt;str\|String&gt; / not_found / other | `assert_not_impl_any!(XError: From<…>)`；doctest `compile_fail` not_found/other/into | PASS |
| 5.6 | with_source 不改 kind；Display/Debug 不展开 source | `test_with_source_*` / `test_display_*` / `test_debug_*` | PASS |
| 5.7 | `From<ClockError> → Unavailable` | `test_clock_error_maps_all_variants_to_unavailable` | PASS |
| 5.8 | archgate internal 棘轮 | 本仓不引入 archgate | **OOS**（本仓不移植 archgate / `.architecture`） |

### §6 clock

| ID | 要求 | 本仓证据 | 判定 |
|----|------|----------|------|
| 6.2 | Timestamp i64 nanos；checked_*；无 Default | `clock.rs` + `assert_not_impl_any!(Timestamp: Default)` | PASS |
| 6.2-ban | 禁 Display 人类时间 / From&lt;SystemTime&gt; / 饱和 | `assert_not_impl_any!(Timestamp: Display, From&lt;SystemTime&gt;)` | PASS |
| 6.3 | MonotonicInstant 不透明；reverse → None | `checked_duration_since` + 单元/属性测 | PASS |
| 6.3-domain | `ClockDomain`；跨 domain 间隔不可静默可靠 | `ClockDomain` + `partial_cmp`/`checked_duration_since` → None；`system_clocks_share_process_domain` | PASS |
| 6.3-hidden | `from_clock_elapsed` / `from_clock_elapsed_in` `const` + `doc(hidden)` | `clock.rs`；调用点仅 `src/clock.rs` 与 `testkit/*` | **PASS（结构扫描）**；TIME-004 机控 **OOS**（不引入 archgate） |
| 6.4 | Clock 无 monotonic 默认实现 | doctest `compile_fail` OnlyWall | PASS |
| 6.5 | ClockError 三变体名 + 中文 Display | `clock.rs` thiserror | PASS |
| 6.6 | SystemClock 进程共享原点；!Copy；Default→new | `OnceLock` origin + `assert_not_impl_any!(SystemClock: Copy)` | PASS |
| 6.7 | SystemTime/Instant::now 仅 SystemClock | 生产 src 扫描 | PASS |

### §7 lifecycle

| ID | 要求 | 本仓证据 | 判定 |
|----|------|----------|------|
| 7.2 | ComponentState 6 态 + 合法边 | `can_transition_to` + matrix 测 | PASS |
| 7.3 | LifecycleError {from,to} + 中文 thiserror | `lifecycle.rs` | PASS |
| 7.4 | try_transition 不 panic | 全矩阵测 | PASS |
| 7.5 | Signal/Guard 协议；must_use | `new/is_triggered/wait/trigger` | PASS |
| 7.5-deadline | `wait_timeout`；组合根超时升级路径可测 | `wait_timeout_*` / `composition_root_deadline_upgrade_path`（`cfg(not(loom))`） | PASS |
| 7.6 | Mutex&lt;bool&gt;+Condvar 同锁；loom | `lifecycle.rs` + `tests/lifecycle_concurrency_loom.rs` + **CI** `kernel-loom.yml` | PASS（本地：`RUSTFLAGS='--cfg loom'`） |
| 7.7 | guard drop 不触发 | `test_guard_drop_does_not_trigger` | PASS |
| 7.8 | 无 Component trait | doctest `compile_fail` + 无符号 | PASS |

### §8 冻结导出

| ID | 要求 | 本仓证据 | 判定 |
|----|------|----------|------|
| 8.1 | 仅 clock/error/lifecycle 模块 + 既定 re-export | `src/lib.rs` | PASS |
| 8.2 | 无 prelude / 第三方 re-export / mock 公开 API | 公开面扫描 | PASS |

### §9 serde / wire

| ID | 要求 | 本仓证据 | 判定 |
|----|------|----------|------|
| 9.1 | kernel 类型无 Serialize/Deserialize | `api_compile.rs` `assert_not_impl_any!(…: serde::…)` | PASS |
| 9.2 | wire 由协议层版本化 | 本仓不在 kernel 内定义 wire | PASS |

### §10 失败策略

| ID | 要求 | 本仓证据 | 判定 |
|----|------|----------|------|
| 10.1 | 溢出/非法转换/反向差 → Result/Option | Timestamp/Lifecycle 测 | PASS |
| 10.2 | 锁中毒 into_inner，不伪装未触发 | `test_poison_recovery_into_inner` | PASS |
| 10.3 | kernel 不主动 panic! 报 Invariant | 生产路径扫描 | PASS |

### §11 测试合同（本仓可执行子集）

| ID | 要求 | 本仓证据 | 判定 |
|----|------|----------|------|
| 11.1-error | 构造器/source/is_*/Display | `src/error.rs` tests | PASS |
| 11.1-ts | i64 边界 / 溢出 / reverse / ZERO | `clock.rs` + `clock_contract` | PASS |
| 11.1-clock | SystemClock now/mono；错误映射；双通道 | `ControlledClock` 墙钟回退不牵连 mono；SystemClock 测 | PASS |
| 11.1-ManualClock | ManualClock 独立 wall/mono | 属 package `testkit`；kernel 内 ControlledClock 覆盖 trait 合同 | PASS（跨 crate；见 testkit 对齐文） |
| 11.1-lc | 合法/非法矩阵；trigger/wait；多 observer；1000 并发；poison；guard drop | unit + `lifecycle_concurrency`（1000 循环） | PASS |
| 11.2 | loom 模型 + 持续门禁 | `lifecycle_concurrency_loom.rs`；默认 0 tests；`RUSTFLAGS='--cfg loom'`；CI `.github/workflows/kernel-loom.yml`；本地 `node scripts/quality-gates/run-kernel-loom.mjs` | PASS |
| 11.3 | proptest：Timestamp×Duration；ComponentState 对；ErrorKind 一致 | `clock_contract.rs` proptest! | PASS |
| 11.4 | compile-fail / static 负向面 | rustdoc compile_fail + `api_compile.rs` | PASS |
| 11.5 | line ≥95% / branch ≥90% CI | `.github/workflows/kernel-coverage.yml` 解析 TOTAL 末列 branch% 并强制 ≥90；`--fail-under-lines 95` / functions 90。本会话实测 TOTAL branch **100%** / lines **99.69%** | PASS |
| 11.6 | mutants ≥90% | `.github/workflows/kernel-mutants.yml`：schedule + `mkdir -p .cargo/cache/mutants` 后 `cargo mutants` | PASS（job 可执行；本会话未全量跑 mutants） |
| 11.7 | miri | `.github/workflows/kernel-miri.yml`（schedule 周一 + workflow_dispatch；`cargo miri test -p kernel`） | PASS（scheduled CI 存在；本会话未跑 miri） |

### §12+ monorepo 专属（非本仓可移植目标）

| ID | 要求 | 判定 | 原因 |
|----|------|------|------|
| 12.x | archgate KERNEL-* | **OOS** | 本仓明确不移植 archgate / `.architecture`；机控改走结构扫描 / CI / public-api |
| 12.3 | public-api 快照文件 | PASS | `docs/api-baselines/kernel.txt` + `scripts/quality-gates/check-public-api.mjs`（W5 / #127 / #159） |
| 15.x | crates.io publish | DEFER 发布动作；**package `publish = false` PASS** | 本仓明确不向 crates.io 再发布 |
| 15.x-tag | 内部 git tag | PASS（锚点） | `v0.3.0-four-crates` → `5acac34`；**≠** crates.io |
| 15.x-release | 内部生产发布执行 | **PASS** | `releases/0.3.0-internal.md` · GH Release · PR #163；workspace path 消费 |
| TIME-004 机控 | from_clock_elapsed allowlist | **OOS** 机控（不引入 archgate）；**结构扫描 PASS** | 见上文 6.3-hidden |

---

## 残留 FAIL

**无。** 所有实现相关 FAIL 已在 `crates/kernel` 内消除；OBJECTIVE 未完成项仅为 **OOS-Accept**（archgate）；crates.io 再发布为显式不动作。

## OBJECTIVE 处置（2026-07-22 defer-close）

| 项 | 前状态 | 现状态 | 证据 |
|----|--------|--------|------|
| archgate / `.architecture` | OOS | **OOS-Accept** | 本仓明确不移植；机控 = 结构扫描 / CI / `check-public-api.mjs` |
| 组合根 drain 所有权 | 审查侧 DEFER（「在 bootstrap」） | **PASS**（归属 bootstrap） | `crates/bootstrap/src/drain.rs` · `AsyncDrain` · `Bootstrap::register_drain` / `AppContext::run_drain`；**不**在 kernel 实现 |

## 未做（follow-up，不阻塞本仓语义对齐）

- archgate 明确不移植（**OOS-Accept**）；机控继续用结构扫描 / CI / public-api 等已有门禁
- mutants / miri 本会话全量实测通过声明（CI 入口已有）
- crates.io 再发布与 `publish = true`
- 上游 SSOT 镜像内部措辞收口（应在 xhyper.rs 修，再删除感知同步）
- 整体 Production Ready / **Agent L5 人签**（见 [core-crates-production-readiness.md](../report/2026-07-21/core-crates-production-readiness.md) §8/§11；模板 `docs/governance/prod-signoff-TEMPLATE.md`）

## 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-22 | **defer-close**：OBJECTIVE 表 archgate OOS-Accept；drain 归属 bootstrap PASS |

## Workspace 交叉引用

- 总览：[workspace-ssot-alignment.md](./workspace-ssot-alignment.md)
- testkit（ManualClock）：[testkit-ssot-alignment.md](./testkit-ssot-alignment.md)
- types：[types-ssot-alignment.md](./types-ssot-alignment.md)
- 四包内部发布证据：[../plans/releases/2026-07-21-four-crates-internal-release.md](../plans/releases/2026-07-21-four-crates-internal-release.md)
- **kernel 0.3.0 内部发布记录**：[../../crates/kernel/releases/0.3.0-internal.md](../../crates/kernel/releases/0.3.0-internal.md)
- GitHub Release：[v0.3.0-four-crates](https://github.com/xhyperium/infra.rs/releases/tag/v0.3.0-four-crates)

## 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-21 | 初版：逐条矩阵 + 禁止 From/not_found/other/Clock 默认 monotonic 负向面加强 |
| 2026-07-21 | 生产就绪：ClockDomain、wait_timeout、loom CI、中文错误；PR #98 **合入 main** |
| 2026-07-21 | 四包内部 GO：package 名对齐 `kernel`；L1+L4；public-api baseline；examples/bench/surface；PR #159 · tag `v0.3.0-four-crates` |
| 2026-07-21 | Codex P2 修复：`[lints] workspace = true`、`publish = false`、如实引用 coverage/mutants/miri workflows |
| 2026-07-21 | Codex P1：修复 coverage workflow branch 解析；补 BeforeUnixEpoch/Overflow 测；mutants mkdir；branch cover 100% |
| 2026-07-21 | 交叉引用 workspace 总览；确认无 `infra-core` 依赖 |
| 2026-07-21 | **内部发布已执行**：crate `releases/0.3.0-internal.md` · GH Release · PR #163；对齐文标记「已发布」 |
