# Changelog — observex

## [0.1.0] - 2026-07-21

### 新增

- `TracingInstrumentation`：零字段 `Copy` 实现 `contracts::Instrumentation`
- 三方法：`record_retry` / `record_circuit_open` / `record_circuit_close` → `tracing::info!`
- ADR-005 兼容别名 `ObservexInstrumentation`
- unit + 消费者侧 + tracing 字段捕获测试
- LCOV 行覆盖率 100% 门禁（`scripts/cov-gate-100.mjs`）

## [Unreleased]

当前无新增条目。

## [0.1.2] — 2026-07-23

> 本节记录工作树中的 Cargo 版本与候选变更，不代表已创建 tag 或完成外部发布。

### 新增

- 真实 `benches/hot_path`
- 公开 API 集成覆盖扩展
- `docs/API.md`
- `InMemoryExporter::with_capacity`、`stats`、dropped 统计与 `counters_saturated` 溢出状态
- 容量、恶意 `op`、并发、exporter 错误和 shutdown 数据处置测试

### 变更

- 所有真实记录路径统一移除 `op` 控制字符，并按 UTF-8 字节边界限制为 128 字节
- `InMemoryExporter` 改为 span/metric 各自有界；满载时单次同类批次全有或全无
- `shutdown` 在同一临界区先完成进程内 flush 计数再幂等关闭
- 文档明确该能力只是有界进程内 sink，不是 OpenTelemetry API/SDK 或 OTLP
- `ExportingInstrumentation` 隔离 exporter 的 unwind panic，并公开 failed/panicked/unconfirmed 诊断
- `ExportError` 改用 `thiserror`，所有用户可见错误为简体中文
- `TelemetryExporter` 固定为必须快速返回的同步非阻塞合同
