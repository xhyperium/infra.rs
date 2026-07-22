# Changelog — contract-testkit

## 0.1.2 — 2026-07-22

## 0.1.2 — 2026-07-23

### Added

- ObjectStore / TimeSeriesStore / AnalyticsSink / PubSub 四个 portable core suite
- EventBus portable surface 与 snapshot/replay Fake profile 分层
- OSS / TAOS / ClickHouse / Redis PubSub 真实 adapter ignored target 调用入口
- 标准七项布局：`docs/` · `benches/` · `examples/` · `review/` · `releases/`
- `examples/basic.rs`：Fake KV + Tx suite 最小消费者路径
- `benches/hot_path`：RecordingInstrumentation + FakeKeyValueStore
- ObjectStore、TimeSeriesStore、AnalyticsSink、PubSub suite 与确定性 `FixtureNamespace`。
- 14 trait / 15 case broken implementation 负测矩阵；精确断言 `ContractFailure.contract/case`。
- `contract-testkit` 公开 API baseline 与 test-support production graph gate。

### Changed

- EventBus / PubSub 明确降为操作 smoke，不声明交付、重放、顺序或次数。
- AnalyticsSink / Instrumentation 增加 observer-aware suite，不改变 contracts trait。

## 0.1.0 — 2026-07-21

### Added

- 独立 crate：`crates/test-support/contracts`（package `contract-testkit`）
- 自 `contracts` 迁出 Fake/Recording：Tx / EventBus / KV / Repository / Instrumentation
- 新增 exchange capability Fake：MarketData / Catalog / ExecutionVenue / Account / VenueTime
- Per-trait suite：`assert_*` + `ContractFailure`（SPEC-TESTKIT-002 §3.2 / §9.6）
- `tests/suite_self_tests.rs`：Fake 驱动 suite 自测
