> **Note:** Agent audit mid-migration; see residual-open.txt for current open items.

# R-test / gate / downstream — SPEC-KERNEL-002 §11–§12 与 XError/Clock/Component 下游影响

| 字段 | 值 |
|------|-----|
| 审计类型 | 只读（read-only） |
| SSOT | `/home/workspace/xhyper.rs/.agent/SSOT/kernel/spec/spec.md`（SPEC-KERNEL-002） |
| 聚焦 | §11 测试合同 · §12 CI/机器门禁 · §16 迁移残余 · 下游 XError/Clock/Component |
| 代码根 | `/home/workspace/xhyper.rs/crates/kernel` |
| 日期 | 2026-07-14 |
| 结论 | **未闭合**。单元测试与 line coverage CI 部分达标；loom / proptest / trybuild / branch·mutation·Miri / 具名 KERNEL-* 规则 / public-api 快照 / E3 与 lifecycle 终态均未达标 |

---

## 0. 总览状态

| 域 | 相对 SPEC-KERNEL-002 | 状态 |
|----|----------------------|------|
| §11.1 单元测试 | 内联 `#[cfg(test)]` 有基础覆盖；多项必测缺口 | **部分** |
| §11.2 loom | 无 dev-dep、无测试、无 CI | **缺口** `RES-TEST-001` |
| §11.3 proptest | 无 | **缺口** `RES-TEST-002` |
| §11.4 trybuild / compile-fail | 无 `tests/`、无 trybuild | **缺口** `RES-TEST-003` |
| §11.5 覆盖率 | CI 强制 kernel **line ≥95%**；**无 branch ≥90%** | **部分** `RES-TEST-004` |
| §11.6 mutation | 本地/just 可选；**非 CI 硬门禁** | **缺口** `RES-TEST-005` |
| §11.7 Miri | 无 kernel 定期/nightly 接线证据 | **缺口** `RES-TEST-006` |
| §12.1 必过命令 | fmt/clippy/test/llvm-cov-lines/archgate/lint-deps/crate-standard 有；**semver-checks 未硬接线** | **部分** `RES-GATE-001` |
| §12.2 KERNEL-* 具名规则 | archgate 有 R7 子集；**无 KERNEL-ID 目录、无 loom/API 快照/ERR 棘轮等** | **缺口** `RES-GATE-002` |
| §12.3 public API 快照 | 无 `.architecture/api/kernel-public-api.txt` | **缺口** `RES-GATE-003` |
| XError not_found/other | 生产路径清零；**API 仍保留（E1）** | **残余** `RES-DOWN-001` |
| Component trait | 仍公开；仅 kernel 内 Dummy 实现 | **残余** `RES-DOWN-002` |
| Clock::monotonic 默认实现 | 仍 `Instant::now` 默认；ManualClock **不覆盖** | **残余** `RES-DOWN-003` |
| Timestamp::from_nanos 调用点 | 少量测试/固定钟；生产极少 | **可管** `RES-DOWN-004` |
| Shutdown 并发协议 | AtomicBool + Condvar；trigger **不持同一 mutex** | **残余** `RES-DOWN-005` |

---

## 1. `tests/` 目录与 dev-deps

### 1.1 物理布局

```text
crates/kernel/
  Cargo.toml          # 无 [dev-dependencies]
  src/
    lib.rs
    error.rs          # 内联 #[cfg(test)]
    clock.rs          # 内联 #[cfg(test)]
    lifecycle.rs      # 内联 #[cfg(test)]
  # 不存在 tests/
  # 不存在 tests/*.rs trybuild / loom / proptest 入口
```

**事实**：`crates/kernel` **没有** 集成/compile-fail/loom 用的 `tests/` 目录。所有现有测试均为模块内 `#[cfg(test)]`。

### 1.2 `Cargo.toml` 依赖面

| 段 | 内容 | 对照 SPEC |
|----|------|-----------|
| `[dependencies]` | 仅 `thiserror` | §3.2 生产外部依赖 ✅ |
| `[dev-dependencies]` | **缺失** | §3.3 允许/要求 loom、proptest、trybuild、static_assertions ❌ |
| `[features]` | 无（含无 `default = []` 显式段） | §3.4 无 feature ✅（`default=[]` 显式清单未写，非阻断） |
| mock feature | 已移除；ManualClock 在 `testkit` | R5 DONE ✅ |

**残余**：`RES-TEST-001` / `RES-TEST-002` / `RES-TEST-003` 的前置条件是补齐 dev-deps 与（可选）`tests/` 布局。

---

## 2. loom / proptest / trybuild

| 机制 | SPEC 要求 | 仓库现状 | ID |
|------|-----------|----------|-----|
| **loom** | §11.2：waiter/park 竞争、多 waiter、post-trigger 观察、无 lost wake-up、无永久阻塞；通过前不得宣称 ShutdownSignal 并发正确。§12.2 `KERNEL-LIFECYCLE-001` | `crates/kernel` 无 loom dep；全仓无 kernel loom 测试；lifecycle 单测用 `thread::sleep` 作「阻塞」证明（与 §6.7 / §11.1 / §16.3 冲突） | `RES-TEST-001` |
| **proptest** | §11.3：任意 i64 Timestamp×Duration checked 运算；ComponentState 全矩阵；XError 构造器分类一致 | kernel 无 proptest；他处（如 decimalx）有 proptest，**不覆盖 kernel** | `RES-TEST-002` |
| **trybuild** | §11.4：`Timestamp`/`MonotonicInstant` !Default；`ShutdownGuard` !Clone；无 serde；**不导出 Component**；私有字段不可达 | 无 trybuild、无 compile-fail 用例 | `RES-TEST-003` |

**实现侧风险（支撑 loom 必要性）**：

`ShutdownInner` 同时使用 `AtomicBool` + `Mutex<()>` + `Condvar`；`trigger` **先 store AtomicBool 再 `notify_all`，不获取与 `wait` 相同的 mutex**（`lifecycle.rs`）。这正是 SPEC §7.6 禁止的模式之一，且 §16.3 明确「AtomicBool + Condvar 存在 lost wake-up 风险」。

---

## 3. §11.1 单元测试覆盖对照

### 3.1 error（`error.rs` 内联测试）

| 必测项 | 现状 | 判定 |
|--------|------|------|
| 每个 ErrorKind 构造器 | `new_kind_constructors` + `all_constructors_display_and_flags` 覆盖 9 类 | ✅ 基本 |
| kind / context / retry_after | 有 | ✅ |
| with_source 不改变 kind | `with_source_preserves_each_variant` | ✅ |
| is_retryable 仅 Transient | 多处 assert | ✅ |
| is_bug 仅 Invariant | 多处 assert | ✅ |
| Display 不包含 source 细节 | `internal("ctx", io)` → `"internal: ctx"` | ✅ 抽样 |
| Internal 使用点不通过模糊构造器新增 | **无机器棘轮**（属 gate） | ❌ `RES-GATE-002` 子集 |
| not_found / other 兼容映射 | 有单测证明映射 | ⚠️ 兼容层仍在，非终态 |

### 3.2 Timestamp / Clock

| 必测项 | 现状 | 判定 |
|--------|------|------|
| i64::MIN / MAX 边界 | 仅 `i64::MAX`+add 溢出；**无 i64::MIN 专测** | ⚠️ 部分 |
| checked_add / checked_sub 溢出 | 有 add；**无独立 checked_sub API**（仅 `checked_duration_since`） | ⚠️ 与 API 面一致 |
| 相等差 Some(ZERO)；earlier>self → None | 有 | ✅ |
| 大于 u64 纳秒边界 | 未显式覆盖 | ⚠️ |
| 无 Default 编译失败 | **无 trybuild** | ❌ `RES-TEST-003` |
| SystemClock::now 可表示 | `system_clock_returns_positive` | ✅ 弱（仅 >0） |
| monotonic 非递减 | 仅 duration_since 前进；**无 SystemClock 连续 monotonic 非递减序列** | ⚠️ |
| **Clock trait 无 monotonic 默认实现** | **代码仍有默认实现**（`clock.rs:89-92`） | ❌ 合同与实现双失败 `RES-DOWN-003` |
| 时间错误 → XError::Unavailable | `clock_error_maps_to_unavailable` | ✅ |
| ManualClock 独立控制 wall/**monotonic** | testkit 仅 AtomicI64 墙钟；**不 override monotonic** | ❌ `RES-DOWN-003` |

### 3.3 lifecycle

| 必测项 | 现状 | 判定 |
|--------|------|------|
| 全部合法/非法转换 | 抽样 2 合法 + 2 非法；**非全矩阵** | ⚠️ → proptest 矩阵 `RES-TEST-002` |
| trigger-before-wait / wait-before-trigger | 多观察者路径偏 wait-then-trigger；**无独立 trigger-before-wait 用例** | ⚠️ |
| 多 observer；trigger 后新 observer | 双 signal clone；**无 trigger 后新建 signal 可见性专测** | ⚠️ |
| 1000 次并发回归 | **无** | ❌ |
| poison recovery | **无** | ❌ |
| guard !Clone / signal Clone | 无 compile-fail；运行时仅 clone signal | ⚠️ / ❌ trybuild |
| guard drop 不触发 | **无** | ❌ |
| Component::drain | Dummy 仅测 Running→Draining | ⚠️ 且 trait 本身应移除 `RES-DOWN-002` |

**睡眠证明**：`shutdown_multi_observer` 使用 `thread::sleep(40ms)` 断言「触发前未完成」。SPEC：`std::thread::sleep` 不得用于证明时钟正确性；§16.3 亦要求删除 sleep 作为并发正确性证明 → `RES-TEST-001`。

---

## 4. archgate 与 KERNEL-* 规则

### 4.1 SPEC §12.2 具名规则 vs 实现

| 规则 ID | 意图 | archgate / CI 现状 |
|---------|------|-------------------|
| KERNEL-DEP-001 | 内部 workspace dep = 0 | 由 workspace/lint-deps + 无 path dep 间接保证；**无 KERNEL- 标签** |
| KERNEL-DEP-002 | 生产外部仅 thiserror | archgate R7：`kernel_external_deps` 白名单，**失败计入 exit** ✅ 语义 |
| KERNEL-FEATURE-001 | 除 default=[] 外无 feature | **无显式检查**（当前 Cargo.toml 碰巧无 feature） |
| KERNEL-API-001 | 公开 API = 冻结清单 | **无**；且现导出 `Component` 已超出 §8 冻结面 |
| KERNEL-API-002 | 新增公开项需 Approved RFC | **无** |
| KERNEL-TIME-001 | kernel 外生产 `SystemTime::now` → fail | archgate 扫 `SystemTime::now` / `Utc::now`（排除 kernel、tools）；allowlist 分类 ⚠️ 部分（tools 整树豁免） |
| KERNEL-TIME-002 | kernel 外 `Instant::now` → fail | **未扫描 Instant::now** ❌ |
| KERNEL-TIME-003 | `Timestamp::from_unix_nanos` 调用白名单 | **无** |
| KERNEL-ERR-001 | `XError::internal` 使用点 ≤ 基线 | **无棘轮** |
| KERNEL-ERR-002 | 字符串匹配决定错误分类 → fail | **无** |
| KERNEL-SERDE-001 | kernel 类型 Serialize/Deserialize → fail | 间接：forbidden tokens 含 serde（use/pub 行）；**非类型级证明** |
| KERNEL-ASYNC-001 | tokio/async-std → fail | forbidden tokens 含 tokio（use/pub）⚠️ 非签名级 |
| KERNEL-UNSAFE-001 | unsafe = 0 | `#![forbid(unsafe_code)]` 源码级；archgate 未计 unsafe 数 |
| KERNEL-LIFECYCLE-001 | loom test 未过 → fail | **完全缺失** |

### 4.2 archgate 已实现（R7 子集，CHANGELOG/main.rs）

- `kernel_external_deps`：`[dependencies]` 仅允许 `thiserror`
- `kernel_forbidden_tokens`：kernel 源码 use/pub 行禁 `anyhow`/`serde`/`tokio`/`chrono`/`tracing`
- `kernel_public_api_leaks`：`crates/kernel/` 路径下 public_api_leaks **计入失败**（与普通 public_api_leaks「只报不阻断」不同）
- 时间调用：`SystemTime::now` / `Utc::now` + exceptions allowlist

**结论**：存在 **R7 实用子集**，但 **没有** SPEC 要求的 `KERNEL-*` 规则目录/ID 映射与完整强制面 → `RES-GATE-002`。

### 4.3 Public API 快照 §12.3

- 期望：`.architecture/api/kernel-public-api.txt`（rustdoc JSON / cargo-public-api 优先）
- 现状：`.architecture/` 下仅有 `workspace.toml`、`exceptions.toml`、`migration.toml`、`policies/*` —— **无 `api/` 快照**
- 冻结清单（§8）与 `lib.rs` **不一致**：代码仍 `pub use … Component`

→ `RES-GATE-003`、`RES-DOWN-002`

---

## 5. CI 覆盖率与 §12.1 命令矩阵

### 5.1 覆盖率（`.github/workflows/ci.yml` `coverage` job）

| 项 | SPEC | CI |
|----|------|-----|
| line ≥ 95%（kernel） | §11.5 / §12.1 | `cargo llvm-cov report -p kernel --fail-under-lines 95` ✅ |
| branch ≥ 90% | §11.5 | **未强制** ❌ `RES-TEST-004` |
| core 包 80% | 仓库策略 | market_data/ledger/risk_engine ✅（非 kernel 合同） |
| mutation ≥ 90% | §11.6 | **非 CI**；review 记 `just mutants-kernel` 可选 ❌ `RES-TEST-005` |
| Miri | §11.7 | 无 kernel job ❌ `RES-TEST-006` |

历史证据：`CHANGELOG` 称本地 line ≥95% 且 49 mutants missed=0；**不得**把历史本地结果当作当前 CI 合同闭合。

### 5.2 §12.1 必过命令对照

| 命令 | CI / 工具链 |
|------|-------------|
| `cargo fmt -- --check` | `fmt` job ✅ |
| `cargo clippy -p kernel --all-targets -- -D warnings` | workspace clippy job（需确认含 kernel；通常 `--all-targets` workspace）✅ 方向 |
| `cargo test -p kernel --all-features` | build-test / nextest 路径含 workspace；包级等价 ✅ |
| `cargo llvm-cov -p kernel --fail-under-lines 95` | coverage job ✅（**无** branch fail-under） |
| `cargo run -p archgate -- --json` | `architecture-drift` ✅ |
| `cargo run -p xtask -- lint-deps` | `lint-deps` ✅ |
| `cargo run -p xtask -- crate-standard --check` | `lint-deps` job ✅ |
| `cargo semver-checks check-release -p kernel` | **未在 ci.yml 硬接线**；证据目录记工具缺失/无 baseline（G8）→ `RES-GATE-001` |

---

## 6. ManualClock

| 项 | 事实 |
|----|------|
| 位置 | `/home/workspace/xhyper.rs/crates/testkit/src/lib.rs`（R5 迁出 ✅） |
| 状态 | 单一 `AtomicI64` 墙钟纳秒 |
| `Clock::now` | `Timestamp::from_nanos(load)` |
| `Clock::monotonic` | **未实现** → 落入 trait **默认** `Instant::now` |
| 测试 | `manual_clock_set_and_advance`、Send/Sync bounds |
| SPEC 终态 | §11.1 / §16.2：可**独立**控制 wall 与 monotonic；删除 monotonic 默认实现 |

**下游影响**：凡只实现 `now()` 的 `FixedClock` / `ManualClock` 在调用 `monotonic()` 时都会读真实单调钟，**测试不可复现间隔**，也无法注入回拨。

→ `RES-DOWN-003`

---

## 7. `XError::not_found` / `other` 残余

### 7.1 API 仍在（E1 兼容）

`crates/kernel/src/error.rs`：

- `XError::not_found` → `invalid`（注释：**E2 删除**，但代码仍 pub）
- `XError::other` → `internal("other", source)`（注释：**E2 删除**）
- 两参 `internal` / `internal_msg` 仍为兼容面；E3 目标 opaque struct

### 7.2 调用点扫描（workspace `*.rs`）

| 模式 | 命中 |
|------|------|
| 生产路径 `XError::not_found` / `::other` | **0**（CHANGELOG E2：生产清零） |
| kernel 内单测 | `xerror_not_found_maps_to_invalid_display`、`other_maps_to_internal` |

**状态**：E2「调用点清零」✅；E3「删除兼容 API + opaque」❌ → `RES-DOWN-001`

### 7.3 下游错误构造习惯（抽样）

生产侧已普遍使用 `missing` / `invalid` / `transient` / `unavailable` / `internal`（如 oss/nats/resiliencx/gate/transport）。**删除 not_found/other 对生产路径预期零改动**；仅需删 kernel 兼容 API + 更新单测 + API 快照。

---

## 8. `Component` 使用面

| 位置 | 用途 |
|------|------|
| `kernel::lifecycle::Component` trait | `state` + `drain` |
| `lib.rs` pub use | **公开导出**（违反 §7.8 / §8 / §18.2） |
| `lifecycle` 测试 `Dummy` | **唯一** `impl Component` |
| bootstrap / marketd / adapters | **不使用** `Component`；仅用 `ShutdownGuard`/`ShutdownSignal`/`ComponentState`（若有） |

`ComponentState` 状态机被定义且有限单测；生产 composition（`bootstrap`、`apps/marketd`）消费的是 **关停原语**，不是 `Component` trait。

→ 移除 trait 的下游破环面 ≈ **仅 kernel 自身测试** → `RES-DOWN-002`（低迁移成本、高规范收益）

---

## 9. monotonic 默认实现的消费者

| 实现 | `monotonic` | 说明 |
|------|-------------|------|
| `SystemClock` | 默认实现（自身未 override） | 生产默认；且 **SystemClock 为 `Copy`**，与 §6.6「禁止 Copy / 每次 monotonic 新 origin」设计冲突更甚：当前 `SystemClock` 无 `origin: Instant` 字段，每次默认 `Instant::now()` 而非固定 origin 的 elapsed |
| `testkit::ManualClock` | 默认 | 不可注入 |
| binance/okx/gate 测试 `FixedClock` | 仅 `now()` | 默认 monotonic |
| 生产 `Arc<SystemClock>`（binance/okx/gate） | 默认 | 真实时间 |

**SPEC 终态**（§6 / §16.2）：

1. 删除 `Clock::monotonic` 默认体 → 所有 `impl Clock` 必须显式实现；
2. `SystemClock` 持有 `origin: Instant`，`monotonic()` = `origin.elapsed()`；
3. ManualClock 独立 monotonic 状态。

当前默认实现消费者 = **所有未 override 的 Clock impl**（上表全部）→ `RES-DOWN-003`。

**额外实现偏差（时钟）**：

- SPEC：`SystemClock` 非 `Copy`，含 `origin`；代码：`#[derive(Copy, Default)] struct SystemClock;` 单元结构。
- SPEC：`MonotonicInstant` 反向差语义讨论为 `None`（§16.2）；代码：`saturating_duration_since` → 0。

---

## 10. `Timestamp::from_nanos` / `from_unix_nanos` 调用点

| 文件 | 上下文 |
|------|--------|
| `crates/kernel/src/clock.rs` | 定义 + `SystemClock::now` + 单测 |
| `crates/testkit/src/lib.rs` | ManualClock::now |
| `crates/gate/src/lib.rs` | 测试 FixedClock |
| `crates/adapters/exchange/binance/src/rest.rs` | 测试 FixedClock |
| `crates/adapters/exchange/okx/src/rest.rs` | 测试 FixedClock |

**生产路径**：除 `SystemClock::now` 构造外，**无**业务代码直接 `Timestamp::from_nanos`。  
`from_unix_nanos` 几乎仅 kernel 别名/单测。

**门禁缺口**：`KERNEL-TIME-003` 应对 `from_unix_nanos`（及可能 `from_nanos`）做允许清单；当前 **零强制** → `RES-DOWN-004` / `RES-GATE-002`。

风险级别：**低**（调用点少、多为测试固定钟）；迁移成本低。

---

## 11. 其他下游：Shutdown / Clock 生产消费

| 消费方 | 用法 | 备注 |
|--------|------|------|
| `infra/bootstrap` | `ShutdownSignal::new`，注入 `AppContext` | 文档禁止 async 线程 `wait` |
| `apps/marketd` composition | 配对 guard/signal；显式 `trigger` | R8 DONE |
| `infra/gate` | `SystemClock` / 可注入 `Clock` | FixedClock 测试 |
| exchange binance/okx | `Arc<SystemClock>` 默认 | 墙钟用于签名时间等 |
| `resiliencx` | `XError::is_retryable` 门控重试 | 与 ErrorKind 对齐 |

关停路径已有真实生产消费者 → **lifecycle 并发正确性（loom）是硬前置**，不能仅靠 sleep 单测宣称正确 → `RES-TEST-001` + `RES-DOWN-005`。

---

## 12. Residual 登记

### RES-TEST-*（测试合同）

| ID | 描述 | 优先级 | 建议动作 |
|----|------|--------|----------|
| **RES-TEST-001** | 无 loom；Shutdown 并发未模型检验；单测依赖 sleep | P0 | dev-dep loom；§11.2 场景；CI/`KERNEL-LIFECYCLE-001`；删 sleep 正确性证明 |
| **RES-TEST-002** | 无 proptest（Timestamp/Duration、状态矩阵、XError 分类） | P1 | dev-dep proptest + 属性测 |
| **RES-TEST-003** | 无 trybuild/`tests/` compile-fail | P1 | trybuild：!Default、!Clone(Guard)、无 Component 导出、无 serde |
| **RES-TEST-004** | CI 无 branch coverage ≥90% | P1 | llvm-cov branch fail-under 或等价 |
| **RES-TEST-005** | mutation 非 CI 硬门禁 | P2 | PR 命中 kernel 时 mutants；nightly 全量 |
| **RES-TEST-006** | 无 Miri 定期执行 | P2 | nightly `cargo miri test -p kernel` |
| **RES-TEST-007** | lifecycle 单测缺口：全矩阵、1000 并发、poison、guard drop、trigger 后新 observer | P1 | 补单测 + 与 loom 分工 |
| **RES-TEST-008** | Timestamp 边界（MIN、>u64 nanos）与 Clock「无默认 monotonic」合同测缺失 | P1 | 随 API 终态补齐 |

### RES-GATE-*（CI / archgate）

| ID | 描述 | 优先级 | 建议动作 |
|----|------|--------|----------|
| **RES-GATE-001** | `cargo semver-checks -p kernel` 未硬接线 / 无 baseline | P1 | pin 工具 + tag baseline 或明确 SKIP≠PASS 政策 |
| **RES-GATE-002** | KERNEL-* 规则未机器化完整集（TIME-002 Instant、TIME-003、ERR-001/002、API-001/002、FEATURE-001、LIFECYCLE-001、…） | P0 | archgate 扩展或独立 kernel-gate；规则 ID 与 exit 绑定 |
| **RES-GATE-003** | 缺 `.architecture/api/kernel-public-api.txt` | P0 | cargo-public-api/rustdoc-json 生成并 CI diff |
| **RES-GATE-004** | 普通 `public_api_leaks` 仍「只报不阻断」（非 kernel 路径）；与 fail-closed 政策张力 | P2 | 安全相关 finding 纳入 exit（既有 audit 已指出） |

### RES-DOWN-*（XError / Clock / Component 下游）

| ID | 描述 | 优先级 | 下游影响 |
|----|------|--------|----------|
| **RES-DOWN-001** | `not_found`/`other` 仍为 pub API（E3 未做） | P1 | 生产调用点已 0；删除时仅改 kernel 测与快照 |
| **RES-DOWN-002** | 仍导出 `Component` trait；无生产 impl | P1 | 删除 trait：仅 kernel Dummy 测；更新 §8 快照与 trybuild |
| **RES-DOWN-003** | `monotonic` 默认实现 + ManualClock/FixedClock 不可注入单调钟；SystemClock 无 origin / 为 Copy | P0 | 所有 `impl Clock` 需显式 `monotonic`；testkit 双状态；binance/okx/gate FixedClock 补实现；SystemClock 字段化 |
| **RES-DOWN-004** | `from_nanos` 调用点无白名单门禁 | P2 | 调用点少；加 KERNEL-TIME-003 即可 |
| **RES-DOWN-005** | Shutdown 实现协议偏离 §7.6（AtomicBool + 无锁 notify） | P0 | 改 `Mutex<bool>+Condvar`；bootstrap/marketd API 形状可不变，**语义需 loom 回归** |
| **RES-DOWN-006** | SPEC Status 仍 `Proposed`；Cargo.toml/README 仍混用 SPEC-KERNEL-001 字样 | P2 | 批准 SPEC 后统一引用；registry stable 与代码终态绑定 §15/§18 |

---

## 13. 与历史 residual（R1–R8）关系

| 旧 ID | 主题 | 与本审计 |
|-------|------|----------|
| R1 package rename | DONE | 不重复 |
| R2–R3 XError taxonomy | DONE 基线九类 | **E3 / not_found 删除** 升级为 `RES-DOWN-001` |
| R5 ManualClock→testkit | DONE 迁址 | **monotonic 独立控制** 未做 → `RES-DOWN-003` |
| R6 coverage 95% line | DONE CI | branch/mutation/Miri 仍开 → `RES-TEST-004/005/006` |
| R7 archgate 机器化 | DONE 子集 | 完整 KERNEL-* → `RES-GATE-002` |
| R8 lifecycle 被 composition 消费 | DONE | 并发正确性未证 → `RES-TEST-001`/`RES-DOWN-005` |

本文件 ID 前缀 **RES-TEST / RES-GATE / RES-DOWN**，避免与 2026-07-13 R1–R8 混淆。

---

## 14. 优先闭合顺序（建议）

```text
1. RES-DOWN-005 + RES-TEST-001  — Shutdown 协议修正 + loom（正确性根）
2. RES-DOWN-003                 — 删 monotonic 默认；SystemClock origin；ManualClock 双钟
3. RES-GATE-003 + RES-DOWN-002  — API 快照 + 移除 Component + 对齐 §8
4. RES-DOWN-001                 — E3：删 not_found/other，opaque XError（若 RFC 批准）
5. RES-TEST-002/003/007/008     — proptest / trybuild / 单测补洞
6. RES-GATE-002/001             — KERNEL-* 全量 + semver-checks
7. RES-TEST-004/005/006         — branch / mutants CI / Miri
```

---

## 15. 证据索引（只读路径）

| 证据 | 路径 |
|------|------|
| SSOT §11–§12、§16–§18 | `.agent/SSOT/kernel/spec/spec.md` |
| kernel 实现 | `crates/kernel/src/{lib,error,clock,lifecycle}.rs` |
| kernel 清单 | `crates/kernel/Cargo.toml` |
| ManualClock | `crates/testkit/src/lib.rs` |
| archgate R7 | `tools/archgate/src/main.rs`、`tools/archgate/CHANGELOG.md` |
| CI coverage / archgate | `.github/workflows/ci.yml` |
| 历史 residual | `.agent/SSOT/kernel/review/review.md` |
| 过期 test evidence（仍写 manual 在 kernel） | `.agent/SSOT/kernel/test/test.md`（**滞后**，不可作当前 SSOT） |
| E2 清零声明 | `crates/kernel/CHANGELOG.md` |

---

## 16. 一句话裁决

**测试门禁与下游迁移均未达到 SPEC-KERNEL-002 完成定义（§18）**：line coverage 与 R7 依赖白名单是少数已机器化的亮点；**loom/并发协议、monotonic 默认真时钟、Component 冻结面违规、KERNEL-* 全量门禁与 API 快照**构成当前最大未闭合面。在 `RES-TEST-001` 与 `RES-DOWN-005` 关闭前，不得宣称 `ShutdownSignal` 并发正确或 lifecycle 语义 stable。
