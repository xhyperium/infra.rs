# AGENTS.md — observex

> 仓库级规则见 [`../../AGENTS.md`](../../AGENTS.md)。  
> 权威规范镜像：SPEC observex · [`.agents/ssot/observex/spec/spec.md`](../../.agents/ssot/observex/spec/spec.md)

## 身份

- **L1 observability**（`publish = false`）
- package / lib：`observex` · path：`crates/infra/observex`
- 稳定公开面：`TracingInstrumentation`（+ 别名 `ObservexInstrumentation`）

## 本 crate 约束

- 生产依赖：`xhyper-kernel`（信封）、`xhyper-contracts`（lib `contracts`）、`thiserror`、`tracing`
- `default = []`；禁止 feature 泄漏
- exporter 返回的 `ExportError` 不得改变记录调用返回；同步泛型 exporter 的阻塞与 panic 边界必须诚实记录
- 只允许自定义有界进程内 sink；禁止宣称 OpenTelemetry API/SDK、OTLP 或远程持久化
- 验证：`cargo test -p observex --all-targets` · clippy · `node scripts/quality-gates/cov-gate-100.mjs -p observex --filter crates/infra/observex/src`
- 对齐：[`../../docs/ssot/observex-ssot-alignment.md`](../../docs/ssot/observex-ssot-alignment.md)

## 与 SSOT 镜像的关系

- `.agents/ssot/observex` 是本仓 active 域规格；COMPLETE 叙事 ≠ 本仓已交付
- 以本仓源码 + `cargo test` + LCOV 证据为准
