# Round 7: Quantitative Trading — 量化交易应用场景评估

| 字段 | 值 |
|------|-----|
| 轮次 | 7/10 |
| 视角 | 量化交易应用场景 |
| 日期 | 2026-07-22 |
| 审查人 | general-purpose-7 |
| 基准 | [生产就绪判断框架](../production-readiness-criteria.md) |

## 1. 审查摘要

Round 7 从 7 个量化交易场景逐一评估 infra.rs workspace 中每个 crate 的生产可用性。核心发现：

- **QS-1 (市场数据接入)**：transportx 传输层就绪；binancex / okxx 仅 scaffold，**不可**用于生产行情订阅
- **QS-2 (订单执行)**：contracts + canonical 契约与 DTO 层就绪；exchange adapter 仅 scaffold，**不可**用于真实交易
- **QS-3 (仓位与风险管理)**：decimalx（货币计算）+ resiliencx（熔断/限流）均可用；缺口：resiliencx 熔断/限流为**无墙钟**模式
- **QS-4 (持久化与审计)**：7 个 storage adapter 生产默认客户端已落地（live 已验）；evidence 仅 dev-only 本地文件，**非**生产合规审计
- **QS-5 (配置与调度)**：configx 仅内存 KV（无多源/热更新）；schedulex 是 ID 登记表，**不是**定时调度器
- **QS-6 (可观测性)**：observex 仅 tracing info，无 OTEL；kernel 错误分类/关停就绪
- **QS-7 (数据聚合与分析)**：decimalx + canonical + storage adapter 数据链可用；无内置指标计算库

| 场景 | 就绪度 | 关键缺口 |
|------|--------|---------|
| QT-1 市场数据接入 | **Conditional** | exchange adapter 仅 scaffold；无 REST/WS 行情协议实现 |
| QT-2 订单执行 | **Conditional** | 契约层就绪；adapter 无真实交易协议实现 |
| QT-3 仓位与风险管��� | **Ready** | decimalx + resiliencx 可用；熔断无墙钟需应用层计时 |
| QT-4 持久化与审计 | **Conditional** | storage 生产就绪；evidence 非合规审计 |
| QT-5 配置与调度 | **Gap** | 无生产配置系统；无定时调度执行器 |
| QT-6 可观测性 | **Conditional** | 仅 tracing info；无 OTEL/metrics/告警 |
| QT-7 数据聚合与分析 | **Conditional** | 存储层就绪；无指标计算引擎 |

---

## 2. 场景逐项评估

### QT-1: 市场数据接入

**场景描述**：WebSocket/HTTP 实时行情、深度、K 线数据接入。

| Crate | 判定 | 证据 |
|-------|------|------|
| **transportx** | **Ready** | `HttpDriver` (reqwest) + `WsConnector`/`WsConnection` (tungstenite) 已实现，含生产默认（30s 超时、16 MiB HTTP 体上限、4 MiB WS 帧上限）。`MockHttpTransport` 提供测试注入��。见 `crates/transport/src/lib.rs:186-221`。 |
| **binancex** | **Conditional** | 实现 `VenueAdapter` scaffold。`BinanceAdapter` 可注入 `HttpDriver` 走 HTTP 传输（`http_get`/`http_post`），`parse_binance_server_time` 可解析 JSON。但 `subscribe_ticks`/`subscribe_orderbook`/`subscribe_trades` 返回错误占位，`fetch_candles` 返回零值数据。见 `crates/adapters/exchange/binance/src/adapter.rs:80-213`。**必须**实现 WebSocket 行情流解析与 K 线 REST API 才能用于生产。 |
| **okxx** | **Conditional** | 与 binancex 近同结构；`OkxAdapter` scaffold + `parse_okx_server_time`。`subscribe_*` 方法均为占位。见 `crates/adapters/exchange/okx/src/lib.rs:1-8`。 |

**汇总**：transportx 传输层能满足 HTTP/WS 连接需求，但交易�� adapter 需要投入协议实现（Binance/OKX WebSocket 行情流、订单簿 depth snapshot/diff、K 线 REST API）。

---

### QT-2: 订单执行

**场景描述**：下单、撤单、改单、批量操作。

| Crate | 判定 | 证据 |
|-------|------|------|
| **contracts** | **Ready** | `VenueAdapter` + `ExecutionVenue`（推荐生产入口，无 additive default）+ `MarketDataSource` + `AccountSource` + `VenueTimeSource` + `InstrumentCatalog`。契约体系完整，含 `CAN-ID Approved` 结构化撤单/查单（`CancelOrderRequest` + `OrderRef`）。见 `crates/contracts/src/lib.rs:206-328`。 |
| **canonical** | **Ready** | `Order` (v1.1 wire)、`OrderAck` (v1 wire)、`OrderStatus`、`Side`、`CancelOrderRequest` (v1 wire)、`OrderRef` (v1 wire)、`Position` (v1.3 wire)、`Money`。全部 DTO 带 `deny_unknown_fields` + golden fixture。见 `crates/types/canonical/src/lib.rs:63-155`。 |
| **binancex** | **Conditional** | 实现 `VenueAdapter` scaffold — `place_order` 返回错误，`cancel_order` 占位 OK。**缺失**：签名认证、REST 下单（POST /api/v3/order）、订单状态机、错误码映射。见 `crates/adapters/exchange/binance/src/adapter.rs:215+`。 |
| **okxx** | **Conditional** | 与 binancex 同理；`VenueAdapter` scaffold，无真实下单/撤单逻辑。**缺失**：OKX V5 API 签名、REST+WS 下单通道、错误分类。 |

**汇总**：contracts + canonical 的契约面完整，可直接用于实现 adapter。但 exchange adapter 需投入 Binance/OKX REST 签名认证、下单/撤单协议、订单状态映射、错误分类。这是连接真实交易所的最关键缺口。

---

### QT-3: 仓位与风险管理

**场景描述**：仓位计算、风险限额、熔断保护。

| Crate | 判定 | 证据 |
|-------|------|------|
| **decimalx** | **Ready** | `Decimal` + `checked_add`/`checked_sub`/`checked_mul`/`checked_div`（必须显式 `RoundingStrategy`）为资金计算提供强类型安全。`Price`/`Qty`/`Ratio`/`Currency`/`Money` newtype 包装防混淆。`MAX_SCALE=18`，`forbid(unsafe_code)`。见 `crates/types/decimal/src/lib.rs:139-371`。完整性：5 种舍入策略覆盖 Floor/Ceiling/HalfUp/HalfDown/HalfEven。 |
| **canonical** | **Ready** | `Position { symbol, qty: Qty, entry_price: Price }` (v1.3 wire) 可用作仓位表示。见 `crates/types/canonical/src/lib.rs:154-161`。 |
| **resiliencx** | **Ready*** | `CircuitBreaker`（Closed/Open/HalfOpen 三态）、`RateLimiter`（令牌桶）、`Bulkhead`（并发上限 RAII）、`RetryConfig`（同步/异步 + Backoff + jitter）。LCOV 行 100%（修复后）。见 `crates/resiliencx/src/lib.rs:1-45`。**注意**：熔断 HalfOpen 过渡依赖拒绝计数（非墙钟冷却），限流需显式 `refill(n)`（非按时间自动补充），需应用层额外驱动墙钟。`Instrumentation` 注入点已预留（re-export `contracts::Instrumentation`），禁止直接依赖 observex。 |

**汇总**：decimalx + resiliencx 组合已为仓位计算和风控提供核心能力。熔断/限流"无墙钟"模式需要在应用层（如 bootstrap 组合根）定期调用 `refill` 和 `circuit_breaker` 过渡方法。

---

### QT-4: 持久化与审计

**场景描述**：订单/成交/Tick 落库、审计证据链。

| Crate | 判定 | 证据 |
|-------|------|------|
| **canonical** | **Ready** | DTO 全部 wire-committed（v1-v1.3），可直接序列化/反序列化落库。见 `crates/types/canonical/src/lib.rs:52-55`。 |
| **evidence** | **Conditional** | `EvidenceAppender` trait + `InMemoryEvidenceAppender`（进程内）+ `FileEvidenceAppender`（本地文件最小持久化）。策略文件明确标注 **"dev-only"**（`allows_in_memory_for_compliance() == false`）。见 `crates/evidence/src/lib.rs:1-10`。**缺口**：无远程签名链、无跨进程总线、无合规级别审计。 |
| **redisx** | **Ready** | `RedisPool` + `RedisClient`（实现 `KeyValueStore`）+ `PubSub` + `FOUNDATIONX_REDISX_*` 环境变量 + live/bench。生产默认路径完整。gap-matrix P0 **done**。见 `crates/adapters/storage/redis/src/lib.rs:1-32`。 |
| **postgresx** | **Ready** | `PostgresPool` + `PgTransaction` + `PgTxRunner`（`TxRunner`）+ SQLSTATE→`ErrorKind` 映射 + 参数化查询（防注入）+ live/bench。gap-matrix P0 **done**。见 `crates/adapters/storage/postgres/src/lib.rs:1-22`。 |
| **clickhousex** | **Ready** | `ClickHousePool` + `ClickHouseClient` HTTP（端口 8123）+ `AnalyticsSink` + live/bench。gap-matrix P0 **done**。见 `crates/adapters/storage/clickhouse/src/lib.rs:1-19`。 |
| **taosx** | **Ready** | `TaosPool` + `TaosClient` REST（端口 6041）+ `TimeSeriesStore`（`Tick.ts` 纳秒 epoch）+ live/bench。gap-matrix P0 **done**。见 `crates/adapters/storage/taos/src/lib.rs:1-19`。 |
| **ossx** | **Ready** | `OssClient` OSS Signature V1 + `ObjectStore` + `FOUNDATIONX_OSSX_*` + live/bench。multipart **DEFER**。gap-matrix P0 **done**。见 `crates/adapters/storage/oss/src/lib.rs:1-28`。 |
| **kafkax** | **Ready** | `KafkaPool` + `Producer` + `Consumer` + `EventBus`（at-most-once）+ SASL + live/bench。EOS/tx **DEFER**。gap-matrix P0 **done**。见 `crates/adapters/storage/kafka/src/lib.rs:1-18`。 |
| **natsx** | **Ready** | `NatsPool` + `EventBus`（at-most-once）+ `FOUNDATIONX_NATS_*` + live/bench。JetStream **DEFER**。gap-matrix P0 **done**。见 `crates/adapters/storage/nats/src/lib.rs:1-17`。 |

**汇总**：7 个 storage adapter 生产默认客户端已全部落地（#188-#190），live 验证通过。evidence 仅 dev-only 本地文件，不满足量化交易合规审计需求（需要远程签名链 + 不可篡改存储）。

---

### QT-5: 配置与调度

**场景描述**：策略参数管理、定时任务调度、热更新能力。

| Crate | 判定 | 证据 |
|-------|------|------|
| **configx** | **Conditional** | `ConfigStore` — 内存 `HashMap<String, String>` + RwLock。提供 `get`/`set`/`remove`/`contains_key` + `ConfigDiff`/`snapshots_agree` + `subset_snapshot`。active SSOT §2–§7 可移植子集 PASS。**缺口**：多源加载、类型化 schema、热更新、secret 管理均 **DEFER**。见 `crates/configx/src/lib.rs:1-81`。 |
| **schedulex** | **Gap** | `Scheduler` — 进程内任务 ID 登记表。**明确非目标**：Once/FixedDelay/FixedRate/cron/并发 lease/timeout token/graceful shutdown/持久化恢复。active SSOT §3 明确禁止 Clock/timer/Job/Run。见 `crates/schedulex/src/lib.rs:1-7`。**不可**用于定时任务调度执行。 |

**汇总**：configx 可用于进程内简单配置传递，但缺少多源加载和热更新。schedulex 仅 ID 登记，不提供任何定时触发或执行能力。量化交易需要独立的配置中心（如 etcd/consul 集成）和调度执行器（如 cron/tokio interval）。

---

### QT-6: 可观测性

**场景描述**：分布式链路追踪、指标采集、告警。

| Crate | 判定 | 证据 |
|-------|------|------|
| **observex** | **Conditional** | `TracingInstrumentation` 仅 `tracing::info!` 三个方法（retry/circuit_open/circuit_close）。`PrefixedInstrumentation` 前缀包装，`CountingInstrumentation`（本地计数器，**非** OTEL）。SSOT 明确 OTEL exporter/flush/shutdown **DEFER**。见 `crates/observex/src/lib.rs:1-52`。**缺口**：无 OTEL Span/Metrics/Logging 管线，无导出器。 |
| **kernel** | **Ready** | `ErrorKind` 9 类错误分级（含 `is_retryable`/`is_bug`）+ `ShutdownSignal`（一次触发、多方观察、不可逆）+ `ComponentState` 合法状态转换（Created→Starting→Running→Draining→Stopped）+ `Clock`/`SystemClock` 时间注入。见 `crates/kernel/src/lib.rs:1-23`。 |

**汇总**：kernel 提供错误分类和关停信号；observex 仅最小 tracing，不满足量化交易对分布式追踪和指标告警的生产需求。

---

### QT-7: 数据聚合与分析

**场景描述**：Tick→K 线转换、技术指标计算、回测数据准备。

| Crate | 判定 | 证据 |
|-------|------|------|
| **decimalx** | **Ready** | `Decimal` 已提供 `checked_*` 运算 + `RoundingStrategy`，可构建 OHLC 聚合、指标公式计算。 |
| **canonical** | **Ready** | `Tick { symbol, bid, ask, ts }` (v1.2 wire) + `Trade { symbol, price, qty, ts }` (v1.2 wire) 可用作聚合输入格式。 |
| **clickhousex** | **Ready** | `AnalyticsSink` + `ClickHousePool` HTTP 可写入分析事件。适合 OLAP 聚合查询（如 GROUP BY 窗口）。 |
| **taosx** | **Ready** | `TimeSeriesStore` + `TaosPool` REST 可写入时间序列点。适合 Tick 级原始数据存储。 |
| **postgresx** | **Ready** | `PostgresPool` 可存储聚合结果（如策略回测净值和交易记录）。 |

**汇总**：decimalx + canonical 提供计算和数据形状基础，storage adapter 提供持久化通道。主要缺口：**无**内置技术指标库（SMA/EMA/MACD/RSI 等）、**无**Tick→K 线聚合引擎、**无**回测框架。

---

## 3. 量化交易就绪度总评

```text
场景就绪度分布

  Ready:         QT-1(1/3)  QT-2(2/4)  QT-3(3/3)  QT-4(9/9-prod)  QT-7(5/5)
  Conditional:   QT-1(2/3)  QT-2(2/4)  QT-4(1/9)    QT-5(1/2)      QT-6(1/2)
  Gap:           QT-5(1/2)
  N/A:           无
```

| 层 | 就绪 crates | 缺口 crates |
|----|------------|------------|
| **传输/基础设施** | transportx (Ready), kernel (Ready) | — |
| **契约/类型** | contracts (Ready), canonical (Ready), decimalx (Ready) | — |
| **弹性/风控** | resiliencx (Ready*) | 无墙钟熔断需应用层计时 |
| **持久化** | redisx/postgresx/clickhousex/taosx/ossx/kafkax/natsx (Ready) | evidence (Conditional) |
| **可观测性** | — | observex (Conditional) |
| **配置/调度** | configx (Conditional) | schedulex (Gap) |
| **交易所接入** | — | binancex/okxx (Conditional, **关键缺口**) |

**核心瓶颈**：binancex/okxx 仅 scaffold，无真实交易所 REST/WS 协议实现，这是连接量化交易系统与真实市场的**硬阻塞**。

---

## 4. 场景缺口汇总

| ID | 场景 | 缺口描述 | 影响 |
|----|------|---------|------|
| **GAP-QT1-01** | 行情接入 | binancex/okxx 无 WebSocket 行情流解析 | 无法订阅实时 Tick/OrderBook/Trade |
| **GAP-QT1-02** | 行情接入 | binancex/okxx 无 REST K 线 API 实现 | 无法拉取历史 K 线数据 |
| **GAP-QT2-01** | 订单执行 | binancex/okxx 无 REST 签名认证 | 无法向交易所发送下单请求 |
| **GAP-QT2-02** | 订单执行 | binancex/okxx 无下单/撤单协议实现 | 无法执行真实订单操作 |
| **GAP-QT2-03** | 订单执行 | binancex/okxx 无错误码分类映射 | 无法根据交易所错误分类重试/告警 |
| **GAP-QT3-01** | 风险管理 | resiliencx 熔断/限流无墙钟 | 需应用层额外驱动（定期 refill/transition） |
| **GAP-QT4-01** | 审计 | evidence 无远程签名链/不可篡改存储 | 不满足合规审计 |
| **GAP-QT5-01** | 配置 | configx 无多源/热更新/类型化 schema | 策略参数无法外部管理和热加载 |
| **GAP-QT5-02** | 调度 | schedulex 是 ID 登记表，非定时调度器 | 无定时任务执行能力 |
| **GAP-QT6-01** | 可观测性 | observex 仅 tracing::info，无 OTEL | 无法与分布式追踪/指标平台集成 |
| **GAP-QT7-01** | 分析 | 无内置技术指标/Tick→K 线/回测引擎 | 需另行实现或集成外部库 |

---

## 5. 轮次结论

### 可量化交易使用的前提条件

要在 infra.rs 基础上构建量化交易系统，**最低**需要完成：

1. **exchange adapter 生产化**（P0）：binancex/okxx 实现 REST 签名认证 + 下单/撤单/查单 + WebSocket 行情流（至少 Tick/Trade）+ 错误码→ErrorKind 映射
2. **wall-clock 驱动**（P0）：resiliencx 的 `CircuitBreaker` 和 `RateLimiter` 需追加墙钟模式，或由应用层实现定时任务
3. **可观测性增强**（P1）：observex 集成 OTEL Span/Metrics exporter

### 可直接复用的能力

- **decimalx**：资金安全计算，生产可用
- **canonical + contracts**：DTOS + trait 契约面，完整可用
- **所有 7 个 storage adapter**：生产默认客户端已落地（#188-#190），live 已验
- **resiliencx**：熔断/限流/舱壁/重试，核心逻辑就绪（需墙钟驱动）
- **kernel**：错误分类 + 关停信号 + 时钟注入，生产可用

### 与 SSOT 镜像的关系

- binancex/okxx SSOT 包含完整 spec（goal→spec→plan），但**镜像 COMPLETE ≠ 本仓实现完成**
- storage adapter 生产 P0 **已落地**（gap-matrix done），与镜像声明的 Cluster/JetStream/EOS 全量仍有差距（DEFER）
- evidence 的 policy 诚实标注 "dev-only"，不虚构能力

### 后续建议

- **R8（跨 crate 集成风险）**应验证 contracts→binancex、resiliencx→observex、bootstrap 组合根等依赖链
- **R9（DEFER 复查）**应审视 evidence 合规升级、schedulex 真实调度器等延期项优先级
- **exchange adapter** 是量化交易路径的**唯一硬阻塞**，建议在 PR 队列中提升优先级
