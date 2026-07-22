# observex SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| 策略 | **B — 本仓移植 observex 最小面 + 进程内 export** |
| 日期 | 2026-07-21；**defer-close 复核 2026-07-22** |
| 规范 | `.agents/ssot/observex/spec/spec.md` |
| 当前版本 | 0.1.1（L1 TracingInstrumentation 最小面；OTEL 进程内 PASS）|
| package（`cargo -p`） | `observex` · lib `observex`（产品名别名 `xhyper-observex`，不可用于 `-p`） |
| 契约面 | `contracts` · **Instrumentation** |
| 跟进 | L3 Instrumentation 真入口（#172）；**export/flush 声明层 PASS**；**≠** 完整 OpenTelemetry SDK |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 上游/镜像 COMPLETE 叙事 | **禁止**单独当作本仓交付证明 |
| 本仓 `crates/observex` | **已落地**（`TracingInstrumentation` + tracing 三方法） |
| 本仓 Instrumentation 契约 | `contracts::Instrumentation`；L3 子集非 scaffold 入口 |
| OTEL-**compatible** 进程内导出 | **PASS**：`export.rs` · `TelemetryExporter` / `InMemoryExporter` / `ExportingInstrumentation` · `flush` / `shutdown` |
| 完整 OpenTelemetry SDK / OTLP 远端 | **OPEN（诚实边界）** — **禁止**宣称 OTEL 栈完成 |
| resiliencx / bootstrap 注入链 | **PASS**（bootstrap 默认 `TracingInstrumentation`） |
| LCOV 行覆盖率 100% | **PASS** |
| Agent L5 | **未填** |

## 本仓可观察事实

```text
crates/observex/                EXISTS
  TracingInstrumentation        impl Instrumentation
  export.rs                     TelemetryExporter / InMemoryExporter / ExportingInstrumentation
  flush / shutdown              ExportingInstrumentation::{flush,shutdown}
publish                         false
prod deps                       kernel, contracts, tracing
features.default                []
```

## 验证命令

```bash
cargo test -p contracts -p observex
cargo clippy -p contracts -p observex --all-targets -- -D warnings
cargo fmt --all --check
node scripts/quality-gates/cov-gate-100.mjs -p observex --filter crates/observex/src
```

## Clause matrix（摘要）

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| 1.1 | L1 tracing 封装，实现 Instrumentation | PASS | `impl Instrumentation` |
| 1.2 | 默认路径 tracing info | PASS | `tracing::info!` |
| 3.x | TracingInstrumentation 公开面 | PASS | unit |
| 4.1 | 实现 contracts trait | PASS | trait impl |
| 4.6 / 6.7 | exporter / flush | **PASS（进程内）** | `src/export.rs` |
| 7.3 | 不把 tracing/export 宣称为完整 OTEL SDK | PASS | README / 本文 |
| 7.4 | bootstrap 注入链 | PASS | bootstrap 默认 |

## OBJECTIVE 处置（2026-07-22 defer-close）

| 项 | 前状态 | 现状态 | 证据 |
|----|--------|--------|------|
| OTEL exporter / flush | DEFER | **PASS（in-process compatible）** | `crates/observex/src/export.rs` |
| 完整 OTEL SDK / OTLP | — | **OPEN** | 明确非目标 |

## 未做（诚实边界）

- OpenTelemetry SDK、OTLP 导出、metric 命名规范产品、采样/缓冲策略全集
- `op` 受控集合强制校验
- Agent L5 人签

## 双栏落地（2026-07-22 · STATUS 100% structure）

| 标尺 | 状态 |
|------|------|
| STATUS 结构完成度 | **100%**（layout+tests+content；非 Production Ready） |
| 声明面生产硬化 | 公共 API 集成测 + 热路径 bench + `docs/` 红线；**cov-gate-100 行覆盖** |
| 非宣称 | **禁止** workspace Production Ready / Agent L5 / 完整 OTEL 产品 |

自验证：`cargo test -p observex --all-targets`；`node scripts/quality-gates/cov-gate-100.mjs -p observex`。

## 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-22 | **defer-close**：进程内 TelemetryExporter/flush PASS；full OTEL 仍 OPEN |
