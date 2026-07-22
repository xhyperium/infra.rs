# observex Round 01 Findings

> 历史版本说明：本轮执行者未负责版本更新；root 已在第 3 轮发布准备阶段完成 PATCH bump 至 `0.1.2`。

日期：2026-07-23
基线：`3cd29a942710c0fb42f3f6bc05e3c31570acad47`
范围：observex active spec、实现、测试与本仓 alignment

## 已发现

1. tracing、prefix 与 export 记录路径直接使用原始 `op`；已有 helper 未接线，且多字节小预算可超限。
2. `InMemoryExporter` 使用无界 `Vec`，没有容量、满载策略或 dropped 状态。
3. 原 `shutdown` 直接清空 pending，未累计 flushed。
4. exporter `Err` 被忽略，但同步阻塞与 panic 传播边界未文档化。
5. active/complete spec、README、API、标准、CHANGELOG 与 alignment 对 exporter 现状互相矛盾，
   并使用了没有协议证据的 OpenTelemetry 兼容表述。

## 本轮处置

- 用 `sanitize_op` 统一 trim、控制字符移除、空值回落和 128 UTF-8 字节上限。
- 所有真实 tracing/prefix/export 记录路径只下传清理后的 `op`。
- span/metric 各自使用有界容量；单次同类批次全有或全无，容量拒绝精确计入 dropped。
- 以单一 stats 快照暴露 capacity/buffered/flushed/dropped/shutdown；计数溢出通过
  `counters_saturated` 显式标记。
- shutdown 在同一 mutex 临界区执行 flush 计数、clear、close，重复调用幂等。
- 当轮明确普通 exporter `Err` 不改变记录返回；panic 传播边界已在 Round 02 改为隔离并诊断。
- 删除 OpenTelemetry 兼容声明，固定为自定义有界进程内 sink。

## 必测失败条件

- 控制字符或恶意长后缀进入 tracing/span/metric。
- 任意 `truncate_op(op, max)` 返回字节数超过 max 或破坏 UTF-8。
- 容量不足时发生部分写入，或 dropped 不等于整批长度。
- 并发后 buffered 超容量，或 accepted + capacity dropped 不守恒。
- 累计计数溢出后仍未设置 `counters_saturated`。
- shutdown 未先累计 pending、重复计数、或返回后仍接受事件。
- exporter 返回 `Err` 阻止 inner 记录或导致记录调用异常返回。
- active/complete spec 不能通过 `cmp`。

## 残余风险

- 清理不是 PII/secret 检测或 allowlist；敏感的合法字符串仍可能进入观测面。
- retry span 与 metric 是两次调用，不具备跨信号事务原子性。
- 泛型 exporter 没有阻塞、timeout 或取消隔离；panic 已在 Round 02 包装边界隔离。
- buffer 不持久化，flushed 只是进程内处置计数，不代表远端交付。
- 容量限制事件数而非总字节；直接 exporter 调用者可提供大字段。
- OpenTelemetry SDK/OTLP、完整信封与生产运维 SLO 仍为 OPEN。

## 验证证据

- `cargo fmt -p observex --check`：exit 0。
- `cargo test -p observex --all-targets`：exit 0；26 unit + 5 public API + 1 public surface
  测试通过，bench 与 examples 目标可运行。
- `cargo clippy -p observex --all-targets -- -D warnings`：exit 0。
- `cmp .agents/ssot/observex/spec/spec.md .agents/ssot/observex/spec/xhyper-observex-complete-spec.md`：exit 0。
- 当时执行的 `git diff --check -- crates/observex .agents/ssot/observex docs/ssot/observex-ssot-alignment.md`
  相对 HEAD exit 0，但不能证明相对实施 base 无空白；Round 3 使用带 base 命令重新验证。
- `cargo fmt --all --check`：最终复跑 exit 0。此前一次 exit 1 仅来自并行 writer 当时尚未格式化的
  `crates/bootstrap/**` 与 `crates/resiliencx/**`；本任务未修改这些越界路径。
