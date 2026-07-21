# Changelog — contract-testkit

## 0.1.0 — 2026-07-21

### Added

- 独立 crate：`crates/test-support/contracts`（package `contract-testkit`）
- 自 `contracts` 迁出 Fake/Recording：Tx / EventBus / KV / Repository / Instrumentation
- 新增 exchange capability Fake：MarketData / Catalog / ExecutionVenue / Account / VenueTime
- Per-trait suite：`assert_*` + `ContractFailure`（SPEC-TESTKIT-002 §3.2 / §9.6）
- `tests/suite_self_tests.rs`：Fake 驱动 suite 自测
