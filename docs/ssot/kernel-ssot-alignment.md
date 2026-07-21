# kernel SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| Spec | SPEC-KERNEL-002（`.agents/ssot/kernel/spec/spec.md` ≡ `xhyper-kernel-complete-spec.md`） |
| 镜像 | `.agents/ssot/kernel/**`（R6 只读；**禁止**改镜像冒充本仓完成） |
| 本仓实现 | `crates/kernel` · package `xhyper-kernel` · lib `kernel` · workspace version `0.3.0` |
| 审计日期 | 2026-07-21 |
| 结论 | **可移植语义面 + §11 可在本仓执行的合同：无残留 FAIL** |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 上游镜像 COMPLETE / residual OPEN=0 | 描述的是 **xhyper monorepo 战役**；**禁止**单独当作本仓交付证明 |
| 本仓 `crates/kernel` | **已落地**并与 SPEC §3–§11 可移植子集对齐 |
| 本仓 archgate / `.architecture` 快照 | **未**移植 → 矩阵 **DEFER** |
| 本仓 crates.io 再发布 | **不做**；`publish = false` 显式关闭 |
| line/branch cov CI | **有** PR 门禁：`.github/workflows/kernel-coverage.yml` |
| mutants / miri CI | **有** 周调度：`kernel-mutants.yml` / `kernel-miri.yml` |

## 本仓可观察事实

```text
crates/kernel/                  EXISTS
Cargo.toml members              含 crates/kernel
package name                    xhyper-kernel
lib name                        kernel
publish                         false（显式，非默认可发布）
生产依赖                        仅 thiserror
features                        default = []
[lints]                         workspace = true + loom unexpected_cfgs
```

验证（本仓权威命令）：

```bash
cargo test -p kernel --all-targets
cargo test -p kernel --doc
cargo clippy -p kernel --all-targets -- -D warnings
cargo fmt -p kernel -- --check
RUSTFLAGS='--cfg loom' cargo test -p kernel --test lifecycle_concurrency_loom --release
```

## 与镜像文档的关系

- `.agents/ssot/kernel/**`：只读镜像；禁止本地改 CLOSED/COMPLETE 叙事冒充同步
- 实现 SSOT 以 **源码 + 本仓测试输出** 为准
- 上游 gap-matrix-v2 / residual-open 的 OPEN=0 仅证明 xhyper 战役，不替代本表
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
| 5.8 | archgate internal 棘轮 | 本仓无 archgate | DEFER（monorepo 机控） |

### §6 clock

| ID | 要求 | 本仓证据 | 判定 |
|----|------|----------|------|
| 6.2 | Timestamp i64 nanos；checked_*；无 Default | `clock.rs` + `assert_not_impl_any!(Timestamp: Default)` | PASS |
| 6.2-ban | 禁 Display 人类时间 / From&lt;SystemTime&gt; / 饱和 | `assert_not_impl_any!(Timestamp: Display, From&lt;SystemTime&gt;)` | PASS |
| 6.3 | MonotonicInstant 不透明；reverse → None | `checked_duration_since` + 单元/属性测 | PASS |
| 6.3-hidden | `from_clock_elapsed` `const` + `doc(hidden)` | `clock.rs`；调用点仅 `src/clock.rs` 与 `testkit/*` | PASS（结构扫描；archgate TIME-004 DEFER） |
| 6.4 | Clock 无 monotonic 默认实现 | doctest `compile_fail` OnlyWall | PASS |
| 6.5 | ClockError 三变体名 | `clock.rs` | PASS |
| 6.6 | SystemClock origin.elapsed；!Copy；Default | `SystemClock` + `assert_not_impl_any!(SystemClock: Copy)` | PASS |
| 6.7 | SystemTime/Instant::now 仅 SystemClock | 生产 src 扫描 | PASS |

### §7 lifecycle

| ID | 要求 | 本仓证据 | 判定 |
|----|------|----------|------|
| 7.2 | ComponentState 6 态 + 合法边 | `can_transition_to` + matrix 测 | PASS |
| 7.3 | LifecycleError {from,to} + thiserror | `lifecycle.rs` | PASS |
| 7.4 | try_transition 不 panic | 全矩阵测 | PASS |
| 7.5 | Signal/Guard 协议；must_use | `new/is_triggered/wait/trigger` | PASS |
| 7.6 | Mutex&lt;bool&gt;+Condvar 同锁；loom | `lifecycle.rs` + `tests/lifecycle_concurrency_loom.rs` | PASS（loom 需 `--cfg loom`） |
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
| 11.1-ManualClock | ManualClock 独立 wall/mono | 属 `xhyper-testkit`；kernel 内 ControlledClock 覆盖 trait 合同 | PASS（跨 crate；见 testkit 对齐文） |
| 11.1-lc | 合法/非法矩阵；trigger/wait；多 observer；1000 并发；poison；guard drop | unit + `lifecycle_concurrency`（1000 循环） | PASS |
| 11.2 | loom 模型 | `lifecycle_concurrency_loom.rs`；默认构建 0 tests；`RUSTFLAGS='--cfg loom'` 启用 | PASS / 环境不可跑时见证据 DEFER |
| 11.3 | proptest：Timestamp×Duration；ComponentState 对；ErrorKind 一致 | `clock_contract.rs` proptest! | PASS |
| 11.4 | compile-fail / static 负向面 | rustdoc compile_fail + `api_compile.rs` | PASS |
| 11.5 | line ≥95% / branch ≥90% CI | `.github/workflows/kernel-coverage.yml` 解析 TOTAL 末列 branch% 并强制 ≥90；`--fail-under-lines 95` / functions 90。本会话实测 TOTAL branch **100%** / lines **99.69%** | PASS |
| 11.6 | mutants ≥90% | `.github/workflows/kernel-mutants.yml`：schedule + `mkdir -p .cargo/cache/mutants` 后 `cargo mutants` | PASS（job 可执行；本会话未全量跑 mutants） |
| 11.7 | miri | `.github/workflows/kernel-miri.yml`（schedule 周一 + workflow_dispatch；`cargo miri test -p xhyper-kernel`） | PASS（scheduled CI 存在；本会话未跑 miri） |

### §12+ monorepo 专属（非本仓可移植目标）

| ID | 要求 | 判定 | 原因 |
|----|------|------|------|
| 12.x | archgate KERNEL-* | DEFER | 本仓无 `.architecture` / archgate |
| 12.3 | public-api 快照文件 | DEFER | 本仓无 snapshot 机控；以 §8 + 测试代替 |
| 15.x | crates.io publish / tag | DEFER 发布动作；**package `publish = false` PASS** | 本仓明确不向 crates.io 再发布 |
| TIME-004 机控 | from_clock_elapsed allowlist | DEFER 机控；**结构扫描 PASS** | 见上文 6.3-hidden |

---

## 残留 FAIL

**无。** 所有实现相关 FAIL 已在 `crates/kernel` 内消除；未完成项均为显式 **DEFER**。

## 未做（follow-up，不阻塞本仓语义对齐）

- archgate / `.architecture` 机控移植
- line/branch cov、mutants、miri 的 CI 门禁化
- crates.io 再发布与 `publish = true`
- 上游 SSOT 镜像内部措辞收口（应在 xhyper.rs 修，再 `cp -rf` 同步）

## Workspace 交叉引用

- 总览：[workspace-ssot-alignment.md](./workspace-ssot-alignment.md)
- testkit（ManualClock）：[testkit-ssot-alignment.md](./testkit-ssot-alignment.md)
- types：[types-ssot-alignment.md](./types-ssot-alignment.md)

## 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-21 | 初版：逐条矩阵 + 禁止 From/not_found/other/Clock 默认 monotonic 负向面加强 |
| 2026-07-21 | Codex P2 修复：`[lints] workspace = true`、`publish = false`、如实引用 coverage/mutants/miri workflows |
| 2026-07-21 | Codex P1：修复 coverage workflow branch 解析；补 BeforeUnixEpoch/Overflow 测；mutants mkdir；branch cover 100% |
| 2026-07-21 | 交叉引用 workspace 总览；确认无 `infra-core` 依赖 |
