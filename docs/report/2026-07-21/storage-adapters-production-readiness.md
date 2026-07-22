# Storage Adapters Production Readiness Report

> **Date:** 2026-07-21 | **Auditor:** infra.rs CI | **Scope:** crates/adapters/storage/{redis,kafka,nats,postgres,taos,oss,clickhouse}

## 1. Executive Summary

7 storage adapter crates evaluated against production readiness criteria defined in §2.
Total implementation: 6,866 LOC across 7 crates. All 7 have production implementations (not scaffold),
but production readiness varies significantly.

| Crate | LOC | Files | Mock | Pool | Config | Test Files | Production Ready? |
|-------|----:|:----:|:----:|:----:|:------:|:---:|:--:|
| **postgresx** | 1,693 | 9 | ✅ | ✅ | ✅ | 1 | **Yes** |
| **redisx** | 1,509 | 7 | ✅ | ✅ | ✅ | 2 | **Near-Prod** |
| **kafkax** | 1,041 | 10 | ✅ | ✅ | ✅ | 1 | **Near-Prod** |
| **taosx** | 755 | 4 | ❌ | ❌ | ✅ | 1 | **Partial** |
| **ossx** | 693 | 5 | ❌ | ❌ | ✅ | 1 | **Partial** |
| **natsx** | 666 | 6 | ✅ | ✅ | ✅ | 1 | **Near-Prod** |
| **clickhousex** | 509 | 4 | ❌ | ❌ | ✅ | 1 | **Partial** |

**Overall:** 1 Production-Capable, 3 Near-Prod, 3 Partial. No crate is production-stable across all criteria.

---

## 2. Production Readiness Criteria

A crate is considered **Production Ready** when it meets ALL of:

| # | Criterion | Definition |
|---|-----------|------------|
| P1 | **Trait Implementation** | Implements the corresponding `contracts` trait with all methods |
| P2 | **Error Handling** | All errors are typed (`thiserror`), no bare `unwrap()`/`expect()` |
| P3 | **Configuration** | Environment-variable-driven config via `FOUNDATIONX_*` prefix |
| P4 | **Connection Pool** | Production-grade connection pooling with health checks |
| P5 | **Mock/Fake** | In-memory mock implementation for offline testing |
| P6 | **Integration Tests** | `tests/` directory with real-service connectivity tests |
| P7 | **Documentation** | `docs/README.md` with API docs, config reference, migration guide |
| P8 | **TLS/SSL** | Support for encrypted connections (not default-only) |
| P9 | **Retry/Resilience** | Integration with `resiliencx` for retry/circuit-breaking |
| P10 | **CHANGELOG** | Versioned release history following Keep a Changelog |

---

## 3. Per-Crate Analysis

### 3.1 postgresx (1,269 LOC) — Production Capable 🟢

**Contract:** `Repository<T, Id>` + `TxContext` + `TxRunner`

| File | LOC | Purpose |
|------|----:|---------|
| `config.rs` | 344 | Connection pool config, auth, SSL |
| `pool.rs` | 214 | Deadpool-managed connection pool with health check |
| `mock.rs` | 216 | In-memory `MockRepo` for unit testing |
| `error.rs` | 142 | Typed errors (thiserror) |
| `tx.rs` | 112 | `TxContext` + `TxRunner` implementations |
| `adapter.rs` | 91 | `Repository` trait impl |
| `runner.rs` | 75 | SQL runner abstraction |
| `conn.rs` | 39 | Connection wrapper |
| `lib.rs` | 36 | Public API exports |

**Note:** Production `Repository<T,Id>` trait is implemented on scaffold types only (`PostgresAdapter`, `ObservingPostgresAdapter`). The production path (`PostgresPool`) provides direct SQL execution without the Repository abstraction. This is a deliberate design choice (low-level pool for performance) but leaves the contracts trait unimplemented in production code.

**Readiness:** P1⚠️ P2✅ P3✅ P4✅ P5✅ P6✅ P7✅ P8⚠️ P9❌ P10✅

| Gap | Severity | Action |
|-----|----------|--------|
| Repository trait not in production | Medium | Either implement Repository on PostgresPool or document this as intentional |
| Missing TLS enforcement | Medium | SSL mode must be `require` or `verify-full` in production config |
| No retry integration | Medium | Wrap pool operations with resiliencx retry policy |
| Only 1 integration test | Low | Add transaction boundary tests, connection failure tests |

### 3.2 redisx (1,509 LOC) — Near Production 🟡🟢

**Contract:** `KeyValueStore` + `PubSub`

| File | LOC | Purpose |
|------|----:|---------|
| `config.rs` | 469 | Redis cluster/sentinel config (`RedisConfig`, `RedisConfigBuilder`, `RedisMode`) |
| `pool.rs` | 280 | `RedisPool` — `ConnectionManager` + Semaphore backpressure |
| `scaffold.rs` | 236 | `RedisAdapter` / `InMemoryRedis` + `MockRedisAdapter` (TTL simulation) |
| `client.rs` | 223 | **Production** `RedisClient` — implements `KeyValueStore` + extended API |
| `pubsub.rs` | 163 | `RedisPubSub`, `RedisPubSubFacade` (feature `pubsub`) |
| `error_map.rs` | 96 | Redis error → `kernel::ErrorKind` mapping |
| `lib.rs` | 42 | Module root, re-exports |

**Readiness:** P1✅ P2✅ P3✅ P4✅ P5✅ P6✅ P7✅ P8⚠️ P9❌ P10✅

| Gap | Severity | Action |
|-----|----------|--------|
| Scaffold module leaks into production | Medium | Gate `scaffold.rs` behind feature flag (already exists, need cleanup) |
| No retry integration | Medium | Add circuit breaker for Redis connection failures |
| No TLS config in production config builder | Medium | Add SSL/TLS options for Redis 6+ encrypted connections |
| PubSub behind feature gate | Low | Consider making pubsub default-enabled for quant trading use |

### 3.3 kafkax (806 LOC) — Near Production 🟡🟢

**Contract:** `EventBus`

| File | LOC | Purpose |
|------|----:|---------|
| `pool.rs` | 192 | Producer/consumer pool |
| `config.rs` | 143 | SASL/SSL broker config |
| `mock.rs` | 82 | In-memory mock bus |
| `consumer.rs` | 78 | Consumer group management |
| `bus.rs` | 70 | Event bus trait glue |
| `adapter.rs` | 63 | Trait implementation |
| `error_map.rs` | 44 | Error translation |
| _other_ | 134 | Message types, producer, lib |

**Readiness:** P1✅ P2✅ P3✅ P4✅ P5✅ P6✅ P7⚠️ P8⚠️ P9❌ P10✅

| Gap | Severity | Action |
|-----|----------|--------|
| No consumer offset management | High | Must implement offset commit/reset for at-least-once delivery |
| No retry integration | Medium | Add exponential backoff on broker connection failure |
| Missing SASL_SSL enforcement | Medium | Current config prefers PLAINTEXT; must enforce SASL_SSL in prod |
| Mock doesn't simulate partitioning | Low | Add partition-awareness to MockEventBus |

### 3.4 taosx (755 LOC) — Partial 🟡

**Contract:** `TimeSeriesStore`

| File | LOC | Purpose |
|------|----:|---------|
| `client.rs` | 468 | **Production** `TaosPool` — implements `TimeSeriesStore`, REST client (port 6041), stable management, precision auto-detection |
| `config.rs` | 186 | `TaosConfig`, `TsPrecision` |
| `adapter.rs` | 82 | `TaosAdapter` (scaffold) — in-memory `TimeSeriesStore` |
| `lib.rs` | 19 | Minimal root |

**Deepest production client** among all 7 adapters — 468 LOC in client.rs alone. Handles stable auto-creation, precision auto-detection, multi-table batch INSERT, and table-not-found graceful recovery.

**Readiness:** P1✅ P2❌ P3✅ P4❌ P5❌ P6✅ P7✅ P8❌ P9❌ P10✅

| Gap | Severity | Action |
|-----|----------|--------|
| **No mock** | **High** | Must provide `MockTaos` for testability |
| **No pool** | **High** | Add connection pooling (native TDengine connection is expensive) |
| **No error typing** | **High** | Replace raw result types with `thiserror` typed errors |
| Large monolithic client.rs (392 LOC) | Medium | Split into submodules: connection, query, write |
| No TLS support | Low | Add TLS config for TDengine 3.x encrypted mode |

**Quant trading note:** TDengine is a critical time-series store for market data (trades, order books, klines). Missing connection pool makes high-frequency ingestion impossible in production.

### 3.5 ossx (550 LOC) — Partial 🟡

**Contract:** `ObjectStore`

| File | LOC | Purpose |
|------|----:|---------|
| `client.rs` | 260 | Aliyun OSS HTTP client |
| `config.rs` | 151 | Bucket/region/endpoint config |
| `sign.rs` | 69 | Signature generation (HMAC-SHA1) |
| `adapter.rs` | 56 | Adapter trait impl |
| `lib.rs` | 14 | Exports |

**Readiness:** P1✅ P2❌ P3✅ P4❌ P5❌ P6✅ P7⚠️ P8✅ P9❌ P10✅

| Gap | Severity | Action |
|-----|----------|--------|
| **No mock** | **High** | Must provide `MockOSS` for testability |
| **No retry** | **High** | HTTP client without retry — OSS is prone to transient failures |
| No upload progress tracking | Medium | Large file uploads need progress/checksum verification |
| Hardcoded HMAC-SHA1 signing | Medium | Extract to pluggable signer for other OSS providers (AWS S3, GCS) |
| Monolithic client.rs | Low | Split into operations: get, put, delete, list |

**Quant trading note:** OSS stores historical market data snapshots, model checkpoints, and audit artifacts. Without retry, large uploads will fail on transient network errors.

### 3.6 natsx (499 LOC) — Near Production 🟡🟢

**Contract:** `EventBus` + `PubSub`

| File | LOC | Purpose |
|------|----:|---------|
| `pool.rs` | 200 | NATS connection pool |
| `config.rs` | 108 | Server URL/auth config |
| `mock.rs` | 70 | In-process mock NATS |
| `adapter.rs` | 63 | Adapter impl |
| `bus.rs` | 43 | Bus glue |
| `lib.rs` | 15 | Exports |

**Readiness:** P1✅ P2✅ P3✅ P4✅ P5✅ P6✅ P7⚠️ P8❌ P9❌ P10✅

| Gap | Severity | Action |
|-----|----------|--------|
| **No TLS** | **High** | Must add TLS config for NATS 2.x encrypted connections |
| **No JetStream support** | **High** | Core NATS is at-most-once; quant trading needs JetStream for persistence |
| No retry integration | Medium | Add reconnection with backoff for NATS cluster failover |
| Mock doesn't simulate latency | Low | Add configurable delay to MockNats |

**Quant trading note:** NATS is the message backbone between trading services. Without JetStream (persistent streams), in-flight orders and market data can be lost on restart. This is a blocking gap for production.

### 3.7 clickhousex (400 LOC) — Partial 🟡

**Contract:** `AnalyticsSink`

| File | LOC | Purpose |
|------|----:|---------|
| `client.rs` | 246 | HTTP client |
| `config.rs` | 94 | Connection/auth config |
| `adapter.rs` | 51 | Adapter impl |
| `lib.rs` | 9 | Exports |

**Readiness:** P1✅ P2❌ P3✅ P4❌ P5❌ P6✅ P7⚠️ P8✅ P9❌ P10✅

| Gap | Severity | Action |
|-----|----------|--------|
| **No mock** | **High** | Must provide `MockClickHouse` for testability |
| **No batch insert** | **High** | Row-by-row insert is too slow; must add batch/chunked writes |
| No connection pool | Medium | HTTP connection reuse via keep-alive pool |
| No retry | Medium | Add retry for ClickHouse HTTP errors |

**Quant trading note:** ClickHouse is the OLAP layer for trade analytics, backtesting results, and audit logs. Batch inserts are mandatory — single-row inserts at quant trading volumes will overload the cluster.

---

## 4. Contract Trait → Adapter Mapping

| contracts Trait | Adapter | Implementation Status |
|-----------------|---------|----------------------|
| `KeyValueStore` | redisx | ✅ Basic get/set implemented |
| `EventBus` | kafkax | ✅ Publish/subscribe with mock |
| `EventBus` | natsx | ✅ Publish/subscribe with mock |
| `Repository<T, Id>` | postgresx | ✅ Full CRUD with mock |
| `TxContext` / `TxRunner` | postgresx | ✅ Transaction support |
| `TimeSeriesStore` | taosx | ⚠️ Client exists, trait methods incomplete |
| `ObjectStore` | ossx | ✅ Get/put operations |
| `AnalyticsSink` | clickhousex | ✅ Write via HTTP client |
| `PubSub` | natsx / redisx | ⚠️ Redis pubsub partial, NATS pubsub via EventBus |

---

## 5. Quant Trading Application Context

### 5.1 Market Data Pipeline

```
Exchange → kafkax/natsx (EventBus) → taosx (TimeSeriesStore) → postgresx (Repository)
                                                                    ↓
                          redisx (KeyValueStore ← cache)      clickhousex (AnalyticsSink)
```

| Flow | Adapter | Current | Required |
|------|---------|---------|----------|
| Tick → Ingest | kafkax | ✅ Publish only | Subscribe with offset management |
| Tick → Store | taosx | ⚠️ Single connection | Pooled connections + batch writes |
| Tick → Cache | redisx | ✅ Get/Set | PubSub for real-time push |
| Tick → Query | postgresx | ✅ Repository | Transaction boundaries |
| Tick → Analytics | clickhousex | ⚠️ Single insert | Batch inserts with chunking |

### 5.2 Critical Gaps for Trading

| Gap | Impact | Adapters Affected |
|-----|--------|-------------------|
| **No connection pooling** | Cannot handle > 100 conn/s | taosx, ossx, clickhousex |
| **No mock implementations** | Untestable offline | redisx, taosx, ossx, clickhousex |
| **No retry/circuit breaker** | Transient failures cascade | All 7 adapters |
| **NATS no JetStream** | In-flight data loss on restart | natsx |
| **No batch write** | Single-row bottlenecks | clickhousex, taosx |
| **No TLS enforcement** | Credentials exposed in dev | postgresx, redisx, kafkax, natsx |

---

## 6. Recommended Priority Matrix

### P0 — Blocking Production (Fix before any production deployment)

1. **Add mocks** to taosx, ossx, clickhousex (3 crates — redisx already has MockRedisAdapter)
2. **Add connection pools** to taosx, ossx, clickhousex (3 crates)
3. **Add NATS JetStream** support to natsx
4. **Add batch inserts** to clickhousex and taosx

### P1 — Production Hardening (Required for stable production)

5. **Integrate resiliencx retry** across all 7 adapters
6. **TLS enforcement** in postgresx, redisx, kafkax, natsx
7. **Consumer offset management** in kafkax
8. **Error typing** in taosx, ossx, clickhousex
9. **Repository trait in production** for postgresx (or document as intentional gap)

### P2 — Production Polish (Desirable for maintainability)

9. Expand test coverage to > 3 integration tests per adapter
10. Separate scaffold modules from production code
11. Add progress tracking for OSS uploads
12. Documentation expansion (API docs, migration guides)

---

## 7. Estimated Effort

| Phase | Tasks | Estimated LOC | Estimated Days |
|-------|-------|:------------:|:--------------:|
| P0 (3 mocks) | Create MockTaos, MockOss, MockClickHouse | ~600 | 2 |
| P0 (3 pools) | Add pool to taosx, ossx, clickhousex | ~600 | 2-3 |
| P0 (JetStream) | NATS JetStream consumer/producer | ~400 | 1-2 |
| P0 (Batch) | Batch insert for clickhousex and taosx | ~300 | 1-2 |
| P1 (Resilience) | Retry integration for 7 adapters | ~500 | 2-3 |
| P1 (TLS) | TLS enforcement in 4 adapters | ~200 | 1 |
| **Total P0-P1** | | **~2,600** | **10-14 days** |

---

## 8. Conclusion

The storage adapters have made good progress with 5,308 LOC of implementation code across 7 crates.
PostgreSQL is the most mature (production-capable). Kafka and NATS are near-production but missing
critical features (offset management, JetStream). The remaining 4 adapters need mock implementations,
connection pools, and error typing to be production-ready.

**For quant trading deployment**, the minimum viable set is: postgresx + kafkax (with offset management)
+ redisx (with mock) + taosx (with pool). ClickHouse and OSS can follow in a second wave.
