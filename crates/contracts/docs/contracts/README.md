# Trait 语义文档

本目录为 `contracts` 当前公开 trait 的语义合同；文档存在不等于对应后端已达到
Production Ready，真实能力仍以 adapter 测试与 live 证据为准。

| Trait | 文档 |
|-------|------|
| KeyValueStore | [key_value_store.md](./key_value_store.md) |
| TxContext | [tx_context.md](./tx_context.md) |
| TxRunner | [tx_runner.md](./tx_runner.md) |
| EventBus | [event_bus.md](./event_bus.md) |
| Instrumentation | [instrumentation.md](./instrumentation.md) |
| Repository | [repository.md](./repository.md) |
| ExecutionVenue | [execution_venue.md](./execution_venue.md) |
| MarketDataSource | [market_data_source.md](./market_data_source.md) |
| InstrumentCatalog | [instrument_catalog.md](./instrument_catalog.md) |
| AccountSource | [account_source.md](./account_source.md) |
| VenueTimeSource | [venue_time_source.md](./venue_time_source.md) |
| ObjectStore | [object_store.md](./object_store.md) |
| TimeSeriesStore | [time_series_store.md](./time_series_store.md) |
| AnalyticsSink | [analytics_sink.md](./analytics_sink.md) |
| PubSub | [pub_sub.md](./pub_sub.md) |

## 模板字段

每篇覆盖：ownership / success / failure·XError / idempotency / cancel·timeout / ordering / resource release / not-found / pagination / object-safety / fake entry / test entry。

## Venue 入口

- **生产推荐**：[`ExecutionVenue`](./execution_venue.md)（无 additive default）
- **迁移 facade**：`VenueAdapter`（`cancel_order_request` / `query_order_request` 有中文 Invalid default；树内必须 override，见 CT-10 / `tests/venue_override_gate.rs`）
