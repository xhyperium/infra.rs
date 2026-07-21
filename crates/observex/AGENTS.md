# AGENTS.md — observex

> 仓库级规则见 [`../../AGENTS.md`](../../AGENTS.md)。  
> 权威规范镜像：SPEC observex · [`.agents/ssot/observex/spec/spec.md`](../../.agents/ssot/observex/spec/spec.md)

## 身份

- **L1 observability**（`publish = false`）
- package：`xhyper-observex` · lib：`observex` · path：`crates/observex`
- 稳定公开面：`TracingInstrumentation`（+ 别名 `ObservexInstrumentation`）

## 本 crate 约束

- 生产依赖：`xhyper-kernel`（信封）、`xhyper-contracts`（lib `contracts`）、`tracing`
- `default = []`；禁止 feature 泄漏
- 观测调用不得 panic、不得改变业务结果
- 禁止在本战役引入 OTEL SDK / exporter
- 验证：`cargo test -p xhyper-observex` · clippy · `node scripts/cov-gate-100.mjs -p observex --filter crates/observex/src`
- 对齐：[`../../docs/ssot/observex-ssot-alignment.md`](../../docs/ssot/observex-ssot-alignment.md)

## 与 SSOT 镜像的关系

- `.agents/ssot/observex` 是上游只读镜像；COMPLETE 叙事 ≠ 本仓已交付
- 以本仓源码 + `cargo test` + LCOV 证据为准
