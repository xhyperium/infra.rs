# Changelog

## [Unreleased]

### Added

- Adapter contracts: `ExchangeAdapter` / storage traits / `AdapterState` / `Error`
- `Ticker` 价格字段使用 `decimalx::Price`（禁止 f32/f64 金额）
- `Instrumentation` trait (ADR-005) for observex injection
