# observex 实现规范

状态：active；当前 `0.1.2` 候选已重冻，本地 reviewer 完成、verifier 技术/证据初验完成；
GitHub CI/交付 pending，非 package stable

- Package / lib：`observex` / `observex`
- 路径：`crates/infra/observex`
- 当前版本：`0.1.2`
- 发布：`false`
- 机器证据：本轮新树 root 串行覆盖率 `942 / 942`、zeros 0、100.0000%、exit 0

## 0. 裁定边界

本文区分本仓已验证事实与开放能力。源码存在或本文标记 PASS 只表示对应的进程内行为已落地，
不表示 OpenTelemetry、生产可观测平台或 package stable。

## 1. 定位与非目标

observex 是 L1 tracing/metrics 封装，实现 `contracts::Instrumentation`。当前有两类实现面：

1. `TracingInstrumentation`：把 retry/circuit 事件写入 `tracing::info!`。
2. `ExportingInstrumentation<I, E>`：先调用 inner，再同步调用自定义 `TelemetryExporter`；
   `InMemoryExporter` 是该接口的有界进程内 sink。

当前数据模型不是 OpenTelemetry API/SDK 或语义约定，不实现 OTLP、远端导出、持久化、采样、
异步批处理、timeout 或资源/trace 上下文。业务审计属于 evidence；重试和熔断策略属于 resiliencx。

## 2. 依赖与 feature

| 项 | 当前事实 |
| --- | --- |
| 普通依赖 | `kernel`、`contracts`、`thiserror`、`tracing` |
| 测试依赖 | `tracing-subscriber` |
| 默认 feature | 空 |
| async runtime / OTEL SDK | 无 |

`kernel` 当前为依赖信封保留导入；记录行为由 contracts trait 与 tracing 提供。

## 3. 公开 API

| API | 当前语义 |
| --- | --- |
| `TracingInstrumentation` | 零字段 `Debug + Default + Clone + Copy`；实现 `Instrumentation` |
| `ObservexInstrumentation` | `TracingInstrumentation` 的 ADR 兼容别名 |
| `PrefixedInstrumentation<I>` | 给清理后的 op 增加受限前缀，再下传 |
| `CountingInstrumentation` | 进程内测试计数；不是生产 metrics |
| `sanitize_op` / `MAX_OP_BYTES` | 统一清理，最大 128 UTF-8 字节 |
| `TelemetryExporter` | 同步自定义 exporter trait |
| `InMemoryExporter` | 有界进程内 sink |
| `ExportingInstrumentation<I, E>` | 同步调用 inner 与 exporter |
| `InMemoryExporterStats` | 同一锁下的容量、buffered、flushed、dropped、shutdown 快照 |
| `ExportingInstrumentationStats` | exporter failed/panicked/unconfirmed 原子诊断快照 |

## 4. op 治理

所有真实记录路径必须使用同一 `sanitize_op` 结果：

- trim；
- 移除 Unicode control 字符；
- 结果为空时回落 `"_"`；
- 超过 128 UTF-8 字节时，在字符边界截断并追加 `~`。

该清理仅限制资源占用与控制字符注入。它不检测 PII、secret，也不验证 allowlist；调用方仍必须
让 `op` 来自稳定、低基数且不含敏感值的受控词汇。不得把本清理宣称为脱敏闭环。

## 5. 缓冲、满载与统计

- `InMemoryExporter::new/default` 对 span 与 metric 分别提供 1024 个槽位。
- `with_capacity(n)` 设置每类信号各自独立的容量；`0` 拒绝所有非空批次。
- 单次 `export_spans` 或 `export_metrics` 容量不足时整批拒绝，原缓冲不变，返回
  `ExportError::BufferFull`，并按整批事件数累计对应 dropped。
- span 与 metric 是独立调用，跨信号不提供事务原子性。
- `stats()` 在同一 mutex 临界区读取一致性状态；各兼容访问器返回相同状态的单项投影。
- flushed/dropped 在 `usize` 表示范围内精确；溢出时字段饱和并设置 `counters_saturated`，
  此后对应值只能解释为下界。
- dropped 只统计容量拒绝；shutdown 拒绝返回 `Shutdown`，不重复计入容量 dropped。
- 容量按事件数计算，不限制直接 exporter 调用中单个事件字段的字节数。

## 6. flush、shutdown 与失败边界

- `flush()` 把当前 buffered 数累计到 flushed 并清空；它只表示进程内处置，不表示持久化。
- 首次 `shutdown()` 在同一 mutex 临界区执行 flush-and-close；重复调用返回成功且统计不变。
- shutdown 返回后 export/flush 返回 `ExportError::Shutdown`。
- `ExportingInstrumentation` 先调用 inner；exporter 返回的 `ExportError` 与 unwind panic 均被内化，
  不改变记录调用的返回，并累计 failed/panicked/unconfirmed 诊断；`panic=abort` 不可捕获。
- `unconfirmed_spans/metrics` 只表示失败调用涉及、交付状态未知且 wrapper 不重试；exporter 可能
  在返回 Err 或 unwind 前已有部分副作用，不得把 unconfirmed 宣称为实际 dropped。
- `flush` / `shutdown` 的普通 exporter 错误原样返回，unwind panic 转为 `ExportError::Panicked`。
- `TelemetryExporter` 是同步非阻塞接口。实现必须快速返回，不得等待外部 I/O 或无界阻塞；
  违反合同的第三方实现仍会阻塞调用线程。
- 诊断计数原子饱和更新，并以短临界区提供多字段一致快照；`counters_saturated` 表示数值只能作为下界。
- `InMemoryExporter` 从 poisoned mutex 恢复内部状态；这是本实现行为，不扩展到泛型 exporter。

## 7. 并发不变量

`InMemoryExporter` 用单一 mutex 线性化 export、flush 和 shutdown。任意时刻每类 buffered
不超过容量；单事件并发容量测试中，accepted buffered/flushed 与 capacity dropped 必须守恒。
shutdown 与 export 竞态由持锁顺序决定，shutdown 返回后不得再成功接受事件。

## 8. 测试与验收

最低验证：

```bash
cargo fmt --all --check
cargo test -p observex --all-targets
cargo clippy -p observex --all-targets -- -D warnings
cmp .agents/ssot/infra/observex/spec/spec.md \
    .agents/ssot/infra/observex/spec/xhyper-observex-complete-spec.md
```

测试必须覆盖：

- 控制字符、空值、恶意长 op 与 UTF-8 1/2/边界字节预算；
- 默认/显式/零容量、恰满、超限整批拒绝、flush 后复用与 dropped 精确计数；
- 并发容量守恒；
- shutdown 自动 flush 计数、幂等及关闭后拒绝；
- exporter `Err` / unwind panic 不改变 inner 记录和正常返回，并产生诊断；
- flush/shutdown unwind panic 转换及简中错误 Display。

## 9. 开放项

OpenTelemetry SDK/OTLP、远端持久化、异步队列、timeout、采样、PII/secret 检测、op allowlist、
完整事件信封与服务级运维 SLO 均为 OPEN，不得由当前进程内测试推断为已交付。

## 10. 追溯

| 合同 | 证据 |
| --- | --- |
| Instrumentation 签名 | `crates/contracts/src/lib.rs` |
| op 清理 | `crates/infra/observex/src/ops.rs` |
| tracing 记录 | `crates/infra/observex/src/lib.rs` |
| 有界 sink 与生命周期 | `crates/infra/observex/src/export.rs` |
| 公开/失败/并发测试 | `crates/infra/observex/src/*.rs`、`crates/infra/observex/tests/` |
| 本仓裁定 | `docs/ssot/observex-ssot-alignment.md` |
