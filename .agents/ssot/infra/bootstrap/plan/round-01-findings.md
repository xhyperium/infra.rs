# bootstrap 第 1 轮审计与加固记录

> 历史版本说明：本轮执行者按范围保留 `0.3.1`；root 已在第 3 轮发布准备阶段完成 PATCH bump 至 `0.3.2`。

> 后续裁定见 [round-02-findings.md](./round-02-findings.md)：第 1 轮的 ownerless
> graceful 成功语义存在缺口，已在第 2 轮改为 fail-closed。本文件保留为历史证据，
> 不再作为当前 API 合同。

| 字段 | 值 |
|---|---|
| 日期 | 2026-07-23 |
| Base commit | `3cd29a942710c0fb42f3f6bc05e3c31570acad47` |
| Worktree | `.worktrees/feat/infra-2d9.9-infra-core-domains` |
| 范围 | `crates/infra/bootstrap/**`、`.agents/ssot/infra/bootstrap/**`、`docs/ssot/bootstrap-ssot-alignment.md` |
| 版本策略 | 本轮明确禁止版本变更；package 保持 `bootstrap` v0.3.1 |

## 发现

1. `Bootstrap::build` / `try_build` 构造 `AppContext` 时未搬运唯一
   `ShutdownGuard`。guard drop 不会触发 signal，因此普通 build 产物只能观察、
   不能触发关停。
2. `ShutdownController::trigger` 与 `AppContext::run_drain` 是两条分离路径，
   没有固定 signal-before-drain 的组合入口，也没有返回完整步骤结果的应用级 API。
3. `AsyncDrain` 实际执行同步 `FnOnce` hook，没有 deadline、取消或 panic 隔离；
   标准文档却宣称具备 timeout / cancellation safety。
4. `register` 与 drain 快照已有 mutex 线性化实现，但文档和测试未定义：快照后注册
   留给下一批，hook 锁外执行，不保证跨批全局 LIFO 或串行。
5. active spec 虽与 complete spec `cmp` 一致，但两者共同过期：记录了不存在的
   dev-dependencies / `tests/e2e.rs`，错误描述 build 校验和测试库存，并使用了不符合
   Cargo manifest 的 package 别名。

## 修复

- `AppContext` 现在持有唯一 `ShutdownController`；四条成功 build 路径都保留
  可触发关停所有权。
- 新增消费式 `AppContext::{trigger_shutdown, graceful_shutdown}` 与
  `BootstrappedApp::graceful_shutdown`。graceful 路径先触发 signal，再 drain，
  返回本批所有正常返回 hook 的 `DrainStepResult`；单步 `Err` 不短路。
- 保持 `BootstrappedApp::trigger_shutdown` 的 trigger-only 兼容语义；
  `into_parts` 仍返回 `(AppContext, ShutdownController)`，但明确把唯一 guard 移交给
  controller，拆出的 context 只保留 drain 能力。
- 保持 `AsyncDrain` 锁内取快照、锁外执行；补充确定性并发测试和精确合同。
- README、API、标准、CHANGELOG、示例、rustdoc、active/complete spec 与对齐矩阵
  已按 Cargo、源码和测试事实更新。

## 测试增量

- 普通 `build` / `try_build` 产物可实际触发 signal。
- AppContext graceful shutdown 的 signal-before-drain、三步 LIFO、单步失败后继续、
  完整结果返回。
- BootstrappedApp graceful 委托与旧 trigger-only 不运行 hook。
- `into_parts` 唯一所有权转移后，controller 触发、context 继续 drain。
- `take_shutdown_guard` 后不凭空重建 guard，但 context 仍可 drain。
- hook 执行期间注册的 late hook 不进入当前快照，只进入下一次 drain。

## 验证证据

| 命令 | 退出码 | 结果 |
|---|---:|---|
| `cargo fmt -p bootstrap` | 0 | package 源码已格式化 |
| `cargo test -p bootstrap --all-targets` | 0 | 42 单元 + 9 public_api + 3 public_api_surface，共 54 个测试通过；bench/example target 通过 |
| `cargo clippy -p bootstrap --all-targets -- -D warnings` | 0 | 无 warning |

最终静态检查（fmt check、spec `cmp`、diff whitespace 与范围）以交付回报中的命令和
退出码为准。

## 已知边界与失败条件

- 同步 hook 可永久阻塞；本 crate 不提供 timeout 或取消。
- hook panic 不隔离，会中断当前 drain，未执行快照项随栈展开丢弃。
- 不同 drain 批次可并发执行；只保证每次快照内 LIFO，不保证跨批全局顺序。
- `take_shutdown_guard` / `into_parts` 是显式所有权转移；转移后的 context 不能自行
  触发 signal。
- 若定向 fmt/test/clippy 失败且三次修复不收敛、双 spec `cmp` 不一致，或出现授权
  路径外改动，则停止并报告；禁止破坏性回滚。
- 本轮证据只支持进程内合同，不支持 package stable、生产关停 SLA、完整异步生命周期
  或交易栈端到端闭合声明。
