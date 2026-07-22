# 存储适配器生产就绪性分析报告

> **日期:** 2026-07-21 | **审计:** infra.rs CI | **范围:** crates/adapters/storage/{redis,kafka,nats,postgres,taos,oss,clickhouse}

## 1. 总览

7 个存储适配器 crate 依据 §2 中定义的生产就绪标准进行评估。
总实现量：6,866 LOC，覆盖 7 个 crate。全部 7 个均有生产级实现（非 scaffold），但就绪程度差异明显。

| Crate | LOC | 文件数 | Mock | Pool | Config | 测试文件 | 生产就绪？ |
|-------|----:|:----:|:----:|:----:|:------:|:---:|:--:|
| **postgresx** | 1,693 | 9 | ✅ | ✅ | ✅ | 1 | **是** |
| **redisx** | 1,509 | 7 | ✅ | ✅ | ✅ | 2 | **接近** |
| **kafkax** | 1,041 | 10 | ✅ | ✅ | ✅ | 1 | **接近** |
| **taosx** | 755 | 4 | ❌ | ❌ | ✅ | 1 | **部分** |
| **ossx** | 693 | 5 | ❌ | ❌ | ✅ | 1 | **部分** |
| **natsx** | 666 | 6 | ✅ | ✅ | ✅ | 1 | **接近** |
| **clickhousex** | 509 | 4 | ❌ | ❌ | ✅ | 1 | **部分** |

**总体评估：** 1 个生产可用，3 个接近就绪，3 个部分就绪。尚无 crate 在所有标准上达到生产稳定。

## 2. 生产就绪标准

一个 crate 被认为**生产就绪**需满足以下全部条件：

| # | 标准 | 定义 |
|---|---------|-------------|
| P1 | **Trait 实现** | 实现 contracts 中对应的 trait，全部方法完整 |
| P2 | **错误处理** | 使用 `thiserror` 类型化错误，无裸 `unwrap()`/`expect()` |
| P3 | **配置管理** | 通过 `FOUNDATIONX_*` 环境变量驱动的配置 |
| P4 | **连接池** | 生产级连接池，带健康检查 |
| P5 | **Mock/Fake** | 内存级 mock 实现，支持离线测试 |
| P6 | **集成测试** | `tests/` 目录内含实际服务连通性测试 |
| P7 | **文档** | `docs/README.md` 含 API 文档、配置参考、迁移指南 |
| P8 | **TLS/SSL** | 加密连接支持，非默认关闭 |
| P9 | **重试/弹性** | 集成 `resiliencx` 实现重试/熔断/限流 |
| P10 | **CHANGELOG** | 按 Keep a Changelog 维护的版本发布历史 |

## 3. 逐 Crate 分析

### 3.1 postgresx (1,693 LOC) — 生产可用 🟢

**Contract:** `Repository<T, Id>` + `TxContext` + `TxRunner`

| 文件 | LOC | 用途 |
|------|----:|---------|
| `config.rs` | 431 | 连接池配置、认证、SSL |
| `pool.rs` | 280 | Deadpool 管理的连接池，带健康检查 |
| `mock.rs` | 279 | `ObservingPostgresAdapter` — 提交边界 mock，���暂存写入和 observable 机制 |
| `error.rs` | 199 | SQLSTATE → `kernel::ErrorKind` 映射 |
| `tx.rs` | 158 | `PgTransaction`、`TxState`（Active/Committed/RolledBack） |
| `adapter.rs` | 117 | `PostgresAdapter`（scaffold） |
| `runner.rs` | 100 | 生产级 `PgTxRunner` — 实现 `TxRunner`，含真实 BEGIN/COMMIT/ROLLBACK 边界 |
| `conn.rs` | 63 | 连接封装 |
| `lib.rs` | 66 | 模块根、重导出 |

> **注意：** 生产级 `Repository<T,Id>` trait 仅在 scaffold 类型上实现（`PostgresAdapter`、`ObservingPostgresAdapter`）。生产路径（`PostgresPool`）提供直接 SQL 执行，不经过 Repository 抽象。这是故意的设计取舍（底层池更灵活），但意味着 contracts trait 在生产代码中未实现。

**就绪度:** P1⚠️ P2✅ P3✅ P4✅ P5✅ P6✅ P7✅ P8⚠️ P9❌ P10✅

| 差距 | 严重性 | 行动 |
|-----|--------|------|
| Repository trait ���进入生产 | 中 | 在 PostgresPool 上实现 Repository，或明确记录为有意不实现 |
| 缺少 TLS 强制 | 中 | 生产配置中 SSL mode 应设为 `require` 或 `verify-full` |
| 无重试集成 | 中 | 用 resiliencx 重试策略包裹池操作 |
| 仅 1 个集成测试 | 低 | 增加事务边界测试、连接失败测试 |

### 3.2 redisx (1,509 LOC) — 接近生产 🟡🟢

**Contract:** `KeyValueStore` + `PubSub`

| 文件 | LOC | 用途 |
|------|----:|---------|
| `config.rs` | 469 | Redis 集群/哨兵配置（`RedisConfig`、`RedisConfigBuilder`、`RedisMode`） |
| `pool.rs` | 280 | `RedisPool` — `ConnectionManager` + Semaphore 背压控制 |
| `scaffold.rs` | 236 | `RedisAdapter`/`InMemoryRedis` + `MockRedisAdapter`（TTL 模拟） |
| `client.rs` | 223 | **生产级** `RedisClient` — 实现 `KeyValueStore` + 扩展 API |
| `pubsub.rs` | 163 | `RedisPubSub`、`RedisPubSubFacade`（feature `pubsub`） |
| `error_map.rs` | 96 | Redis 错误 → `kernel::ErrorKind` 映射 |
| `lib.rs` | 42 | 模块根、重导出 |

**就绪度:** P1✅ P2✅ P3✅ P4✅ P5✅ P6✅ P7✅ P8⚠️ P9❌ P10✅

| 差距 | 严重性 | 行动 |
|-----|--------|------|
| Scaffold 模块泄漏到生产 | 中 | 加固 `scaffold` feature gate |
| 无重试集成 | 中 | 为 Redis 连接失败添加断路器 |
| 生产配置构建器中无 TLS 选项 | 中 | 添加 SSL/TLS 选项以支持 Redis 6+ 加密连接 |
| PubSub 在 feature gate 后 | 低 | 量化交易场景建议默认启用 pubsub |

### 3.3 kafkax (1,041 LOC) — 接近生产 🟡🟢

**Contract:** `EventBus`

| 文件 | LOC | 用途 |
|------|----:|---------|
| `pool.rs` | 239 | Producer/Consumer 池 |
| `config.rs` | 182 | SASL/SSL broker 配置 |
| `mock.rs` | 110 | `MockKafkaBus` |
| `consumer.rs` | 99 | Consumer group 管理 |
| `bus.rs` | 93 | **生产级** `KafkaEventBus` — 实现 `EventBus`（at-most-once） |
| `adapter.rs` | 78 | `KafkaAdapter`（scaffold） |
| `message.rs` | 79 | `KafkaMessage`、`Delivery`、`encode_bus_id`/`parse_bus_id` |
| `producer.rs` | 66 | `KafkaProducer` — 带 broker ack 的发布 |
| `error_map.rs` | 52 | 错误翻译 |
| `lib.rs` | 43 | 模块根 |

**就绪度:** P1✅ P2✅ P3✅ P4✅ P5✅ P6✅ P7✅ P8⚠️ P9❌ P10✅

| 差距 | 严重性 | 行动 |
|-----|--------|------|
| **无消费者 offset 管理** | **高** | 实现 offset commit/reset 以支持 at-least-once 投递 |
| 无重试集成 | 中 | broker 连接失败时增加指数退避 |
| SASL_SSL 未强制 | 中 | 当前配置偏好 PLAINTEXT，生产需强制 SASL_SSL |
| Mock 不支持分区感知 | 低 | 为 MockEventBus 增加分区感知 |

### 3.4 taosx (755 LOC) — 部分就绪 🟡

**Contract:** `TimeSeriesStore`

| 文件 | LOC | 用途 |
|------|----:|---------|
| `client.rs` | 468 | **生产级** `TaosPool` — 实现 `TimeSeriesStore`，REST 客户端（port 6041），stable 管理，精度自动检测 |
| `config.rs` | 186 | `TaosConfig`、`TsPrecision` |
| `adapter.rs` | 82 | `TaosAdapter`（scaffold）— 内存级 `TimeSeriesStore` |
| `lib.rs` | 19 | 最小根 |

所有 7 个适配器中**生产客户端最深** — 仅 client.rs 就有 468 LOC。处理 stable 自动创建、精度自动检测、多表批量 INSERT、表不存在时优雅恢复。

**就绪度:** P1✅ P2❌ P3✅ P4❌ P5❌ P6✅ P7✅ P8❌ P9❌ P10✅

| 差距 | 严重性 | 行动 |
|-----|--------|------|
| **无 mock** | **高** | 为可测试性提供 `MockTaos` |
| **无连接池** | **高** | 添加连接池（TDengine 原生连接开销大） |
| **无错误类型化** | **高** | 用 `thiserror` 类型化错误替换裸 result 类型 |
| 单文件 client.rs 过大（468 LOC） | 中 | 拆分子模块：connection、query、write |
| 无 TLS 支持 | 低 | 为 TDengine 3.x 加密模式添加 TLS 配置 |

**量化交易说明：** TDengine 是行情数据（trades、order books、klines）的关键时序存储。缺少连接池意味着高频入库在生产环境中不可行。

### 3.5 ossx (693 LOC) — 部分就绪 🟡

**Contract:** `ObjectStore`

| 文件 | LOC | 用途 |
|------|----:|---------|
| `client.rs` | 311 | **生产级** `OssClient` — 实现 `ObjectStore`，OSS Signature V1 签名，virtual-host style，put/get/delete |
| `config.rs` | 187 | `OssConfig`、`OssConfigBuilder`、环境变量 |
| `sign.rs` | 94 | OSS V1 签名（`sign_v1`、`authorization_header`、`canonicalized_resource`） |
| `adapter.rs` | 73 | `OssAdapter`（scaffold）— 内存级 `ObjectStore` |
| `lib.rs` | 28 | 模块根 |

**就绪度:** P1✅ P2❌ P3✅ P4❌ P5❌ P6✅ P7✅ P8✅ P9❌ P10✅

| 差距 | 严重性 | 行动 |
|-----|--------|------|
| **无 mock** | **高** | 为可测试性提供 `MockOss` |
| **无重试** | **高** | HTTP 客户端无重试 — OSS 容易出现瞬时故障 |
| 无上传进度跟踪 | 中 | 大文件上传需要进度/校验和验证 |
| 硬编码 HMAC-SHA1 签名 | 中 | 提取为可拔插签名器以支持其他 OSS providers（AWS S3、GCS） |
| 单文件 client.rs | 低 | 拆分为 get、put、delete、list 操作 |

**量化交易说明：** OSS 存储历史行情数据快照、模型检查点和审计工件。无重试意味着大文件上传会因瞬时网络错��而失败。

### 3.6 natsx (666 LOC) — 接近生产 🟡🟢

**Contract:** `EventBus` + `PubSub`

| 文件 | LOC | 用途 |
|------|----:|---------|
| `pool.rs` | 256 | NATS 连接池 |
| `config.rs` | 141 | Server URL/auth 配置 |
| `mock.rs` | 96 | `MockNatsBus` |
| `adapter.rs` | 78 | `NatsAdapter`（scaffold） |
| `bus.rs` | 61 | **生产级** `NatsEventBus` — 实现 `EventBus`（at-most-once）；`NatsPool` 也直接实现 `EventBus` |
| `lib.rs` | 34 | 模块根 |

**就绪度:** P1✅ P2✅ P3✅ P4✅ P5✅ P6✅ P7✅ P8❌ P9❌ P10✅

| 差距 | 严重性 | 行动 |
|-----|--------|------|
| **无 TLS** | **高** | 为 NATS 2.x 加密连接添加 TLS 配置 |
| **无 JetStream 支持** | **高** | Core NATS 是 at-most-once；量化交易需 JetStream 实现持久化 |
| 无重试集成 | 中 | NATS 集群故障转移时增加带退避的重连 |
| Mock 不模拟延迟 | 低 | 为 MockNats 增加可配置延迟 |

**量化交易说明：** NATS 是交易服务间的消息骨干。无 JetStream（持久流）意味着在途订单和行情数据在重启时可能丢失。这是阻塞生产的差距。

### 3.7 clickhousex (509 LOC) — 部分就绪 🟡

**Contract:** `AnalyticsSink`

| 文件 | LOC | 用途 |
|------|----:|---------|
| `client.rs` | 300 | **生产级** `ClickHousePool` — 实现 `AnalyticsSink`，HTTP 客户端（port 8123），查询/插入/自动建表 |
| `config.rs` | 121 | `ClickHouseConfig` |
| `adapter.rs` | 69 | `ClickHouseAdapter`（scaffold）— 内存级 `AnalyticsSink` |
| `lib.rs` | 19 | 模块根 |

**就绪度:** P1✅ P2❌ P3✅ P4❌ P5❌ P6✅ P7✅ P8✅ P9❌ P10✅

| 差距 | 严重性 | 行动 |
|-----|--------|------|
| **无 mock** | **高** | 为可测试性提供 `MockClickHouse` |
| **无批量插入** | **高** | 逐行插入太慢；需要批量/分块写入 |
| 无连接池 | 中 | 通过 keep-alive 池实现 HTTP 连接复用 |
| 无重试 | 中 | 为 ClickHouse HTTP 错误增加重试 |

**量化交易说明：** ClickHouse 是交易分析、回测结果和审计日志的 OLAP 层。批量插入是必须的 — 量化交易体量下的单行插入会压垮集群。

## 4. Contract Trait → 适配器映射

| contracts Trait | 适配器 | 实现状态 |
|-----------------|---------|-----------------------|
| `KeyValueStore` | redisx | ✅ 基础 get/set 已实现 |
| `EventBus` | kafkax | ✅ publish/subscribe，含 mock |
| `EventBus` | natsx | ✅ publish/subscribe，含 mock |
| `Repository<T, Id>` | postgresx | ⚠️ 仅 scaffold 实现 |
| `TxContext` / `TxRunner` | postgresx | ✅ 事务支持，含 mock |
| `TimeSeriesStore` | taosx | ✅ REST 客户端全量实现 |
| `ObjectStore` | ossx | ✅ get/put/delete 操作 |
| `AnalyticsSink` | clickhousex | ✅ HTTP 客户端写入 |
| `PubSub` | redisx / natsx | ⚠️ Redis pubsub 部分，NATS pubsub 通过 EventBus |

## 5. 量化交易应用场景

### 5.1 行情数据链路

```text
Exchange → kafkax/natsx (EventBus) → taosx (TimeSeriesStore) → postgresx (Repository)
                                                                    ↓
                          redisx (KeyValueStore ← cache)      clickhousex (AnalyticsSink)
```

| 流程 | 适配器 | 当前 | 需要 |
|------|--------|---------|----------|
| Tick → 入库 | kafkax | ✅ 仅 Publish | Subscribe + offset 管理 |
| Tick → 存储 | taosx | ⚠️ 单连接 | 连接池 + 批量写入 |
| Tick → 缓存 | redisx | ✅ Get/Set | PubSub 实时推送 |
| Tick → 查询 | postgresx | ✅ Repository | 事务边界 |
| Tick → 分析 | clickhousex | ⚠️ 单行插入 | 批量分块插入 |

### 5.2 交易关键差距

| 差距 | 影响 | 影响范围 |
|-----|--------|-------------------|
| **无连接池** | > 100 conn/s 无法处理 | taosx, ossx, clickhousex |
| **无 mock** | 离线不可测试 | redisx 已解决, taosx/ossx/clickhousex 缺失 |
| **无重试/断路器** | 瞬时故障级联扩散 | 全部 7 个适配器 |
| **NATS 无 JetStream** | 重启时在途数据丢失 | natsx |
| **无批量写入** | 单行瓶颈 | clickhousex, taosx |
| **TLS 未强制** | 研发环境凭据暴露 | postgresx, redisx, kafkax, natsx |

## 6. 建议优先级矩阵

### P0 — 阻塞生产（任何生产部署前完成）

1. **增加 mock**：taosx, ossx, clickhousex（3 个 crate，redisx 已有 MockRedisAdapter）
2. **增加连接池**：taosx, ossx, clickhousex（3 个 crate）
3. **增加 NATS JetStream** 支持（natsx）
4. **增加批量写入**：clickhousex 和 taosx

### P1 — 生产加固（稳定生产需要）

1. **集成 resiliencx 重试**：全部 7 个适配器
2. **TLS 强制**：postgresx, redisx, kafkax, natsx
3. **Consumer offset 管理**：kafkax
4. **错误类型化**：taosx, ossx, clickhousex
5. **Repository trait 进入生产**：postgresx（或记录为有意不实现）

### P2 — 生产优化（可维护性改进）

1. 扩展测试覆盖面（每个适配器 3+ 集成测试）
2. 分离 scaffold 模块与生产代码
3. OSS 上传统计跟踪
4. 文档扩展（API 文档、迁移指南）

## 7. 工作量估算

| 阶段 | 任务 | 预估 LOC | 预估天数 |
|-------|-------|:------------:|:--------------:|
| P0（3 mock） | 创建 MockTaos、MockOss、MockClickHouse | ~600 | 2 |
| P0（3 pool） | 为 taosx、ossx 和 clickhousex 添加 pool | ~600 | 2-3 |
| P0（JetStream） | NATS JetStream consumer/producer | ~400 | 1-2 |
| P0（批量） | clickhousex 和 taosx 批量插入 | ~300 | 1-2 |
| P1（弹性） | 7 个适配器重试集成 | ~500 | 2-3 |
| P1（TLS） | 4 个适配器 TLS 强制 | ~200 | 1 |
| **P0-P1 合计** | | **~2,600** | **10-14 天** |

## 8. 结论

存储适配器在 7 个 crate 中累积了 6,866 LOC 的实现代码，进展显著。
PostgreSQL 最成熟（生产可用）。Kafka 和 NATS 接近就绪，但缺少关键特性（offset 管理、JetStream）。
其余 4 个适配器需要 mock 实现、连接池和错误类型化以达到生产就绪。

**量化交易部署的最小可行集：** postgresx + kafkax（含 offset 管理）+ redisx + taosx（含 pool）。
ClickHouse 和 OSS 可在第二波完成。
