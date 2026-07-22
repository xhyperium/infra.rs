# Test — SPEC-KERNEL-002

| 字段 | 当前值 |
|------|--------|
| Status | Active contract；本轮结果待 fresh evidence |
| 变更前 baseline / 当前版本 | `kernel 0.3.0` / `kernel 0.3.1` |
| Source Spec | `SPEC-KERNEL-002` |
| Production certification | 未声明 |

本文件定义必须执行的测试合同，不维护历史 PASS 台账。每项结果以当前 commit 的命令输出为准。

## 1. 必测场景

### 1.1 `ClockDomain` 与进程 origin

- 两个独立 `SystemClock` 返回 `ClockDomain::PROCESS`；
- 两实例的单调点可比较，后采样不早于先采样；
- `from_clock_elapsed` 默认产生进程 domain；
- `from_clock_elapsed_in` 保留显式 domain；
- 同 domain 正向差正确，反向差为 `None`；
- 跨 domain 的 `partial_cmp` 与 `checked_duration_since` 均为 `None`；
- raw domain 不被当作跨进程身份或持久化协议。

### 1.2 `wait_timeout`

目标签名：

```rust
fn wait_timeout(
    &self,
    timeout: Duration,
) -> Result<bool, WaitTimeoutError>;
```

必须覆盖：

| 场景 | 精确结果 |
|------|----------|
| 已触发后以可表示时长调用 | `Ok(true)`，立即返回 |
| 未触发，零时长 | `Ok(false)` |
| 未触发，常规时长到期 | `Ok(false)` |
| deadline 前由其他线程触发 | `Ok(true)` |
| `Duration::MAX` 导致 deadline 不可表示 | `Err(WaitTimeoutError::DeadlineOverflow)` |
| 伪唤醒或重复检查 | 不重置原 deadline，不误报触发 |

`Duration::MAX` 测试必须匹配 typed error。仅验证“不 panic”或 `false` 不合格，因为那会允许 overflow 继续伪装普通 timeout。

### 1.3 核心关停并发

必须覆盖 trigger-before-wait、wait-before-trigger、多 observer、trigger 后新 observer、1000 次回归、poison recovery、signal 可 Clone、guard 不可 Clone 及 guard drop 不触发。

loom 覆盖 waiter 检查与 park 的竞争、多个 waiter、trigger 后观察及无 lost wake-up。`wait_timeout` 在 `cfg(loom)` 下不存在，不要求 loom 模拟。

### 1.4 其余既有合同

- 九种 `ErrorKind` 构造与查询、source chain、retry/bug 判定；
- `Timestamp` 完整 `i64` 边界、checked 算术与 property test；
- `ComponentState` 全部二元转换；
- `ClockError -> XError::Unavailable`；
- `Timestamp`/`MonotonicInstant` 无 Default，guard 无 Clone，公开类型无 serde。

## 2. Doctest 与 API

doctest 必须单独运行，验证所有 rustdoc `compile_fail` 消费面。`cargo test --all-targets` 不能替代 doctest。

公开 API 测试与基线必须包含：

- `ClockDomain`、`PROCESS`、`from_raw`、`as_raw`；
- `MonotonicInstant::domain` 与两个 `#[doc(hidden)]` 构造 seam；
- `WaitTimeoutError::DeadlineOverflow`；
- `ShutdownSignal::wait_timeout -> Result<bool, WaitTimeoutError>`；
- crate 根对 `WaitTimeoutError` 的重导出。

## 3. 验证命令

```bash
cargo fmt --all -- --check
cargo clippy -p kernel --all-targets -- -D warnings
cargo test -p kernel --all-targets
cargo test -p kernel --doc
RUSTFLAGS='--cfg loom' cargo test -p kernel --test lifecycle_concurrency_loom --release
node scripts/quality-gates/check-public-api.mjs
node scripts/quality-gates/check-workspace-deps.mjs
```

若当前 CI 对 kernel 接线 coverage、Miri 或 mutation，则按 CI 配置执行并保存结果；未执行不得记录 PASS。

## 4. Evidence 规则

结果必须绑定当前 commit、工具链、完整命令和原始输出。失败、SKIP 与重跑必须如实记录。

`evidence/2026-07-14/` 只解释历史，不是本轮 fresh evidence。历史 branch、Miri、mutants 或 coverage PASS 不可迁移为当前状态。

## 5. 验收

只有上述场景、doctest、loom 和 API 合同全部通过，gate 才可把本轮 kernel 变更标为 PASS。任何 typed error 被压成普通 timeout 均为阻断项。
