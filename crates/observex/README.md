# observex

L1 **tracing/metrics 封装**（SPEC-INFRA-OBSERVEX 0.1.0 / ADR-005）。

| 项 | 值 |
|----|-----|
| package | `xhyper-observex` |
| lib | `observex` |
| path | `crates/observex` |
| version | `0.1.0` |
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

## 依赖

- 生产：`xhyper-kernel`（信封）、`xhyper-contracts`（lib `contracts` · Instrumentation）、`tracing`
- 测试：`tracing-subscriber`（字段捕获）

## 验证

```bash
cargo test -p xhyper-observex
cargo clippy -p observex --all-targets -- -D warnings
node scripts/cov-gate-100.mjs -p observex --filter crates/observex/src
```

## 非职责

- OpenTelemetry exporter / flush / shutdown / 采样 / 缓冲
- 业务审计（`xhyper-evidence`）
- 重试/熔断策略本身（属 resiliencx）

## 生产误用红线

| 禁止 | 原因 |
|------|------|
| 宣称 OTEL / 生产可观测完成 | 仅 `tracing::info!` 三方法 |
| 依赖本 crate 做 flush/shutdown | API 不存在（DEFER） |

示例：`cargo run -p observex --example trace_events`

