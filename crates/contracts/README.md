# infra-contracts

Adapter and observability contracts for the infra.rs workspace.

Package：`infra-contracts` · path：`crates/contracts`  
SSOT 镜像：`.agents/ssot/contracts/`

## Modules

- `exchange` / `storage` — adapter traits
- `Instrumentation` — ADR-005 observability injection（由 `xhyper-observex` 实现）

## 硬限制

- 只放 trait / type，不放具体 adapter / 驱动实现
- 价格使用 `decimalx::Price`，禁止 `f32`/`f64` 金额
