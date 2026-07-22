# observex

L1 **tracing/metrics 封装**（SPEC-INFRA-OBSERVEX 0.1.2 / ADR-005）。

| 项 | 值 |
|----|-----|
| package / lib | `observex` |
| lib | `observex` |
| path | `crates/observex` |
| version | `0.1.2` |
| publish | `false` |

规范镜像：[`../../.agents/ssot/observex/spec/spec.md`](../../.agents/ssot/observex/spec/spec.md)  
对齐说明：[`../../docs/ssot/observex-ssot-alignment.md`](../../docs/ssot/observex-ssot-alignment.md)

## 公开面

```rust
use contracts::Instrumentation;
use observex::TracingInstrumentation;

let instr = TracingInstrumentation::new();
instr.record_retry("fetch", 1);
instr.record_circuit_open("fetch");
instr.record_circuit_close("fetch");
```

- 零字段 `Copy` 类型；无 subscriber 时不 panic
- 实现 `contracts::Instrumentation`
- ADR 兼容别名：`ObservexInstrumentation`
- 所有真实记录路径统一移除 `op` 控制字符，并限制为 128 个 UTF-8 字节

`sanitize_op` 仅执行资源边界清理：它不检测 PII/secret，也不校验 allowlist。调用方仍须让
`op` 来自稳定、低基数的受控业务词汇。

## 有界进程内 sink

`InMemoryExporter::new()` 对 span 和 metric 分别使用 1024 个事件槽位；
`with_capacity(n)` 可显式设置每类信号容量。单次同类批次容量不足时全批拒绝并累计 dropped；
`stats()` 返回同一锁下的一致性快照。正常表示范围内 flushed/dropped 计数精确；若发生

`usize` 溢出，字段饱和并由 `counters_saturated` 明确标记为下界。`shutdown()` 会先把待处理
事件计入 flushed，再幂等关闭。容量按事件数限制，不限制直接调用 exporter 时单个事件字段的字节数。

`ExportingInstrumentation` 同步调用 exporter。普通 `ExportError` 与可展开（unwind）的 Rust panic
均不会改变记录调用返回；`panic=abort` 不可捕获。失败调用涉及且 wrapper 不重试、交付状态未知的
事件由 `export_stats().unconfirmed_*` 公开，不宣称实际丢弃。`TelemetryExporter` 实现必须快速返回；
不得等待外部 I/O 或无界阻塞；违反合同的第三方实现仍会阻塞调用线程。

## 依赖

- 生产：`xhyper-kernel`、`xhyper-contracts`（lib `contracts`）、`thiserror`、`tracing`
- 测试：`tracing-subscriber`（字段捕获）

## 验证

```bash
cargo test -p observex --all-targets
cargo clippy -p observex --all-targets -- -D warnings
node scripts/quality-gates/cov-gate-100.mjs -p observex --filter crates/observex/src
```

## 非职责

- OpenTelemetry API/SDK、OTLP、远程导出、采样与持久化
- PII/secret 检测或 `op` allowlist
- 业务审计（`xhyper-evidence`）
- 重试/熔断策略本身（属 resiliencx）

## 生产误用红线

| 禁止 | 原因 |
|------|------|
| 宣称 OpenTelemetry / 生产可观测完成 | 仅 tracing + 自定义有界进程内 sink |
| 把 `shutdown` 当远端持久化确认 | 它只更新进程内 flushed 计数并清空内存 |
| 在 exporter 内等待外部 I/O | 同步合同要求快速返回；包装层只隔离 unwind panic，不隔离阻塞 |

示例：`cargo run -p observex --example trace_events`

## Subscriber 故障隔离（infra-s9t.17）

- `TracingInstrumentation` 仅调用 `tracing::info!`；**无**自定义 subscriber 时为 no-op，不 panic。
- 若业务安装阻塞/panic 的 subscriber，隔离责任在**安装方**（本 crate 不包裹 catch_unwind，避免掩盖错误）。
- `InMemoryExporter` 的 flush/shutdown 只是有界内存生命周期，不代表远端导出闭环。
