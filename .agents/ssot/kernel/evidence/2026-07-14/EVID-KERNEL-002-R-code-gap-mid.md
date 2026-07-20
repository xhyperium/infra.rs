> **Note:** Agent audit ran mid-migration; code path closed after. Use residual-open.txt for current residuals.

# R-code-gap：`kernel` 实现 vs SPEC-KERNEL-002（§2–§10、§15–§16）

| 项 | 值 |
|----|-----|
| SSOT | `.agent/SSOT/kernel/spec/spec.md`（SPEC-KERNEL-002） |
| 对照代码 | `crates/kernel/src/{error,clock,lifecycle,lib}.rs` + `Cargo.toml` |
| 审计模式 | 只读源码对照（未跑测试/clippy） |
| 日期 | 2026-07-14 |

**状态说明**：`PASS` = 与终态规范一致；`PARTIAL` = 有实现但形态/语义/迁移未闭合；`FAIL` = 缺项或与终态冲突。

---

## 汇总

| 指标 | 数量 |
|------|------|
| 表内检查项 | 48 |
| **PASS** | **20** |
| **PARTIAL** | **14** |
| **FAIL** | **14** |
| P0 residual | 7 |
| P1 residual | 16 |
| P2 residual | 5 |

### P0 残留（阻塞「代码闭合 / stable」）

| ID | 摘要 |
|----|------|
| **RES-ERR-001** | `XError` 仍为公开 enum，非 opaque `struct`（§5.2） |
| **RES-ERR-002** | 仍提供禁止 API `not_found` / `other`（§5.5 / §16.1 / §18.2） |
| **RES-CLK-001** | `Clock::monotonic` 带真实 `Instant::now` 默认实现（§6.4 / §16.2） |
| **RES-CLK-002** | `MonotonicInstant` 反向差饱和为 0，非 `checked_… → None`（§6.3） |
| **RES-CLK-003** | `SystemClock` 为 `Copy`、无固定 `origin`，`monotonic` 未按 origin.elapsed（§6.6） |
| **RES-LC-001** | `ShutdownInner` 为 `AtomicBool`+空 `Mutex`；`trigger` 不持同一 mutex（lost wake-up，§7.6） |
| **RES-API-001** | 公开导出 `Component` trait；§7.8/§8 明确禁止本版本提供 |

---

## §2 目录结构

| Spec req | Status | Evidence path:line | Residual ID | Severity |
|----------|--------|-------------------|-------------|----------|
| `src/{lib,error,clock,lifecycle}.rs` 仅此四模块，无 util/prelude 等 | PASS | `crates/kernel/src/lib.rs:21-23`；目录仅四文件 | — | — |
| 根文件：`Cargo.toml` / `README.md` / `AGENTS.md` / `CHANGELOG.md` | PASS | `crates/kernel/{Cargo.toml,README.md,AGENTS.md,CHANGELOG.md}` | — | — |
| `tests/{api_compile,clock_contract,lifecycle_concurrency,public_api}.rs` | FAIL | `crates/kernel/tests/` **不存在**；仅有 `#[cfg(test)]` 内联测试 | RES-API-002 | P1 |

---

## §3 依赖合同

| Spec req | Status | Evidence path:line | Residual ID | Severity |
|----------|--------|-------------------|-------------|----------|
| workspace 内部依赖 = ∅ | PASS | `crates/kernel/Cargo.toml:10-11` 仅 `thiserror` | — | — |
| 生产外部依赖白名单仅 `thiserror` | PASS | 同上 | — | — |
| 禁止 anyhow/serde/tokio/chrono 等 | PASS | `Cargo.toml` 无其它 deps；源码无引用 | — | — |
| `[features] default = []`，不允许任何 feature | PARTIAL | 无 `[features]` 段（等价无 feature）；规范字面要求显式 `default = []` | RES-API-003 | P2 |
| 测试可选 loom/proptest/trybuild/static_assertions | PARTIAL | 无 dev-dependencies；§11 所需 loom/trybuild 未接入（本表仅标依赖面） | RES-API-004 | P1 |

---

## §4 Crate 级属性

| Spec req | Status | Evidence path:line | Residual ID | Severity |
|----------|--------|-------------------|-------------|----------|
| `#![forbid(unsafe_code)]` | PASS | `crates/kernel/src/lib.rs:19` | — | — |
| `#![deny(missing_docs)]` | FAIL | `lib.rs` 仅 forbid unsafe，无 deny missing_docs | RES-API-005 | P1 |
| `#![deny(unreachable_pub)]` | FAIL | 同上缺失 | RES-API-006 | P1 |
| `[lints] workspace = true` | PASS | `Cargo.toml:15-16` | — | — |
| 生产路径无 `todo!`/`unimplemented!`/`unwrap`/`expect`/`panic!` | PASS | 生产 `src/*.rs` 无上述宏；`lifecycle` 锁中毒用 `into_inner` | — | — |

---

## §5 `error` 模块

| Spec req | Status | Evidence path:line | Residual ID | Severity |
|----------|--------|-------------------|-------------|----------|
| `BoxError` / `XResult<T>` 类型别名 | PASS | `error.rs:20`、`error.rs:130` | — | — |
| `ErrorKind` 九变体 + `non_exhaustive` + Copy/Eq/Hash | PASS | `error.rs:25-46` | — | — |
| **`XError` 为字段私有的 opaque `struct`** | FAIL | `error.rs:51-128` 公开 `enum` + 公开字段式变体 | **RES-ERR-001** | **P0** |
| 九种语义构造器 + 单参 `internal(context)` | PARTIAL | 有 invalid/missing/…/invariant（`error.rs:137-217`）；`internal` 为 **两参**（`219-224`），另有 `internal_msg`（`227-232`） | RES-ERR-003 | P1 |
| `with_source` / `kind` / `context` / `retry_after` / `is_retryable` / `is_bug` | PASS | `error.rs:239-333`；`is_retryable` 仅 Transient、`is_bug` 仅 Invariant | — | — |
| **禁止** `not_found` / `other` | FAIL | `error.rs:164-166`、`235-237` 仍公开；注释标 E2 删除 | **RES-ERR-002** | **P0** |
| 禁止 `From<String/&str/anyhow>` | PASS | 无此类 `From` impl | — | — |
| `with_source` 保持 kind | PASS | `error.rs:239-285` + 测试 `462-484` | — | — |
| `From<ClockError> → Unavailable` | PASS | `error.rs:341-351`；映射 BeforeEpoch/Overflow/Unavailable | RES-ERR-004（命名见 clock） | P2 |
| Display 人类可读、不链完整 source | PASS | `#[error("… {context}")]` 各变体 | — | — |
| `thiserror::Error` 用于 enum（终态 struct 亦需 Error） | PARTIAL | 现用 thiserror 于 enum；终态 opaque 后需重做 | RES-ERR-001 | P0（并入） |

---

## §6 `clock` 模块

| Spec req | Status | Evidence path:line | Residual ID | Severity |
|----------|--------|-------------------|-------------|----------|
| `Timestamp(i64)` + 比较/Hash，无 Default | PASS | `clock.rs:13-14`；无 Default impl | — | — |
| `from_unix_nanos` / `as_unix_nanos` | PASS | `clock.rs:23-25`、`33-35` | — | — |
| **禁止** `from_nanos` / `as_nanos` 简写 | FAIL | `clock.rs:18-20`、`28-30` 仍提供 | RES-CLK-004 | P1 |
| `checked_add` | PASS | `clock.rs:38-41` | — | — |
| **`checked_sub`** | FAIL | 源码无 `checked_sub` | RES-CLK-005 | P1 |
| `checked_duration_since`（earlier>self → None；相等 → Some(0)） | PASS | `clock.rs:44-50`；测试 `140-142` | — | — |
| `MonotonicInstant` 私有内部 + Hash | PARTIAL | 私有 `Instant`（`clock.rs:54-55`）；**无 Hash** | RES-CLK-006 | P2 |
| **`checked_duration_since` → Option；反向 None** | FAIL | 现为 `duration_since` 饱和 0（`clock.rs:64-66`） | **RES-CLK-002** | **P0** |
| `#[doc(hidden)] from_clock_elapsed` | FAIL | 无此 API；另有公开 `from_std`（`58-61`） | RES-CLK-007 | P1 |
| `Clock: Send+Sync`；`now` + **`monotonic` 无默认实现** | FAIL | `clock.rs:85-93` 含默认 `Instant::now` | **RES-CLK-001** | **P0** |
| `ClockError::{BeforeUnixEpoch, Overflow, Unavailable}` | PARTIAL | 变体名 `BeforeEpoch` 非 `BeforeUnixEpoch`（`clock.rs:72-81`）；语义等价 | RES-CLK-008 | P1 |
| `SystemClock { origin: Instant }` + `new`/`Default`；**非 Copy** | FAIL | `#[derive(..., Copy, Default)]` 空结构（`clock.rs:96-97`）；无 origin | **RES-CLK-003** | **P0** |
| `SystemClock::now` 失败显式错误、不返回 0 | PASS | `clock.rs:100-106` | — | — |
| `SystemClock::monotonic` = `origin.elapsed` | FAIL | 依赖 trait 默认，每次新 `Instant::now` | **RES-CLK-003** | **P0** |
| 无 serde / 无全局 Clock | PASS | 无相关 derive/静态 | — | — |

---

## §7 `lifecycle` 模块

| Spec req | Status | Evidence path:line | Residual ID | Severity |
|----------|--------|-------------------|-------------|----------|
| `ComponentState` 六态 + non_exhaustive | PASS | `lifecycle.rs:9-24` | — | — |
| 合法转换矩阵（7 条） | PASS | `lifecycle.rs:38-49` | — | — |
| `LifecycleError { from, to }` + thiserror | PASS | `lifecycle.rs:27-34` | — | — |
| `can_transition_to` / `try_transition` 不 panic | PASS | `lifecycle.rs:36-59` | — | — |
| `ShutdownSignal::new/is_triggered/wait`；`Guard::trigger(self)` | PASS | `lifecycle.rs:81-119` | — | — |
| `#[must_use]` 于 Signal/Guard | FAIL | 无 `must_use`（`lifecycle.rs:63-72`） | RES-LC-002 | P2 |
| Guard 不可 Clone；Signal 可 Clone；不可重置 | PASS | Signal `Clone`（63）；Guard 无 Clone；无 reset API | — | — |
| **同一 `Mutex<bool>` + Condvar；trigger 持锁** | FAIL | `AtomicBool` + `Mutex<()>`（`75-78`）；`trigger` 只 store+notify（`116-118`），不获取 mutex | **RES-LC-001** | **P0** |
| 锁中毒 `into_inner`，不伪装成功/panic 合同 | PARTIAL | `wait` 用 `into_inner`（`107-109`）；`trigger` 不经 mutex，毒锁语义未统一 | RES-LC-003 | P1 |
| Guard drop **不**自动 trigger | PASS | 无 `Drop` 自动 trigger | — | — |
| **本版本不提供 `Component` trait** | FAIL | `lifecycle.rs:123-129`；`lib.rs:27` re-export | **RES-API-001** | **P0** |

---

## §8 公开 API 冻结面

| Spec req | Status | Evidence path:line | Residual ID | Severity |
|----------|--------|-------------------|-------------|----------|
| 仅 re-export 冻结清单中的类型 | FAIL | `lib.rs:25-27` 额外 `Component`；error/clock 集合本身正确 | **RES-API-001** | **P0** |
| 无 prelude / 第三方 re-export / 宏 / mock | PASS | `lib.rs` 无此类项 | — | — |
| 模块仅 `clock`/`error`/`lifecycle` | PASS | `lib.rs:21-23` | — | — |

**规范冻结清单 vs 实际：**

| 项 | 规范 | 代码 |
|----|------|------|
| Clock, ClockError, MonotonicInstant, SystemClock, Timestamp | 要 | 有 |
| BoxError, ErrorKind, XError, XResult | 要 | 有 |
| ComponentState, LifecycleError, ShutdownGuard, ShutdownSignal | 要 | 有 |
| **Component** | **禁止** | **有** |

---

## §9 Serde / Wire / 持久化

| Spec req | Status | Evidence path:line | Residual ID | Severity |
|----------|--------|-------------------|-------------|----------|
| kernel 类型无 Serialize/Deserialize | PASS | 三模块均无 serde derive；Cargo 无 serde | — | — |
| 无 wire/JSON/人类时间格式 | PASS | 无相关 API | — | — |

---

## §10 Panic 与失败策略

| Spec req | Status | Evidence path:line | Residual ID | Severity |
|----------|--------|-------------------|-------------|----------|
| 时间溢出/epoch 前 → Result，不 panic | PASS | `clock.rs:100-106`；`checked_*` → Option | — | — |
| 非法 lifecycle 转换 → Result | PASS | `lifecycle.rs:53-58` | — | — |
| 反向时间差 → Option/非 panic | PARTIAL | Timestamp 正确 `None`；Monotonic 饱和 0（见 RES-CLK-002） | RES-CLK-002 | P0 |
| 锁中毒恢复策略 | PARTIAL | wait 路径合规；协议本身非 Mutex\<bool\>（RES-LC-001） | RES-LC-003 | P1 |
| kernel 不 `panic!` 报告 Invariant | PASS | 仅构造 `XError::invariant` | — | — |

---

## §15 版本与兼容性

| Spec req | Status | Evidence path:line | Residual ID | Severity |
|----------|--------|-------------------|-------------|----------|
| 当前版本 0.1.0，目标 0.1.1 | PARTIAL | `Cargo.toml:3` = `0.1.0`；尚未 bump 到 0.1.1 | RES-API-007 | P2 |
| `publish = false` 治理 | PARTIAL | `Cargo.toml` 未显式 `publish = false`（依赖 workspace 默认需再确认） | RES-API-008 | P2 |
| 描述/文档对齐 SPEC-KERNEL-002 | PARTIAL | `Cargo.toml:6` 仍写 **SPEC-KERNEL-001**；`lib.rs:15` 写「002 迁移中」 | RES-API-009 | P1 |
| 破坏性变更须 RFC+CHANGELOG 等 | PASS（流程项） | 代码侧 E1 注释与 CHANGELOG 承认迁移；非实现缺口 | — | — |

---

## §16 迁移计划（相对终态）

| Spec req | Status | Evidence path:line | Residual ID | Severity |
|----------|--------|-------------------|-------------|----------|
| **§16.1 Error**：E1 新增 ErrorKind+构造器 | PASS | `error.rs:25-46`、`137-217`；`lib.rs:15-17` | — | — |
| **§16.1** 删除 `not_found`/`other`；opaque XError | FAIL | 兼容 API 仍在；enum 未改 struct | RES-ERR-001, RES-ERR-002 | P0 |
| **§16.1** `internal` 单参 + `with_source` 模式 | PARTIAL | 两参 `internal` + `internal_msg` 并存 | RES-ERR-003 | P1 |
| **§16.2 Clock**：删 monotonic 默认实现 | FAIL | `clock.rs:90-92` | RES-CLK-001 | P0 |
| **§16.2** 反向 monotonic → None | FAIL | `clock.rs:64-66` | RES-CLK-002 | P0 |
| **§16.2** 去掉 from_nanos/as_nanos | FAIL | `clock.rs:18-30` | RES-CLK-004 | P1 |
| **§16.2** from_clock_elapsed + 调用位限制 | FAIL | 未实现 | RES-CLK-007 | P1 |
| **§16.3 Lifecycle**：Mutex\<bool\>+Condvar | FAIL | AtomicBool 协议 `lifecycle.rs:75-118` | RES-LC-001 | P0 |
| **§16.3** 移除 Component；loom；去 sleep 证明 | FAIL | Component 仍在；无 loom；测试 `sleep`（`lifecycle.rs:170`） | RES-API-001, RES-LC-004 | P0/P1 |

---

## Residual 注册表（去重）

| ID | 描述 | Sev | 主条款 |
|----|------|-----|--------|
| RES-ERR-001 | `XError` 须 opaque struct，字段私有 | P0 | §5.2 |
| RES-ERR-002 | 删除 `not_found` / `other` | P0 | §5.5 / §16.1 |
| RES-ERR-003 | `internal(context)` 单参；去掉 `internal_msg` 双轨 | P1 | §5.4 |
| RES-ERR-004 | ClockError 变体命名与 From 文案对齐 `BeforeUnixEpoch` | P2 | §5.7 / §6.5 |
| RES-CLK-001 | 删除 `Clock::monotonic` 默认实现 | P0 | §6.4 / §16.2 |
| RES-CLK-002 | `MonotonicInstant::checked_duration_since` → `Option`，禁饱和 0 | P0 | §6.3 |
| RES-CLK-003 | `SystemClock` 持 `origin`、非 Copy、显式实现 monotonic | P0 | §6.6 |
| RES-CLK-004 | 删除 `from_nanos`/`as_nanos`，仅保留 unix 命名 | P1 | §6.2 |
| RES-CLK-005 | 实现 `Timestamp::checked_sub` | P1 | §6.2 |
| RES-CLK-006 | `MonotonicInstant` 补 `Hash` | P2 | §6.3 |
| RES-CLK-007 | `from_clock_elapsed`；收紧/替换公开 `from_std` | P1 | §6.3 |
| RES-CLK-008 | `ClockError::BeforeEpoch` → `BeforeUnixEpoch`（破坏性，随 0.1.1） | P1 | §6.5 |
| RES-LC-001 | Shutdown 改为同一 `Mutex<bool>`+Condvar 协议 | P0 | §7.6 / §16.3 |
| RES-LC-002 | `#[must_use]` on Signal/Guard | P2 | §7.5 |
| RES-LC-003 | 毒锁路径与 trigger 持锁语义统一 | P1 | §7.6 / §10.2 |
| RES-LC-004 | loom + 去掉 sleep 正确性证明 | P1 | §16.3 / §11.2 |
| RES-API-001 | 删除并停止 re-export `Component` | P0 | §7.8 / §8 |
| RES-API-002 | 补 `tests/` 契约测试四文件 | P1 | §2 |
| RES-API-003 | 显式 `[features] default = []` | P2 | §3.4 |
| RES-API-004 | 接入 loom/trybuild 等 dev-deps（测试合同） | P1 | §3.3 |
| RES-API-005 | `#![deny(missing_docs)]` | P1 | §4 |
| RES-API-006 | `#![deny(unreachable_pub)]` | P1 | §4 |
| RES-API-007 | 迁移闭合后 bump `0.1.1` | P2 | §15.2 |
| RES-API-008 | 显式 `publish = false`（若 workspace 未强制） | P2 | §15.1 |
| RES-API-009 | Cargo description / 文档对齐 SPEC-KERNEL-002 | P1 | §15 / 元数据 |

---

## 计数复核

| Status | 表内行数 |
|--------|----------|
| PASS | 20 |
| PARTIAL | 14 |
| FAIL | 14 |
| **合计** | **48** |

| Severity（unique residual） | 数 |
|-----------------------------|----|
| P0 | 7（ERR×2, CLK×3, LC×1, API×1） |
| P1 | 16 |
| P2 | 5 |

> 注：部分表行共享同一 Residual ID（如 RES-CLK-003 两行），上表 residual 按 ID 去重。

---

## 结论（实现者优先级）

1. **P0 语义/安全**：`RES-LC-001`（关停协议）→ `RES-CLK-001/002/003`（时钟）→ `RES-ERR-001/002` + `RES-API-001`（错误 opaque 与 API 冻结）。
2. **P1 契约完整**：`checked_sub`、命名 API、crate attrs、`tests/`、dev 测试栈、文档元数据。
3. **P2 抛光**：`must_use`、Hash、features 字面、`publish`/`0.1.1`。

当前 `kernel` 处于 **SPEC-KERNEL-002 迁移中段（约 E1 完成）**：依赖面与模块边界大体合规，**错误形态、时钟终态、关停并发协议、公开 API 冻结** 四处尚未达到 §18.2 代码闭合。

---

*审计依据：SPEC-KERNEL-002 与上列 path:line；未执行 `cargo test`/`clippy`。*
