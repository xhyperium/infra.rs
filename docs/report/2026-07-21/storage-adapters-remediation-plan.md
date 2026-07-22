# Storage Adapters — Production Remediation Plan & Effort Estimate

> **Date:** 2026-07-21 | **Based on:** [storage-adapters-production-readiness.md](storage-adapters-production-readiness.md)
> **Total Gap:** 3 P0 items, 5 P1 items, 5 P2 items | **Estimate:** 10-14 days

---

## 1. Plan Overview

| Phase | Priority | Tasks | Crates Affected | Days |
|-------|:--------:|-------|:---------------:|:----:|
| A. Mocks | P0 | 3 mocks | taosx, ossx, clickhousex | 2 |
| B. Connection Pools | P0 | 3 pools | taosx, ossx, clickhousex | 2-3 |
| C. JetStream | P0 | NATS persistence | natsx | 1-2 |
| D. Batch Writes | P0 | Batch insert | clickhousex, taosx | 1-2 |
| E. Retry Integration | P1 | Circuit breaker × 7 | all 7 | 2-3 |
| F. TLS Enforcement | P1 | SSL config in 4 adapters | postgresx, redisx, kafkax, natsx | 1 |
| G. Error Typing | P1 | Type errors in 3 adapters | taosx, ossx, clickhousex | 1 |
| H. Repository Trait | P1 | Production Repository impl | postgresx | 1 |
| I. Polish | P2 | Tests, docs, scaffold cleanup | all 7 | 2-3 |
| **Total** | | | | **10-14** |

---

## 2. Detailed Task Breakdown

### Phase A: Mock Implementations (P0, 2 days)

**Goal:** Provide in-memory mock implementations for offline testing.

#### A.1 MockTaos (taosx) — 0.7 days

```rust
// crates/adapters/storage/taos/src/mock.rs
#[derive(Clone)]
pub struct MockTaosStore {
    tables: Arc<RwLock<HashMap<String, Vec<Tick>>>>,
}

#[async_trait]
impl TimeSeriesStore for MockTaosStore {
    async fn write_series(&self, table: &str, points: Vec<Tick>) -> XResult<()> {
        self.tables.write().unwrap().entry(table.into()).or_default().extend(points);
        Ok(())
    }
    async fn query_series(&self, table: &str, start: i64, end: i64) -> XResult<Vec<Tick>> {
        // Filter by time range from in-memory store
    }
}
```

Files to create: `src/mock.rs` (~150 LOC)
Feature gate: `#[cfg(feature = "scaffold")]`

#### A.2 MockOss (ossx) — 0.7 days

```rust
// crates/adapters/storage/oss/src/mock.rs
pub struct MockObjectStore {
    objects: Arc<RwLock<HashMap<String, Bytes>>>,
}
```

Files to create: `src/mock.rs` (~120 LOC)

#### A.3 MockClickHouse (clickhousex) — 0.6 days

```rust
// crates/adapters/storage/clickhouse/src/mock.rs
pub struct MockAnalyticsSink {
    events: Arc<RwLock<Vec<(String, Bytes)>>>,
}
```

Files to create: `src/mock.rs` (~100 LOC)

**Deliverable:** 3 new `mock.rs` files (~370 LOC total), gated behind `scaffold` feature.

---

### Phase B: Connection Pools (P0, 2-3 days)

**Goal:** Add connection pooling to adapters that currently use single connections.

#### B.1 TaosPool (taosx) — 1 day

Current `TaosPool` is a direct REST client. Need to add:
- Connection pool with configurable max connections
- Health check (`SELECT 1`)
- Connection reuse for REST client

Files to modify: `src/client.rs`, `src/config.rs` (~200 LOC)

#### B.2 OssPool (ossx) — 0.7 days

Add `reqwest::Client` pool for OSS HTTP connections:
- Connection pooling via `reqwest::Client::builder().pool_max_idle_per_host()`
- Health check via `HEAD /` on bucket

Files to modify: `src/client.rs`, `src/config.rs` (~120 LOC)

#### B.3 ClickHousePool (clickhousex) — 0.7 days

Add `reqwest::Client` pool for ClickHouse HTTP connections:
- Same pattern as OSS
- Health check via `SELECT 1`

Files to modify: `src/client.rs`, `src/config.rs` (~100 LOC)

**Deliverable:** Pool configurations for 3 adapters (~420 LOC).

---

### Phase C: NATS JetStream (P0, 1-2 days)

**Goal:** Add JetStream persistent stream support to natsx.

Current `NatsEventBus` is at-most-once (Core NATS). For quant trading production:
- In-flight market data and orders must survive service restart
- JetStream provides at-least-once delivery with persistent storage

```rust
// crates/adapters/storage/nats/src/jetstream.rs
pub struct JetStreamBus {
    jetstream: async_nats::jetstream::Context,
}
```

Files to create: `src/jetstream.rs` (~300 LOC)
Files to modify: `src/config.rs` (add JetStream config), `src/lib.rs` (feature gate)

Feature gate: `jetstream`

**Deliverable:** JetStream consumer/producer (`~300 LOC`), behind `jetstream` feature.

---

### Phase D: Batch Writes (P0, 1-2 days)

**Goal:** Add batch/chunked insert support for analytics and time-series adapters.

#### D.1 clickhousex batch insert — 0.5 days

Current: single-row `INSERT INTO ... FORMAT JSONEachRow` per `sink()` call.
Target: Buffer accumulated rows and flush in batches.

```rust
impl ClickHousePool {
    pub async fn sink_batch(&self, rows: &[Bytes]) -> XResult<()> { ... }
}
```

Files to modify: `src/client.rs` (~80 LOC)

#### D.2 taosx batch insert — 0.5 days

Current: single `INSERT INTO ... VALUES (...)` per `write_series()` call.
Target: Multiple rows per INSERT statement.

Files to modify: `src/client.rs` (~60 LOC)

**Deliverable:** Batch insert methods for 2 adapters (~140 LOC).

---

### Phase E: Retry Integration (P1, 2-3 days)

**Goal:** Integrate `resiliencx` retry/circuit-breaking across all 7 adapters.

Each adapter gets a `RetryConfig` with:
- Max retries: 3
- Backoff: exponential (1s, 2s, 4s)
- Circuit breaker: 5 consecutive failures → open for 30s

```rust
// Example: postgresx
use resiliencx::RetryPolicy;

impl PostgresPool {
    pub async fn execute_with_retry<R>(&self, f: impl Fn() -> XResult<R>) -> XResult<R> {
        self.retry_policy.retry(f).await
    }
}
```

Each adapter: ~60 LOC for retry config + ~20 LOC for pool integration × 7 = ~560 LOC total.

**Deliverable:** Retry policy classes per adapter (~560 LOC), configurable via `RetryConfigBuilder`.

---

### Phase F: TLS Enforcement (P1, 1 day)

**Goal:** Add SSL/TLS config enforcement for production deployments.

| Adapter | Current | Target | LOC |
|---------|---------|--------|:--:|
| postgresx | `SslMode::Disable` default | `SslMode::Require` or `VerifyFull` in prod | 50 |
| redisx | No TLS config builder | `use_tls: bool` with `rustls` feature | 50 |
| kafkax | SASL_PLAINTEXT default | SASL_SSL enforcement | 50 |
| natsx | No TLS config | `tls: bool` config option | 50 |

**Deliverable:** TLS config options for 4 adapters (~200 LOC).

---

### Phase G: Error Typing (P1, 1 day)

**Goal:** Replace generic error types with `thiserror` typed errors.

| Adapter | Current | Target | LOC |
|---------|---------|--------|:--:|
| taosx | Generic XError | `TaosError` with variants (ConnectionFailed, InsertFailed, QueryTimedOut) | 80 |
| ossx | Generic XError | `OssError` with variants (BucketNotFound, SignatureInvalid, UploadFailed) | 80 |
| clickhousex | Generic XError | `ClickHouseError` with variants (TableNotFound, InsertFailed, QueryError) | 80 |

**Deliverable:** Typed error enums for 3 adapters (~240 LOC).

---

### Phase H: Repository Trait (P1, 1 day)

**Goal:** Implement `Repository<T, Id>` on `PostgresPool` in production.

Current: Repository only implemented on scaffold `PostgresAdapter`.
Target: Add `Repository` impl on `PostgresPool` with:
- `find(id)` → `SELECT * FROM {table} WHERE id = $1`
- `save(entity)` → `INSERT INTO {table} ... ON CONFLICT (id) DO UPDATE`

Files to modify: `src/pool.rs` or new `src/repository.rs` (~150 LOC).

**Deliverable:** Production `Repository` impl (~150 LOC).

---

### Phase I: Polish (P2, 2-3 days)

#### I.1 Test Coverage — 1 day
- Expand integration tests from 1 to 3+ per adapter
- Add connection failure tests
- Add transaction boundary tests for postgresx

#### I.2 Documentation — 0.5 days
- Migrate guides for each adapter
- Config reference completeness
- Add quant trading usage examples

#### I.3 Scaffold Cleanup — 0.5 days
- Verify all scaffolds are behind feature gates
- Remove any scaffold modules leaking into production paths

---

## 3. Dependency Graph

```
Phase A (Mocks) ─────────────────────────────┐
Phase B (Pools) ─────────────────────────────┤
Phase C (JetStream) ─────────────────────────┤  All independent (parallel)
Phase D (Batch) ─────────────────────────────┤
                                              │
Phase E (Retry) ← depends on B (pools exist) ┘
Phase F (TLS)   ← independent
Phase G (Error)  ← independent
Phase H (Repo)   ← independent

Phase I (Polish) ← depends on A-H
```

Phases A-D can run in parallel (different crates). Phases E-H can start after B is done.

---

## 4. Resource Plan

### Single Developer: 10-14 calendar days

| Week | Mon | Tue | Wed | Thu | Fri |
|------|-----|-----|-----|-----|-----|
| 1 | A1 (MockTaos) | A2 (MockOss) | B1 (TaosPool) | B2 (OssPool) | C (JetStream) |
| 2 | D (Batch) | E (Retry × 4) | E (Retry × 3) + F (TLS) | G (Error) + H (Repo) | I (Polish) |

### Two Developers: 7-10 days

| Developer 1 | Developer 2 |
|-------------|-------------|
| A (Mocks) + C (JetStream) | B (Pools) + D (Batch) |
| E (Retry × 4) + H (Repo) | E (Retry × 3) + F (TLS) + G (Error) |
| I (Polish — tests) | I (Polish — docs) |

---

## 5. Success Criteria

Each phase is **done** when:

| Phase | Criteria |
|-------|----------|
| A | `cargo test -p {crate} --features scaffold` passes with mock |
| B | Health check endpoint responds, pool size configurable |
| C | `cargo test -p natsx --features jetstream` passes with local NATS |
| D | Batch insert throughput > 10× single-row insert |
| E | Circuit breaker opens after 5 consecutive failures |
| F | `SslMode::Require` is default, connection fails without TLS in CI |
| G | All error variants have docstrings, error chain preserved |
| H | `impl Repository<T, Id> for PostgresPool` compiles and passes tests |
| I | `cargo test --workspace` passes, `cargo clippy -D warnings` passes |

---

## 6. Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|:---------:|:------:|------------|
| NATS JetStream API changes | Low | Medium | Pin async-nats version |
| TDengine REST API changes | Low | Low | Use stable v3.x API endpoints |
| Aliyun OSS signature incompatibility | Low | Medium | Test with dev bucket before production |
| Connection pool configuration conflicts | Medium | Low | Use `ConfigBuilder` pattern with sensible defaults |
| Scaffold feature gate breakage | Low | Low | Verify `#[cfg(feature = "scaffold")]` in CI |
