# observex SSOT 对齐与本仓落地状态

| 字段 | 值 |
| --- | --- |
| 策略 | 本仓最小 tracing 面 + 自定义有界进程内 sink |
| 日期 | 2026-07-23（round 03 候选准备） |
| active spec | `.agents/ssot/infra/observex/spec/spec.md` |
| 当前版本 | 0.1.2 |
| package / lib | `observex` / `observex` |
| 发布 | `false` |

## 结论

`TracingInstrumentation`、`PrefixedInstrumentation` 与 `ExportingInstrumentation` 已统一使用
有界的 `op` 清理路径；`InMemoryExporter` 已改为有界进程内 sink，并公开一致性统计与
flush-and-close 生命周期。

这只证明自定义进程内行为。它不是 OpenTelemetry API/SDK 或语义约定，不实现 OTLP、远端持久化、
异步隔离、timeout、采样、PII/secret 检测或 op allowlist。

## 实现映射

| 条款 | 状态 | 证据 |
| --- | --- | --- |
| contracts Instrumentation 三事件 | PASS | `src/lib.rs` |
| tracing 路径 op 清理 | PASS | `src/lib.rs` + `src/ops.rs` |
| prefix / export 路径 op 清理 | PASS | `src/lib.rs` + `src/export.rs` |
| 128 UTF-8 字节上限 | PASS | `MAX_OP_BYTES` / `sanitize_op` |
| PII/secret/allowlist | OPEN | 明确非本清理职责 |
| 默认与显式容量 | PASS | `DEFAULT_BUFFER_CAPACITY` / `with_capacity` |
| 单次同类批次全有或全无 | PASS | 容量检查先于 `Vec::extend` |
| 跨 span + metric 事务原子 | OPEN | 两次独立 trait 调用，不作承诺 |
| dropped / buffered / flushed / shutdown 统计 | PASS | `InMemoryExporterStats` |
| 计数溢出可见性 | PASS | `counters_saturated` 标记饱和值为下界 |
| shutdown 先 flush 计数再关闭 | PASS | 同一 mutex 临界区 |
| exporter Err 隔离 | PASS | record 忽略 export Result |
| exporter unwind panic 隔离与诊断 | PASS | `catch_unwind` + `ExportingInstrumentationStats` |
| wrapper 失败事件语义 | PASS | `unconfirmed_*`：交付未知、不重试，非实际 dropped 声明 |
| exporter abort 隔离 | OOS | `panic=abort` 不可捕获 |
| exporter 阻塞隔离 | OPEN / OOS | trait 要求非阻塞；违反合同的实现仍能阻塞调用线程 |
| OpenTelemetry SDK / OTLP | OPEN / OOS | 未引入相关依赖或协议实现 |

## 容量与生命周期语义

- span 与 metric 各自拥有独立的 `capacity_per_signal`。
- 默认每类 1024；显式容量可以为 0。
- 单次 `export_spans` 或 `export_metrics` 容量不足时整批拒绝，缓冲不变，整批计入 dropped。
- flushed/dropped 在 `usize` 范围内精确；溢出后饱和并设置 `counters_saturated`。
- 槽位容量限制事件数，不限制直接 exporter 调用中单个事件字段的字节数。
- `flush` 只把进程内 buffered 转为 flushed 计数并清空。
- 首次 `shutdown` 原子执行 flush-and-close；重复调用成功且统计不变。
- shutdown 后 export/flush 返回 `Shutdown`；关闭拒绝不重复计入容量 dropped。

## 失败边界

`ExportingInstrumentation` 先执行 inner，再同步调用 exporter。普通 `ExportError` 与 unwind panic
不会跨无返回值的记录接口传播，并累计诊断；flush/shutdown unwind panic 转为
`ExportError::Panicked`。`panic=abort` 不可捕获。泛型 exporter 必须快速返回；本轮没有 timeout
或阻塞线程隔离。

失败调用涉及的事件计入 `unconfirmed_spans/metrics`。exporter 可能在 Err/unwind 前已有部分副作用，
wrapper 无法确认交付状态且不会重试；该诊断不得解释为实际丢弃量。

## 验证

```bash
cargo fmt --all --check
cargo test -p observex --all-targets
cargo clippy -p observex --all-targets -- -D warnings
cmp .agents/ssot/infra/observex/spec/spec.md \
    .agents/ssot/infra/observex/spec/xhyper-observex-complete-spec.md
```

round 01 发现与残余风险见
`.agents/ssot/infra/observex/plan/round-01-findings.md`。

Round 2 已闭合 sanitizer、有界 exporter、错误/unwind 诊断、简体中文、`thiserror` 与 poison 恢复；
本轮新树 root 串行覆盖率为 942/942、zeros 0、100.0000%、exit 0。治理修正后候选已重冻，
本地独立 reviewer 已完成实现/证据审查，独立 verifier 已完成技术/证据初验；本次纯状态 delta
不改变受审源码/测试。GitHub 固定提交 CI artifact、PR、维护者审批、合并、tag/发布仍 pending，详见
`.agents/ssot/infra/observex/plan/round-03-findings.md`。

## 残余边界

- 没有完整事件信封、trace/span 上下文、资源属性或 schema 演进合同。
- 没有真实远端 exporter、持久化、重试、batch、timeout、backpressure worker 或运行时健康 SLO。
- `op` 清理不阻止敏感但无控制字符的值，也不把高基数字符串映射到受控词表。
- 当前 buffer 是测试/本地组合用进程内存，进程退出即丢失。
- 事件数有界不等于总字节严格有界；直接 exporter 调用者仍能构造大字段。
