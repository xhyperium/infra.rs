> **Post-ship alignment (2026-07-14)**：API-002 **implemented**；crates.io **`xhyper-kernel` 0.1.1**；mutants missed=0；residual OPEN=0。下文部分表格可能含战役中途措辞，以 residual-open + gate.md 为 live SSOT。

# Gap Matrix v2 — SPEC-KERNEL-002 × 源码/门禁

| 字段 | 值 |
|------|-----|
| Spec | `xhyper-kernel-complete-spec.md` / `spec.md` |
| Plan | [plan.md](./plan.md) |
| Residual | [residual-open.txt](../evidence/2026-07-14/residual-open.txt) |
| Live | `cargo test -p kernel` 绿；line cov 98.82%；archgate **infra.rs 不适用（OOS）** |
| Doc-sync | Team-R10 · 与 residual-open 对齐 |
| Campaign | **L1 PASS** · L3 §18 **OPEN** |

## § 逐条对照（实现相关）

### §3 依赖

| 要求 | 现状 | 判定 |
|------|------|------|
| kernel → ∅ workspace | Cargo.toml 无 path dep | PASS |
| 生产仅 thiserror | 确认 | PASS |
| 禁 anyhow/serde/tokio/… | 无 | PASS |
| dev: loom/proptest/trybuild/static_assertions | loom(cfg)+proptest+static；trybuild **DEFER accepted** | PASS（RES-TEST-005 CLOSED DEFER） |
| features 仅 default=[] | 有 | PASS |

### §4 属性

| 要求 | 现状 | 判定 |
|------|------|------|
| forbid(unsafe_code) | lib.rs | PASS |
| deny(missing_docs, unreachable_pub) | lib.rs | PASS |
| 无生产 panic/unwrap | 抽查 lifecycle/error/clock | PASS |

### §5 error

| 要求 | 现状 | 判定 |
|------|------|------|
| ErrorKind ×9 non_exhaustive | 有 | PASS |
| opaque XError 私有字段 | 有 | PASS |
| 构造器全集 | 有 | PASS |
| is_retryable / is_bug | 仅 Transient / Invariant | PASS |
| 禁 From\<str\>/other/not_found | 无 | PASS |
| From\<ClockError\>→Unavailable | 有 | PASS |
| **仅 §5.4 方法** | `context_cow` **已删**（rg=0） | **PASS / CLOSED RES-ERR-010** |
| Display 不含 source | 有测 | PASS |

### §6 clock

| 要求 | 现状 | 判定 |
|------|------|------|
| Timestamp i64 nanos；无 Default | 有；static assert | PASS |
| checked_add/sub/since | 有 | PASS |
| MonotonicInstant 私有；reverse None | 有 | PASS |
| from_clock_elapsed doc(hidden) | 有 | PASS |
| from_clock_elapsed **const fn** | `pub const fn` | **PASS / CLOSED RES-CLK-010** |
| Clock 无 default monotonic | 有 | PASS |
| SystemClock origin；!Copy | 有 | PASS |
| ClockError 三变体名 | 有 | PASS |

### §7 lifecycle

| 要求 | 现状 | 判定 |
|------|------|------|
| ComponentState 6 + 合法转换 | 有 + matrix 测 | PASS |
| LifecycleError | 有 | PASS |
| Mutex\<bool\>+Condvar 同锁 | 有 | PASS |
| must_use Signal/Guard | 有 | PASS |
| 无 Component trait | 无 | PASS |
| loom | 资产+CI | PASS |
| poison recovery **测试** | `poison_recovery_into_inner` + 生产 into_inner | **PASS / CLOSED RES-LC-005** |
| 1000 并发回归 | `concurrent_regression_1000_cycles` | **PASS / CLOSED RES-LC-005** |
| guard drop 不触发 **测试** | `guard_drop_does_not_trigger` + !Clone/!Default | **PASS / CLOSED RES-LC-005** |

### §8 冻结导出

| 项 | 现状 | 判定 |
|----|------|------|
| 模块与 pub use 清单 | 与规范一致 | PASS |
| 方法超面 context_cow | 已删除；API 快照已去行 | **PASS / CLOSED RES-ERR-010** |

### §11 测试

| 合同 | 现状 | 判定 |
|------|------|------|
| error 单元 | 有 | PASS |
| Timestamp 边界+proptest | 有 | PASS |
| Clock contract | 有 | PASS |
| lifecycle 合法/非法矩阵 | 有 | PASS |
| multi observer / trigger-wait | 有 | PASS |
| loom | 有 | PASS |
| compile-fail trybuild | static 替代；正式 DEFER | **CLOSED (DEFER) RES-TEST-005** |
| line ≥95% | 98.82% | PASS |
| branch ≥90% | 未在 stable 测得 | OPEN RES-TEST-014 |
| mutants ≥90% | 未跑（工具缺失） | OPEN RES-TEST-015 |
| miri | 未跑（组件缺失） | OPEN RES-TEST-016 |

### §12 门禁

> **infra.rs 不适用（OOS）**：下表 KERNEL-* 为历史 monorepo 设计意图与战役闭合记录；**本仓不运行 archgate**，不维护 `.architecture/**`。本仓机控 = 结构扫描 / unit tests / CI（coverage, loom, miri, public-api）。

| 规则 | 设计意图（历史 monorepo） | 本仓判定 |
|------|---------------------------|----------|
| KERNEL-DEP-001/002 | 零 workspace dep；生产仅 thiserror | PASS（Cargo + 审查 / 结构扫描） |
| KERNEL-FEATURE-001 | features 仅 default=[] | PASS |
| KERNEL-API-001 | public-api 快照一致 | PASS（本仓 public-api / 审查，非 archgate） |
| **KERNEL-API-002** | baseline + RFC allowlist | **CLOSED (implemented)** RES-GATE-009（历史机控；本仓纪律保留） |
| KERNEL-TIME-001/002/003 | 系统时间调用边界 | 设计意图保留；archgate 机控 **OOS** |
| KERNEL-ERR-001/002 | internal 棘轮 / 窄 residual | 设计意图保留；archgate 机控 **OOS** |
| KERNEL-SERDE/ASYNC/UNSAFE | 源码禁令 | PASS（源码属性 + 测试） |
| KERNEL-LIFECYCLE-001 | loom 资产 | PASS（CI loom / tests） |

### §15–§18

| 项 | 判定 |
|----|------|
| publish=true as **xhyper-kernel**；lib name=kernel | PASS |
| version 0.1.1 + crates.io kernel | **PASS** |
| registry stable | **PASS**（crates/kernel） |
| Spec Approved | **PASS** RES-18-APPROVED CLOSED |
| §18 全勾 | **PASS**（014/015/016 human waiver DEFER） |

## 优先级队列（post W7/W8 release）

```text
CLOSED 本战役 + P3 + W6 + W7/W8:
  RES-API-007 (0.1.1) · RES-TEST-014/015/016 (human waiver DEFER)
  RES-18-APPROVED · RES-18-FULL · registry stable
OPEN 仅余: 无
```
