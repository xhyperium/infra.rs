# bootstrap 第 2 轮审计与加固记录

> 历史版本说明：本轮版本由 root 统一处理；root 已在第 3 轮发布准备阶段完成 PATCH bump 至 `0.3.2`。

| 字段 | 值 |
|---|---|
| 日期 | 2026-07-23 |
| Base commit | `3cd29a942710c0fb42f3f6bc05e3c31570acad47` |
| 前置证据 | [round-01-findings.md](./round-01-findings.md) |
| 范围 | `crates/bootstrap/**`、`.agents/ssot/bootstrap/**`、`docs/ssot/bootstrap-ssot-alignment.md` |
| 版本策略 | root 负责版本；本执行者不改 package version 或 `Cargo.lock` |

## Reviewer P1

第 1 轮把 shutdown owner 移入 `AppContext`，但兼容 API `take_shutdown_guard` 与
`BootstrappedApp::into_parts` 可形成 ownerless context。该 context 在 signal 尚未触发时
调用 `graceful_shutdown` 会静默跳过 trigger、继续 drain 并返回成功，违反
signal-before-drain 合同。

覆盖率门禁同时揭示两个未执行的真实失败映射：

- `AsyncDrain::register` 在 mutex poison 时返回 `XError::Internal`；
- `Bootstrap::register_drain` 过去把该错误丢失并误映射为配置非法。

## 修复

- `AppContext::graceful_shutdown` 与 `BootstrappedApp::graceful_shutdown` 收敛为
  `Result<Vec<DrainStepResult>, BootstrapError>`。
- graceful 先消费本地 guard，再以 `ShutdownSignal::is_triggered()` 为线性化检查点：
  - signal 已由本地或外部 owner 触发：允许 drain；
  - ownerless 且 signal 未触发：返回
    `MissingDependency { name: "shutdown_guard" }`，不取快照、不执行 hook。
- graceful 消费 `self`；Missing 后 context 与未执行 hook 被丢弃，不能重试。
- `trigger_shutdown` 继续保持 trigger-only/no-op ownerless 兼容语义；`run_drain`
  继续作为显式 drain-only 逃生面。
- drain 注册锁中毒在低层保留 `Internal`；builder 映射为
  `DependencyUnavailable { name: "drain", source }`，不伪装为配置非法。

## 真实测试增量

- `take_shutdown_guard` 后 graceful 精确返回 `Missing(shutdown_guard)`，hook 零执行。
- 外部 guard 预先触发后，同形 ownerless context 可正常 drain。
- `into_parts` 后未触发 controller 时 fail-closed；另一个实例先触发 controller 后
  graceful 成功。
- ownerless `BootstrappedApp` 委托返回同一 Missing，hook 零执行。
- 真实毒化 drain mutex，分别验证低层 `Internal` 与 builder 层
  `DependencyUnavailable` + source 保留。
- 正常四条 build、signal-before-drain、LIFO、失败后继续、trigger-only 与并发快照
  测试继续保留。

## 验证证据

| 命令 | 退出码 | 结果 |
|---|---:|---|
| `cargo fmt -p bootstrap` | 0 | package 源码已格式化 |
| `cargo test -p bootstrap --all-targets` | 0 | 44 单元 + 9 public_api + 4 public_api_surface，共 57 tests；bench/example target 通过 |
| `cargo clippy -p bootstrap --all-targets -- -D warnings` | 0 | 首次因测试 `err().expect()` 退出 101；修复后全绿 |
| `cargo doc -p bootstrap --no-deps` | 0 | rustdoc 生成成功 |
| `cargo llvm-cov -p bootstrap --all-targets --fail-under-lines 100 --summary-only` | 1 | 首次实现后 99.45%；报告定位测试自身未执行 panic/closure 行，已改为真实可执行 helper/断言 |
| 最终 llvm-cov 串行复验 | NOT_RUN | root 指示停止并行 coverage，待所有 writer 冻结后由 root 串行复验 |

未使用 coverage 排除、伪造调用或弱化阈值。最终 `cmp`、fmt check、diff check、
manifest/version 检查与冻结哈希见执行者交付报告。

## 已知边界

- ownerless 检查与并发外部 trigger 在线性化点竞争；只有检查前完成的 trigger 可使
  本次 graceful 成功。signal 单调不复位，因此成功检查后不会回退。
- Missing 返回消费 context，不能稍后触发再重试；调用方应先触发外部 controller。
- `run_drain` 明确绕过 signal 前置条件，仅供调用方有意选择 drain-only 行为。
- 同步 hook 仍无 timeout、取消或 panic 隔离，可永久阻塞。
- 本轮证据不支持 package stable、生产关停 SLA 或完整异步生命周期声明。
