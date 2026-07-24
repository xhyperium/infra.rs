# observex Round 02 Findings

> 历史版本说明：本轮执行者未负责版本更新；root 已在第 3 轮发布准备阶段完成 PATCH bump 至 `0.1.2`。

日期：2026-07-23
基线：`3cd29a942710c0fb42f3f6bc05e3c31570acad47`
范围：observex 覆盖率门禁与 `op` 清理顺序

## 已发现

1. `node scripts/quality-gates/cov-gate-100.mjs -p observex --filter crates/infra/observex/src`
   初次 exit 1：instrumented 806、hit 786、20 行未命中、line 97.5186%。
2. 未命中行包括 shutdown 并发测试的非确定分支、失败 exporter 的 flush/shutdown、unwind panic
   exporter 的非 panic 方法及 ops 的冗余条件分支。
3. `truncate_op` 先对原始值 trim、再过滤 control；输入 `\0  api.fetch  \0` 会留下首尾空格，
   与 active spec 的 trim 语义不一致。

## 本轮处置

- `truncate_op` 改为流式过滤 control，清理后跳过前导 whitespace、执行 `trim_end`，再按 UTF-8
  字节预算截断；仍不按输入长度分配无界临时字符串。
- 新增 `\0  api.fetch  \0 -> api.fetch` 精确回归断言。
- shutdown 并发测试改为屏障协调：每个线程确定接受一个事件，shutdown 后确定拒绝其余事件，
  保留并发与生命周期守恒语义且移除随机未命中路径。
- 新增真实 mutex poison 恢复测试，验证恢复后仍可 export、shutdown 与准确 flush 计数。
- 真实调用失败 exporter 的 flush/shutdown，以及 panic exporter 的成功 metric/flush/shutdown 边界。
- 将 ops 中逻辑上穷尽的 `else if` 收敛为 `else`，不保留不可到达的伪分支。
- `is_friendly_op` 复用 `MAX_OP_BYTES`，移除重复的长度魔法数。
- `ExportingInstrumentation` 新增原子 failed/panicked/unconfirmed 诊断，记录路径捕获 exporter 的
  unwind panic；flush/shutdown unwind panic 转为简中 `ExportError::Panicked`，`panic=abort` 不可捕获。
- `TelemetryExporter` 固定为必须快速返回且不得等待外部 I/O 的同步合同；阻塞隔离仍为 OOS。
- `ExportError` 改用 workspace `thiserror`，Cargo description 与全部 Display 文本改为简体中文。
- 新增 crate release 记录，并将实现责任边界写入 active Design。

## 防伪失败条件

- 不得添加 coverage allow、排除标记、阈值降级或不验证行为的空调用。
- poison 测试必须先真实制造 poisoned mutex，再验证恢复后的业务状态。
- exporter 错误测试必须断言公开 wrapper 返回的具体错误。
- shutdown 测试必须同时证明关闭前接受、关闭后拒绝和 flushed 数一致。
- `op` 测试必须精确断言清理后的字符串，而不只断言“不 panic”。

## 验证证据

- `cargo test -p observex --all-targets`：最终 exit 0；28 unit + 5 public API + 1 public surface
  测试通过，bench 与 examples 目标可运行。
- `cargo clippy -p observex --all-targets -- -D warnings`：最终 exit 0。
- `cargo fmt -p observex --check`：exit 0。
- `cargo fmt --all --check`：最终 exit 0。
- `cargo doc -p observex --no-deps`：exit 0。
- `cmp .agents/ssot/infra/observex/spec/spec.md .agents/ssot/infra/observex/spec/xhyper-observex-complete-spec.md`：exit 0。
- coverage gate 在新增诊断 API 前曾 exit 0：instrumented 858、hit 858、zeros 0、line 100.0000%。
  新增诊断 API 后按 root 要求停止并行覆盖率，最终串行复验由 root 执行，当前标记 NOT_RUN。
