# taosx

Time-series store scaffold — `contracts::TimeSeriesStore`（内存）。

## 生产误用警示（infra-s9t.14）

**默认实现是进程内 scaffold/mock，不是生产客户端。**

- 禁止把 `*Adapter` 类型名当成已对接真实 Binance/Postgres/Redis/…
- 真实入口须有显式 feature（如 redisx `live`）与文档/CI 证据
- 详见 `docs/plans/artifacts/prod-consume-surface.md`
