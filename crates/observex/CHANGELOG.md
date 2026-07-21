# Changelog — observex

## [0.1.0] - 2026-07-21

### 新增

- `TracingInstrumentation`：零字段 `Copy` 实现 `infra_contracts::Instrumentation`
- 三方法：`record_retry` / `record_circuit_open` / `record_circuit_close` → `tracing::info!`
- ADR-005 兼容别名 `ObservexInstrumentation`
- unit + 消费者侧 + tracing 字段捕获测试
- LCOV 行覆盖率 100% 门禁（`scripts/cov-gate-100.mjs`）
