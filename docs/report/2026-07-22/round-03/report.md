# Round 3: Contract Completeness — 契约完整性审查

| 字段 | 值 |
|------|-----|
| 轮次 | 3/10 |
| 视角 | Contract Completeness — trait 语义、conformance suite、真实后端验证 |
| 日期 | 2026-07-22 |
| 基准 | `docs/report/2026-07-22/production-readiness-criteria.md` §2 S1–S7, §1 L3, §3 QT-1–QT-7 |

---

## 1. 审查摘要

| 指标 | 数据 |
|------|------|
| workspace member 总数 | 24 |
| 含 public trait 的 crate | 6（contracts、bootstrap、kernel、resiliencx、transport、evidence） |
| contracts 公开 trait | 16（含 9 storage + 1 observability + 6 venue） |
| L3 完全达标 trait | 2（KeyValueStore、Instrumentation�� |
| L3 部分达标 trait | 5（EventBus、Repository、TxContext、TxRunner、VenueAdapter） |
| L3 DEFER trait | 9（TimeSeriesStore、ObjectStore、AnalyticsSink、PubSub、MarketDataSource、InstrumentCatalog、ExecutionVenue、AccountSource、VenueTimeSource） |
| 非 scaffold 验证入口（含 ignored live） | 9（redisx KV、postgresx Tx、kafkax Bus、natsx Bus、ossx、clickhousex、taosx、binancex time、okxx time） |
| conformance suite 覆盖 trait | 10/16（contract-testkit assert_*） |
| 语义文档覆盖 trait | 11/16（first-batch 11 篇 `docs/contracts/`） |

**结论：contracts 契约框架结构已就位（16 trait + contract-testkit + 10 suite），但 L3 深度覆盖仅 KeyValueStore 和 Instrumentation 闭合；9 个 storage/exchange trait 的 conformance 套件仍 DEFER，exchange 适配器整体仍为 scaffold。**

---

## 2. 逐 crate 契约分析

### 2.1 contracts（xhyper-contracts）`crates/contracts/`

#### 2.1.1 所有公开 trait 清单

| # | Trait | 类别 | 方法数 | 语义文档 | conformance suite | 非 scaffold 入口 |
|---|-------|------|--------|----------|-------------------|------------------|
| 1 | `KeyValueStore` | storage | 2 (get/set) | `docs/contracts/key_value_store.md` | `assert_key_value_store` | **PASS**: `redisx::RedisClient` + live |
| 2 | `EventBus` | storage | 2 (publish/subscribe) | `docs/contracts/event_bus.md` | `assert_event_bus` | PARTIAL: kafkax/natsx at-most-once facade |
| 3 | `Repository<T, Id>` | storage | 2 (find/save) | `docs/contracts/repository.md` | `assert_repository` | DEFER: no real postgres Repository impl |
| 4 | `TxContext` | storage | 2 (commit/rollback) | `docs/contracts/tx_context.md` | — (inline) | DEFER: PgTxRunner 仅边界语义，无 SQL 句柄 |
| 5 | `TxRunner` | storage | 1 (begin_tx) | `docs/contracts/tx_runner.md` | `assert_tx_runner` | **PASS**: `postgresx::PgTxRunner` |
| 6 | `TimeSeriesStore` | storage | 2 (write/query) | — | — | PARTIAL: `taosx::TaosPool` REST |
| 7 | `ObjectStore` | storage | 2 (put/get) | — | — | **PASS**: `ossx::OssClient` OSS V1 |
| 8 | `AnalyticsSink` | storage | 1 (sink) | — | — | **PASS**: `clickhousex::ClickHousePool` |
| 9 | `PubSub` | storage | 2 (pub/sub) | — | — | PARTIAL: `redisx::RedisPubSub` (feature) |
| 10 | `Instrumentation` | observability | 3 (record_*) | `docs/contracts/instrumentation.md` | `assert_instrumentation` | **PASS**: `observex::TracingInstrumentation` |
| 11 | `VenueAdapter` | venue | 14 (含 2 deprecated) | `docs/contracts/` (多篇) | — | PARTIAL: binancex/okxx scaffold + time |
| 12 | `MarketDataSource` | venue | 3 (subscribe_*) | `docs/contracts/market_data_source.md` | `assert_market_data_source` | DEFER |
| 13 | `InstrumentCatalog` | venue | 1 (symbol_info) | `docs/contracts/instrument_catalog.md` | `assert_instrument_catalog` | DEFER |
| 14 | `ExecutionVenue` | venue | 4 | `docs/contracts/execution_venue.md` | `assert_execution_venue` | PARTIAL: binancex/okxx scaffold |
| 15 | `AccountSource` | venue | 2 (query_*) | `docs/contracts/account_source.md` | `assert_account_source` | DEFER |
| 16 | `VenueTimeSource` | venue | 1 (server_time) | `docs/contracts/venue_time_source.md` | `assert_venue_time_source` | PARTIAL: binancex/okxx `parse_*_server_time` |

#### 2.1.2 语义文档分析

- **11/16 trait 有文档**（first-batch）：`docs/contracts/` 下 11 篇 Markdown 覆盖 KeyValueStore、EventBus、Repository、TxContext、TxRunner、Instrumentation、VenueAdapter、MarketDataSource、InstrumentCatalog、ExecutionVenue、AccountSource、VenueTimeSource（12 页面但覆盖上述 12 trait）。
- **5 trait 缺文档**：TimeSeriesStore、ObjectStore、AnalyticsSink、PubSub 4 个新建 storage trait（ADR-003 之后新增，`lib.rs` 记为"待新增"），以��� TxContext 有行内文档但无独立文件。
- 每个已文档化 trait 的 `#[doc]` 指向对应 `docs/contracts/*.md`（`lib.rs:48,85,98,114,129,193,207,273,286,296,313,325`）。

#### 2.1.3 Conformance 套件分析

- `contract-testkit`（`crates/test-support/contracts/`）提供 10 个 `assert_*` suite 函数，覆盖：
  - `assert_key_value_store`、`assert_event_bus`、`assert_repository`、`assert_tx_runner`
  - `assert_instrumentation`
  - `assert_market_data_source`、`assert_instrument_catalog`、`assert_execution_venue`、`assert_account_source`、`assert_venue_time_source`
- `crates/contracts/tests/conformance_first_batch.rs` 驱动前 5 个 storage + obs suite。
- **缺失**：TimeSeriesStore、ObjectStore、AnalyticsSink、PubSub、VenueAdapter（主 facade）均无 `assert_*` suite。
- **禁止** provider 大宏（SPEC-TESTKIT-002 §3.2）。

#### 2.1.4 VenueAdapter 门禁

- additive default → 中文 `Invalid`（`venue_gate.rs`）。
- `tests/venue_override_gate.rs`：运行时门禁证明 binancex/okxx 已覆盖。
- **未**实现强制 compile-fail override 机控（DEFER-8 长期项）。
- Production entry → `ExecutionVenue`（无 default）。

#### 2.1.5 评级

| 维度 | 评分 (0-5) | 说明 |
|------|-----------|------|
| S1 域规格存在 | 5 | `.agents/ssot/contracts/spec/xhyper-contracts-complete-spec.md` |
| S2 对齐文档 | 5 | `docs/ssot/contracts-ssot-alignment.md` 最新 2026-07-22 |
| S3 PASS/DEFER 矩阵 | 5 | CT-1 ~ CT-11 条款清晰，DEFER 项明列 |
| S4 禁止表述 | 5 | L3 子集明确：KV+Instr 满足，其余 DEFER |
| S5 版本/成熟度标签 | 4 | L3 子集标记；STATUS 100% 结构完成但非 Production Ready |
| S6 源码对齐 | 5 | 16 trait 在 `lib.rs` 与 `venue_gate.rs` 完整对应 |
| S7 变更记录 | 5 | 2026-07-21 ~ 2026-07-22 多轮 audit 有据 |

- **L3 层级**：**子集 PASS**（KeyValueStore + Instrumentation），其余 DEFER

---

### 2.2 bootstrap（xhyper-bootstrap）`crates/bootstrap/`

#### 2.2.1 Trait 清单

- `bootstrap::traits::Instrumentation` — re-export `contracts::Instrumentation`（ADR-005）。
- `bootstrap::traits::NoopInstrumentation` — 静默空实现。
- `bootstrap::traits::EvidenceAppender` — re-export `evidence::EvidenceAppender`。
- `bootstrap::traits::BoundedMarketDataSource` — 有界行情能力（label 接口）。
- `bootstrap::traits::BoundedInstrumentCatalog` — 有界标的目录。
- `bootstrap::traits::BoundedKeyValueStore` — 有界 KV。
- `bootstrap::traits::BoundedExecutionVenue` — 有界执行场所。
- `bootstrap::traits::BoundedAccountSource` — 有界账户源。
- `bootstrap::traits::BoundedVenueTimeSource` — 有界时间源。

共计 2 个 re-export trait（Instrumentation + EvidenceAppender）和 6 个 `Bounded*` trait。

#### 2.2.2 契约角色

- **组合根**（`Bootstrap` builder）通过类型化字段注入 Instrumentation 和 Evidence（非 Service Locator）。
- `PlatformContext` 聚合横切只读依赖。
- `MarketDataContext` / `ExecutionContext` 有界上下文（DEFER-BOUND-CTX）使用 `Bounded*` trait。
- `Bounded*` trait 当前仅含 `label()` / `venue_id()` 最小标识接口；真正的 async API 面仍 DEFER。

#### 2.2.3 Conformance 评估

- `Instrumentation` re-export 委托 `contracts::Instrumentation`：已有 `assert_instrumentation` + `observex::TracingInstrumentation`。
- `EvidenceAppender` re-export 委托 `evidence::EvidenceAppender`：有 `InMemoryEvidenceAppender`。
- `Bounded*` 仅 unit tests（stub 对象安全证明），无 conformance suite。

#### 2.2.4 评级

| 维度 | 评分 | 说明 |
|------|------|------|
| L3 | 部分 | Instrumentation 面闭合；Bounded* 无 conformance；有界上下文的完整 async API DEFER |

---

### 2.3 kernel `crates/kernel/`

- **1 public trait**: `Clock` (clock.rs) — `now()` → `chrono::DateTime<Utc>`。
- 语义文档: `docs/` 下 core trait 文档。
- 有 `testkit::ManualClock`（test-support）作为 Fake 实现。
- **无** contract-testkit conformance suite（kernel 不在 contracts 域）。

评级: L1 Internal Ready（基础功能 trait）。

---

### 2.4 resiliencx `crates/resiliencx/`

- **2 public traits**: `Wait` + `AsyncWait` (retry.rs) — 重试等待策略。
- 消费 `contracts::Instrumentation` 记录 retry 事件。
- 无独立 conformance suite（策略由类型化实现覆盖：固定延迟、指数退避）。

评级: L1 Internal Ready。

---

### 2.5 transport（xhyper-transportx）`crates/transport/`

- **4 public traits**: `HttpDriver`（单方法 execute）、`WsConnector`、`WsConnection`、`HttpTransport`。
- `HttpDriver` 被 binancex/okxx 注入使用（scaffold path + mock HTTP test）。
- `MockHttpTransport` 在 test paths 中提供 mock 实现。
- 无 contract-testkit conformance suite（transport traits 未注册在 contracts 面）。

评级: L1 Internal Ready。Mock 用于离线 CI。无 conformance suite。

---

## 3. 逐 Adapter 实现分析

### 3.1 交易所适配器

#### binancex `crates/adapters/exchange/binance/`

- **实现的 contracts trait**: VenueAdapter + MarketDataSource + InstrumentCatalog + ExecutionVenue + AccountSource + VenueTimeSource（6 trait）。
- **实现深度**: 默认内存占位。可选注入 `transportx::HttpDriver` 后走 HTTP 边界。
- **非 scaffold 零起点**: `parse_binance_server_time()` — 离线解析 Binance `/api/v3/time` JSON。`server_time` 经 HttpDriver GET 真实端点。
- **Scaffold 残留**: place_order → 静态 `OrderAck`；subscribe_* → `stream::empty()`；query_position/query_balance → 硬编码空/零值；cancel/query request → mock-first string matching。
- **测试**: 22 单元测试（connect/disconnect/place/time/cancel/query/capability/transport error mapping）；`tests/live_server_time.rs`（ignored）。
- **评级**: L3 DEFER — 非真实交易所协议，订单执行/行情/MD 全为 scaffold。

#### okxx `crates/adapters/exchange/okx/`

- **结构**：与 binancex 同构。
- **非 scaffold 零起点**: `parse_okx_server_time()`。
- **评级**: L3 DEFER — 同 binancex。

### 3.2 存储适配器（7 个生产 P0）

#### redisx `crates/adapters/storage/redis/`

- **实现的 contracts trait**: `KeyValueStore`（RedisClient）。
- **生产深度**: `RedisPool`（ConnectionManager + Semaphore 背压）、`RedisClient`（Cloneable KV client）。
- **非 scaffold 入口**: `RedisLiveKv::connect()`、`connect_from_env()`。
- **Live test**: `tests/live_kv_conformance.rs`（2 tests, `#[ignore]`）。
- **L3 KeyValueStore 闭合**: 语义文档 + `assert_key_value_store` conformance + live `RedisClient`。
- **评级**: **L3 PASS** (KeyValueStore)。

#### postgresx `crates/adapters/storage/postgres/`

- **实现的 contracts trait**: `TxRunner`（PgTxRunner）。
- **生产深度**: `PostgresPool`（tokio-postgres 0.29）、`PgTransaction`、SQLSTATE→ErrorKind 映射。
- **诚实限制**: `TxContext` 边界仅 commit/rollback，不传递 SQL 句柄；业务 SQL 需使用 `PostgresPool::with_transaction`。
- **Live test**: `tests/live_postgres.rs`（`#[ignore]`）。
- **评级**: L3 部分 PASS（TxRunner 面闭合；Repository 未实现）。

#### kafkax `crates/adapters/storage/kafka/`

- **实现的 contracts trait**: `EventBus`（KafkaEventBus）。
- **生产深度**: `KafkaPool`（纯 Rust rskafka）、`KafkaProducer`（等待 broker 确认）、`KafkaConsumer`（按分区流式消费）。
- **能力边界**: EventBus 为 at-most-once（无 ack/redelivery）；可靠消费需 `KafkaConsumer`。
- **Live test**: `tests/live_event_bus.rs`（`#[ignore]`）。
- **评级**: L3 部分 PASS（EventBus at-most-once 闭合；无 ack/幂等 conformance）。

#### natsx `crates/adapters/storage/nats/`

- **实现的 contracts trait**: `EventBus`（NatsEventBus + NatsPool 实现）。
- **生产深度**: `NatsPool`（async-nats Core NATS）。
- **Live test**: `tests/live_event_bus.rs`（`#[ignore]`）。
- **评级**: L3 部分 PASS（同 kafkax）。

#### clickhousex `crates/adapters/storage/clickhouse/`

- **实现的 contracts trait**: `AnalyticsSink`（ClickHousePool）。
- **生产深度**: `ClickHousePool`（reqwest HTTP 8123）、`insert_json_each_row`。
- **Live test**: `tests/live_smoke.rs`（`#[ignore]`）。
- **评级**: L3 部分 PASS（AnalyticsSink 面有实绑定，但无 `assert_analytics_sink` 套件）。

#### ossx `crates/adapters/storage/oss/`

- **实现的 contracts trait**: `ObjectStore`（OssClient）。
- **生产深度**: `OssClient`（reqwest + OSS Signature V1）、PUT/GET/DELETE 完整。
- **Live test**: `tests/live_object_store.rs`（`#[ignore]`）。
- **评级**: L3 部分 PASS（ObjectStore 面有实绑定，但无 `assert_object_store` 套件）。

#### taosx `crates/adapters/storage/taos/`

- **实���的 contracts trait**: `TimeSeriesStore`（TaosPool）。
- **生产深度**: `TaosPool`（REST 6041）、创建 STABLE、自动探测精度、子表命名、多表批量 INSERT。
- **Live test**: `tests/live_smoke.rs`（`#[ignore]`）。
- **评级**: L3 部分 PASS（TimeSeriesStore 面有实绑定，但无 `assert_time_series_store` 套件）。

---

## 4. 非 scaffold 验证入口评估

### 4.1 运行时真实后端

| 入口 | 实现 | trait | 验证方式 | 状态 |
|------|------|-------|----------|------|
| `redisx::RedisClient` | redis-rs (7.0) | KeyValueStore | `live_kv_conformance.rs` (#188) | **L3 闭合** |
| `postgresx::PgTxRunner` | tokio-postgres 0.29 | TxRunner | `live_postgres.rs` | **L3 TxRunner 闭合** |
| `kafkax::KafkaEventBus` | rskafka (0.11) | EventBus | `live_event_bus.rs` | **L3 部分** |
| `natsx::NatsEventBus` | async-nats (0.38) | EventBus | `live_event_bus.rs` | **L3 部分** |
| `clickhousex::ClickHousePool` | reqwest/CK HTTP | AnalyticsSink | `live_smoke.rs` | **L3 部分** (缺 suite) |
| `ossx::OssClient` | reqwest/OSS V1 | ObjectStore | `live_object_store.rs` | **L3 部分** (缺 suite) |
| `taosx::TaosPool` | reqwest/TDengine REST | TimeSeriesStore | `live_smoke.rs` | **L3 部分** (缺 suite) |
| `binancex::BinanceAdapter` | transportx (optional) | VenueTimeSource | `live_server_time.rs` | **L3 DEFER** |
| `okxx::OkxAdapter` | transportx (optional) | VenueTimeSource | `live_server_time.rs` | **L3 DEFER** |

### 4.2 observability 注入链

- `observex::TracingInstrumentation` 实现 `contracts::Instrumentation` → **L3 PASS**。
- `resiliencx` 消费 contracts::Instrumentation（通过 bootstrap 注入）→ 间接验证。
- `bootstrap::Bootstrap::new()` 默认注入 TracingInstrumentation。

---

## 5. conformance suite 覆盖矩阵

| trait | assert_* 存在 | contracts tests 驱动 | 生产 adapter 运行 |
|-------|---------------|---------------------|-------------------|
| KeyValueStore | `assert_key_value_store` | `conformance_first_batch` | `redisx` live (ignored) |
| EventBus | `assert_event_bus` | `conformance_first_batch` | — (仅 mock) |
| Repository | `assert_repository` | `conformance_first_batch` | — (Fake 仅 dev) |
| TxContext | — (inline tests) | — | — |
| TxRunner | `assert_tx_runner` | `conformance_first_batch` | — (Fake 仅 dev) |
| TimeSeriesStore | — | — | — |
| ObjectStore | — | — | — |
| AnalyticsSink | — | — | — |
| PubSub | — | — | — |
| Instrumentation | `assert_instrumentation` | `conformance_first_batch` | `observex` |
| VenueAdapter | — (gate, not suite) | `venue_override_gate` | — |
| MarketDataSource | `assert_market_data_source` | — (testkit, not contracts) | — |
| InstrumentCatalog | `assert_instrument_catalog` | — | — |
| ExecutionVenue | `assert_execution_venue` | — | — |
| AccountSource | `assert_account_source` | — | — |
| VenueTimeSource | `assert_venue_time_source` | — | — |

**覆盖率**: contract-testkit 提供 10/16 的 suite（62.5%），但仅 5 在 contracts 集成测中运行；0 在生产 adapter 上运行（KV live conformance 除外）。

---

## 6. 量化交易场景评估

| 场景 | 评定 | 说明 |
|------|------|------|
| QT-1 市场数据接入 | **Gap** | exchange 全 scaffold；subscribe_* 返回空流 |
| QT-2 订单执行 | **Gap** | place_order 返回静态 ack；无真实协议对接 |
| QT-3 仓位与风险管理 | **Conditional** | decimalx+resiliencx 有基础能力；无 integrated test |
| QT-4 持久化与审计 | **Conditional** | 7 storage P0 生产客户端 + live；evidence 有 InMemory；缺完整 pipeline |
| QT-5 配置与调度 | **Conditional** | configx KV 有基础能力；schedulex 有 registry |
| QT-6 可观测性 | **Conditional** | Instrumentation 注入链完整；缺 production tracing wireup |
| QT-7 数据聚合与分析 | **Conditional** | clickhousex/taosx 有生产客户端；缺聚合 pipeline conformance |

---

## 7. 缺失项（Gaps）

### 7.1 关键 gap

| ID | Gap | 影响 | 优先级 |
|----|-----|------|--------|
| G3-1 | **contract-testkit 缺 6 个 trait suite**（TimeSeries/Object/Analytics/PubSub/VenueAdapter/ExecutionVenue） | 4 storage + 2 exchange trait 无法验证合同 | P1 |
| G3-2 | **Repository 无 postgresx 实现** | 泛型 Repository 在生产路径不可用 | P1 |
| G3-3 | **exchange 全 scaffold** | binancex/okxx 订单/行情/MD 非真实协议 | P0 (核心业务) |
| G3-4 | **TxContext 无 SQL 句柄** | 事务内 SQL 需绕过 trait 直接使用 pool API | P1 |
| G3-5 | **Bounded* trait 仅 label 接口** | 有界上下文组合根的 async 能力面 DEFER | P2 |
| G3-6 | **VenueAdapter compile-fail override** | 树外实现者无法在编译期检测未覆盖 | P3 (DEFER-8) |

### 7.2 非 gap（按治理已判定 DEFER）

- Cluster / Sentinel / JetStream / EOS / multipart — 已声明 DEFER（非 P0）。
- Package stable / crates.io — 禁止宣称。
- Adapter 全量业务 Production Ready — 禁止宣称。

---

## 8. 轮次结论

### 8.1 达成

1. **contracts 16 trait 面完整**：Additive Only 设计，有 clear doc comment 指向语义文档。
2. **contract-testkit 独立 crate**：10 个 assert_* suite + Fake/Recording 实现，命名空间统一为 `contract_testkit::`。
3. **2 个 L3 完全闭合 trait**：`KeyValueStore`（语义 + conformance + redisx live）和 `Instrumentation`（语义 + conformance + observex）。
4. **7 个 storage adapter 生产客户端已落地**（P0）：全有 FOUNDATIONX_* env 注入 + live `#[ignore]` 验证可运行。
5. **VenueAdapter 运行时门禁**：additive default → 中文 Invalid，树内 binancex/okxx 已覆盖。
6. **bootstrap 组合根**：Typed composition（ADR-016），Instrumentation/Evidence 注入链完整。

### 8.2 受阻

1. **exchange 适配器全为 scaffold** — 订单执行、行情、深度均为空流/静态值。这是量化交易核心 gap（QT-1/QT-2）。
2. **contract-testkit 未覆盖 6 个 trait** — TimeSeriesStore/ObjectStore/AnalyticsSink/PubSub/VenueAdapter/ExecutionVenue 暂无 suite。
3. **生产适配器上的 conformance suite 运行覆盖为 0** — 除 KV live 外无 adapter 运行 `assert_*`。

### 8.3 建议

1. **P0**: 展开 exchange 适配器真实协议实现（至少一个交易所的��整订单/MD）。
2. **P1**: 补齐 contract-testkit 的 6 个缺失 suite + 在生产 adapter 上运行。
3. **P1**: 实现 Repository 的 postgresx 生产绑定。
4. **P2**: 拓展 Bounded* trait 的 async 能力面。
5. **P3**: 将 assert_* suite 从 contracts 集成测推广到各 adapter crate（引入 contract-testkit dev-dep）。

### 8.4 质量基线

| 门禁 | 状态 |
|------|------|
| `cargo test --workspace --all-targets` | 待验证（应由 CI 或上一轮已通过） |
| `cargo clippy --workspace -- -D warnings` | 待验证 |
| `cargo fmt --all --check` | 待验证 |
| `scripts/cov-gate-100.mjs -p contracts` | 已声明 100%（见对齐文档） |
| `cargo deny check` | 已声明通过 |

---

## 附录 A：全 workspace trait 总览

```
All public traits across 24 workspace crates (30 total):

contracts (16):
  KeyValueStore, EventBus, Repository, TxContext, TxRunner,
  TimeSeriesStore, ObjectStore, AnalyticsSink, PubSub,
  Instrumentation,
  VenueAdapter, MarketDataSource, InstrumentCatalog, ExecutionVenue, AccountSource, VenueTimeSource

kernel (1):
  Clock

resiliencx (2):
  Wait, AsyncWait

transport (4):
  HttpDriver, WsConnector, WsConnection, HttpTransport

bootstrap (8):
  Instrumentation (re-export contracts), EvidenceAppender (re-export evidence),
  NoopInstrumentation (struct, not trait),
  BoundedMarketDataSource, BoundedInstrumentCatalog, BoundedKeyValueStore,
  BoundedExecutionVenue, BoundedAccountSource, BoundedVenueTimeSource

evidence (1):
  EvidenceAppender
```

## 附录 B：审查方法

1. 读取 `production-readiness-criteria.md` L3 标准（trait 语义文档 + conformance suite + 非 scaffold 入口）。
2. 全文扫描 `crates/contracts/src/lib.rs` + `venue_gate.rs`（581 行），提取所有 trait 定义。
3. 读取所有 adapter `lib.rs` + `adapter.rs`/`client.rs`/`bus.rs`/`runner.rs`（约 2000 行），对照 contracts trait 检查实现深度。
4. 读取 `crates/test-support/contracts/src/lib.rs` 确认 conformance suite 覆盖范围。
5. 交叉引用 `docs/ssot/contracts-ssot-alignment.md` + `docs/ssot/adapters-ssot-alignment.md` 的治理判定。
6. 源码级检查：`grep "pub trait"` 扫描全 workspace 30 个 trait。

源码确认日期：2026-07-22。所有观察基于 `/home/workspace/infra.rs` 的当前源码状态。
