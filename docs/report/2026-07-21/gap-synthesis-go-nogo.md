# Workspace 生产就绪 — 差距综合与上线建议

> **第三遍审计** | 综合依赖链分析 + Agent 并行报告 | 2026-07-21

## Go/No-Go 建议

**当前状态：NO-GO** for production deployment. 6/21 crates ready, but the composition root (bootstrap) cannot wire any storage or exchange adapter. The minimum viable set for quant trading requires 12 crates working together — only 6 are production-ready.

**预估时间线：** 14-21 days of P0 work to reach first quant trading deployment.

---

## 差距综合矩阵

| # | 阻塞路径 | P0 缺陷 | 修复方案 | 工时 |
|---|---------|--------|----------|:--:|
| 1 | bootstrap → bounded traits | 仅含 `label()` 占位 | 端口 contracts traits 到 bounded adapters | 2d |
| 2 | bootstrap → StoreSet | 无 storage adapter 接入点 | 在 StoreSet 中添加 PG/redis/kafka 槽位 | 1d |
| 3 | configx → file source | 仅内存存储 | 添加 `FileConfigSource` + `watch` 热重载 | 2d |
| 4 | observex → OTLP | 仅 `tracing::info!` | 添加 `OtelInstrumentation` (tracing-opentelemetry) | 1d |
| 5 | transport → TLS | 无 TLS 配置 | 添加 `TlsConfig`，rustls 强制 | 1d |
| 6 | binancex/okxx → live client | 仅 server_time 有效 | 实现 HTTP order placement + WebSocket tick stream | 5d |
| 7 | taosx/ossx/clickhouse → mock | 3 个 crate 缺 mock | 添加 MockTaos、MockOss、MockClickHouse | 2d |
| 8 | taosx/ossx/clickhouse → pool | 3 个 crate 单连接 | 添加 connection pool + health check | 2d |
| 9 | natsx → JetStream | 仅 core NATS | 实现 JetStream consumer/producer | 2d |
| 10 | kafkax → offsets | 无 offset 管理 | 添加 consumer offset commit/reset | 1d |
| 11 | resiliencx → adapters | 库就绪但未被消费 | 集成到 7 个 storage + 2 个 exchange adapter | 3d |
| 12 | evidence → query | 仅 append() | 添加 read/verify API | 1d |

P0 总工时：14 days (single dev) / 10 days (2 devs)。

---

## 量化交易部署最小集

```text
Phase 1 (Week 1-2): Core infrastructure
  kernel + decimalx + canonical + contracts           ← already ready
  configx (+ file source) + observex (+ OTLP)         ← fix
  transport (+ TLS) + bootstrap (+ StoreSet wiring)    ← fix

Phase 2 (Week 2-3): Trading pipeline
  binancex (+ live order/websocket)                    ← fix
  postgresx (+ Repository trait in prod)               ← fix
  redisx (+ TLS) + kafkax (+ offsets)                   ← fix
  resiliencx (+ adapter integration)                    ← fix

Phase 3 (Week 3): Analytics & persistence
  taosx (+ pool + mock) + clickhousex (+ batch + mock)  ← fix
  natsx (+ JetStream) + evidence (+ query)               ← fix
```

**3 周后可达状态：** binancex → kafkax → postgresx → resiliencx pipeline 可用于生产交易。

---

## 孤儿问题详解

当前依赖图：

```text
kernel ←── contracts ←── {bootstrap, 14 others}
                                      └── bounded traits (占位)
                                              ↑
                                    NO STORAGE/EXCHANGE ADAPTERS WIRED
```

修复后的目标依赖图：

```text
kernel ←── contracts ←── bootstrap (StoreSet + bounded traits)
                              │
         ┌────────────────────┼────────────────────┐
         v                    v                    v
    PostgresPool          RedisPool           KafkaEventBus
   (Repository)         (KeyValueStore)        (EventBus)
```

---

## 总结

3 遍审计确认了同一结论：workspace 有明确的分层结构、良好的 trait 定义（contracts）和优秀的核心类型（kernel/decimalx/canonical）。但生产关键链路断裂——bootstrap 无法将 adapter 接入应用。补齐 bounded trait wiring 并集成 9 个关键 adapter 是下一阶段的最高优先级工作。
