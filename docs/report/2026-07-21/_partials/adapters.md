# Adapters 生产就绪 partial（只读审计）

| 字段 | 值 |
|------|-----|
| 审计日期 | 2026-07-21 |
| 范围 | STATUS.md 全部 **adapter** 模块（Exchange 2 + Storage 7） |
| 源权威 | `crates/adapters/**`、`crates/contracts`、`docs/ssot/adapters-ssot-alignment.md`、`STATUS.md` |
| 审计性质 | **只读**；本文件为 production readiness 报告 partial，**不是** ship 签字 |
| 总判定 | **9/9 均不可作为生产应用对真实后端的依赖**（内存 scaffold / 进程内 mock；无真实 client SDK） |

> STATUS 完成度（83–89%）是**布局·测试·源码量**结构分，**不是** Production Ready。  
> SSOT 对齐文明确：**未**宣称业务实现 / package stable / ship / Production Ready。

---

## 1. 总表

| 包名 | 路径 | STATUS 成熟度 | 完成度 | 实现深度 | contracts trait | 真实后端 | 认证/TLS/重试/超时 | 生产判定 | 主要缺口 |
|------|------|---------------|--------|----------|-----------------|----------|-------------------|----------|----------|
| `binancex` | `crates/adapters/exchange/binance` | `scaffold+mock` | 89% | 内存 Venue + 可选 `HttpDriver` mock 路径；无协议解析 | `VenueAdapter` + 能力拆分 5 trait | **否** | 无签名/API Key；超时仅映射 `TransportError`；无 TLS 配置面；无重试 | **不可用** | 真实 HTTP/WS 协议、HMAC 签名、下单/行情解析、限流、live 集成测 |
| `okxx` | `crates/adapters/exchange/okx` | `scaffold+mock` | 89% | 同 binance 模式（OKX path 占位） | 同上 | **否** | 同上 | **不可用** | 同上（OKX REST/WS + passphrase 等） |
| `clickhousex` | `crates/adapters/storage/clickhouse` | `scaffold` | 83% | pure scaffold：`Vec` 记 sink | `AnalyticsSink` | **否** | 无 | **不可用** | HTTP/native client、batch insert、schema、mock 验证入口 |
| `kafkax` | `crates/adapters/storage/kafka` | `scaffold+mock` | 89% | 内存 topic map + `MockKafkaBus` | `EventBus` | **否** | 无 broker/TLS/SASL/ack | **不可用** | rdkafka/等客户端、consumer group、offset、投递语义 |
| `natsx` | `crates/adapters/storage/nats` | `scaffold+mock` | 88% | 内存 + `MockNatsBus` | `EventBus` | **否** | 无 | **不可用** | async-nats、JetStream、认证 |
| `ossx` | `crates/adapters/storage/oss` | `scaffold` | 83% | pure scaffold：HashMap 对象 | `ObjectStore` | **否** | 无 | **不可用** | S3/OSS SDK、凭证、multipart、mock 入口 |
| `postgresx` | `crates/adapters/storage/postgres` | `scaffold+mock` | 89% | 内存 Repository + `ObservingPostgresAdapter` commit 边界 | `Repository` + `TxRunner`/`TxContext` | **否** | 无连接池/TLS/SQL | **不可用** | sqlx/tokio-postgres、真实事务、迁移、连接管理 |
| `redisx` | `crates/adapters/storage/redis` | `scaffold+mock` | 89% | 内存 KV（scaffold 忽略 TTL）+ `MockRedisAdapter` TTL 模拟 | `KeyValueStore` + `PubSub` | **否** | 无 | **不可用** | redis crate、真实 TTL/PubSub、集群/Sentinel |
| `taosx` | `crates/adapters/storage/taos` | `scaffold+mock`\* | 88% | pure 内存 `TimeSeriesStore`（**无**独立 `Mock*` 类型） | `TimeSeriesStore` | **否** | 无 | **不可用** | TDengine client、时间线/压缩、mock 命名入口 |

\* **STATUS 标签 vs 源码**：`taosx` 生成器标 `scaffold+mock`，但源码仅有 `TaosAdapter` 内存实现、无 `mock.rs`/Mock 类型；对齐文写 **pure scaffold** 更准确。`clickhousex`/`ossx` 为 pure scaffold 一致。

### 依赖事实（Cargo.toml）

| 包 | 生产依赖 | **无** 真实后端 crate |
|----|----------|----------------------|
| `binancex` / `okxx` | `async-trait`, `bytes`, `futures-*`, `canonical`, `contracts`, `decimalx`, `kernel`, `transportx` | 无 `reqwest` 直接依赖（可选注入 `transportx::HttpDriver`；默认无驱动） |
| storage 七包 | `async-trait` + `contracts` + `kernel`（+ 部分 `bytes`/`futures`/`canonical`） | **零** `sqlx` / `redis` / `rdkafka` / `async-nats` / `clickhouse` / `aws-sdk` / `taos` 等 |

全部 `publish = false`，workspace version `0.3.0`（adapters）；**不可** crates.io 消费叙事。

---

## 2. Exchange 专节

### 2.1 共同模式

- **类型**：`BinanceAdapter` / `OkxAdapter`
  - 字段：`name`, `base_url`, `connected: AtomicBool`, `http: Option<Arc<dyn HttpDriver>>`
  - 预设：`testnet()`/`mainnet()`（binance）、`demo()`/`mainnet()`（okx）——**仅字符串 URL 占位**，不建立连接
- **`connect`/`disconnect`**：翻转进程内标志；**不**握手交易所
- **业务方法**（`place_order`、行情 subscribe、余额/仓位等）：未注入 HTTP 时返回占位值或空 stream；`place_order` 本地回 `OrderAck{Open}`
- **结构化 cancel/query**（覆盖 `VenueAdapter` additive default，满足 CT-10 / `venue_override_gate`）：
  - 无 HTTP：内存路径直接 `Ok` / `OrderStatus::Open`
  - 有 `with_http`：拼 path → `HttpDriver` GET/POST；body 做**极简**字符串匹配（Canceled/Filled）；**非**完整 JSON 协议
- **能力拆分**：均实现 `MarketDataSource` / `ExecutionVenue` / `AccountSource` / `InstrumentCatalog` / `VenueTimeSource`（委托 `VenueAdapter`）
- **README 声明**：**非真实 HTTP**；不宣称 package stable

### 2.2 binancex

| 项 | 证据 |
|----|------|
| 入口 | `src/lib.rs` → `adapter::{BinanceAdapter, AdapterState, Candle, Timeframe}` |
| LOC | ~611（STATUS 613） |
| 测试 | 单元全绿（Mock HTTP cancel/query/**parsed** server_time） |
| 集成测试 | `tests/live_server_time.rs`（`#[ignore]`；公共 REST） |
| 扩展 | `fetch_candles` venue 扩展 DTO，非 contracts 面 |
| `server_time` | **有** `parse_binance_server_time`；live 入口 ignore；非业务协议 |
| 认证 | **无** API key / HMAC / 请求头签名 |
| TLS | 不在本 crate 配置；依赖注入的 `HttpDriver`（如 `ReqwestHttpDriver`） |
| 重试 | 无 adapter 级重试；`TransportError::RateLimited` → `XError::transient` |

### 2.3 okxx

| 项 | 证据 |
|----|------|
| 入口 | `src/lib.rs` → `adapter::{OkxAdapter, AdapterState}` |
| LOC | ~477（STATUS 479） |
| 测试 | 单元全绿（Mock HTTP cancel/query/**parsed** server_time） |
| 集成测试 | `tests/live_server_time.rs`（`#[ignore]`） |
| 与 binance 差异 | path/venue_id 不同；无 candles 扩展；体量更薄 |
| 认证 | **无** OK-ACCESS-* 头 / passphrase |

### 2.4 Exchange 生产判定

| 问题 | 结论 |
|------|------|
| 能否作为「生产交易应用」依赖？ | **不能** |
| 能否对接真实 Binance/OKX **业务**？ | **不能**（无签名、无下单协议） |
| 只读 `server_time` | **可以**（公共 REST + 真 `HttpDriver`；默认 ignore；CI 不挡 PR） |
| contracts 接线是否有价值？ | **有**（trait 形状 + override 门禁 + mock + 时间解析） |
| 下一步 | 签名 / 私有 API / testnet 业务读路径另战役 |

---

## 3. Storage 专节

### 3.1 First-batch（有 mock 验证入口）

| 包 | scaffold 类型 | mock 验证入口 | 证明点 | 真实 I/O |
|----|---------------|----------------|--------|----------|
| `postgresx` | `PostgresAdapter`：HashMap + `FakeTxContext`（begin_tx 无真实 staged） | `ObservingPostgresAdapter` / `MockPostgresBackend` + `MockTxContext` | staged 仅 commit 后可见；rollback 丢弃；可观察 commit/rollback 计数；`dyn TxRunner` | **无** |
| `redisx` | `RedisAdapter`：KV **忽略 TTL** + 简易 PubSub | `MockRedisAdapter` + **`RedisLiveKv`（feature `live`）** | mock TTL/PubSub；**live** 真 Redis KV（`live_kv_conformance`） | **有（KV live）** |
| `kafkax` | `KafkaAdapter`：per-topic 下标 id | `MockKafkaBus` | 跨 topic 全局单调 id；`dyn EventBus` | **无** |
| `natsx` | `NatsAdapter`：与 kafka 同构内存 bus | `MockNatsBus` | 同上 | **无** |

订阅语义共性：`subscribe`/`sub_channel` 返回**当前快照** `stream::iter`，**非**持续推送 / 无 redelivery / 无 consumer group。与 contracts 文档 at-most-once 最小面一致，**远不足**生产消息系统。

### 3.2 Pure scaffold（无独立 mock 模块）

| 包 | 类型 | trait | 行为 | 测试 |
|----|------|-------|------|------|
| `clickhousex` | `ClickHouseAdapter` | `AnalyticsSink` | `events: Vec<(name, Bytes)>` | 1 测 |
| `ossx` | `OssAdapter` | `ObjectStore` | HashMap put/get | 2 测 |
| `taosx` | `TaosAdapter` | `TimeSeriesStore` | 按 table 存 `Tick` + 时间过滤 | 1 测 |

三者 **endpoint 字符串仅为元数据**（如 `http://127.0.0.1:8123`），从不拨号。

### 3.3 Storage 生产判定

| 问题 | 结论 |
|------|------|
| 能否作为生产 DB/MQ/对象存储依赖？ | **默认不能**；`redisx` live KV 仅作验证/有限 KV 面 |
| mock 是否替代集成测？ | **否**；mock 验证语义；live 另 feature + optional CI |
| 默认 CI | 离线绿灯；`redisx-live.yml` 在 PR path 上跑 service；exchange live 仅 dispatch |

---

## 4. 与 contracts（L3 / trait 出口）的依赖关系

```text
应用 / bootstrap
      │
      ▼
 contracts  (package name: contracts；文档亦称 xhyper-contracts)
   ├── VenueAdapter / ExecutionVenue / MarketDataSource / …
   ├── KeyValueStore / PubSub / EventBus
   ├── Repository / TxRunner / TxContext / run_tx_commit_on_ok
   ├── TimeSeriesStore / ObjectStore / AnalyticsSink
   └── Instrumentation（observex 实现；非 adapter）
      ▲
      │  impl（当前均为内存 scaffold / mock）
 adapters/*  ──►  kernel + (exchange: transportx/canonical/decimalx)
```

| 关系点 | 状态 |
|--------|------|
| adapters 依赖 `contracts` path | **是**（9 包均声明） |
| 是否实现目标 trait | **是**（签名级 scaffold / mock；redis live 为真路径） |
| `ExecutionVenue` 推荐生产入口 | binance/okx **已**实现；业务语义仍为占位 |
| Venue override 门禁 | `venue_override_gate` + binancex/okxx |
| contracts 自身生产就绪 | **L3 子集** KV+Instr；整体 **非** PR（CT-9 部分） |
| 反向依赖 | 禁止 kernel/types 依赖 adapters——当前满足 |

**生产应用若 `use redisx::RedisAdapter` 等：得到的是进程内 HashMap，不是 Redis。** 这是最危险的误用点：类型名像生产客户端，行为是测试桩。

---

## 5. 认证 / 重试 / 超时 / TLS（横切）

| 能力 | Exchange | Storage |
|------|----------|---------|
| 认证（API Key / DB 用户 / SASL / 云凭证） | **无** | **无** |
| TLS 校验 / mTLS | **无**（本层） | **无** |
| 连接超时 / 读超时 | 仅映射 `transportx::TransportError` 变体（exchange）；storage 无 | **无** |
| 重试 / 熔断 | **无** adapter 级（应组合 `resiliencx`，当前未接） | **无** |
| 限流处理 | RateLimited → `XError::transient` 映射而已 | **无** |
| 优雅关闭 / 连接池 | connect 标志位 | **无** |

---

## 6. 抽样测试证据（本会话）

命令：

```bash
cargo test -p binancex -p okxx -p redisx -p postgresx -p kafkax --all-targets
# 补充：cargo test -p natsx -p clickhousex -p ossx -p taosx --all-targets
```

| 包 | 结果（摘要） |
|----|----------------|
| `binancex` | **13 passed** |
| `okxx` | **9 passed** |
| `kafkax` | **5 passed** |
| `postgresx` | **10 passed** |
| `redisx` | **7 passed** |
| `natsx` | **4 passed** |
| `clickhousex` | **1 passed** |
| `ossx` | **2 passed** |
| `taosx` | **1 passed** |

- 失败 / ignored：**0**
- 含义：编译与**离线 mock/scaffold 语义**健康；**不**证明任何真实后端可用性

---

## 7. P0 阻断（若有人宣称「adapter 生产可用」）

下列任一为真即 **BLOCK** 生产宣称：

1. **默认无真实后端客户端**：多数包无 SDK；**例外** `redisx` feature `live` → `RedisLiveKv`（#168）。
2. **无认证面**：交易所签名、DB/MQ/云凭证均缺失（exchange 仅公共只读 time）。
3. **无完整生产 I/O 路径**：Exchange 默认内存；HTTP 可注入驱动；业务协议未做。
4. **live 证据有限**：默认离线绿；`redisx-live` optional CI；exchange `live_server_time` ignore + **workflow_dispatch only**；业务集成仍空。
5. **语义与生产不符**（默认 scaffold）：
   - Postgres `begin_tx` → `FakeTxContext`（mock 才有 staged）。
   - Redis **scaffold** 忽略 TTL（live KV 另路径）。
   - EventBus subscribe 为一次性快照，非实时流。
6. **命名陷阱**：`*Adapter` / `mainnet()` 像生产客户端——README 红线（s9t.14）。
7. **SSOT / STATUS 误读**：镜像 COMPLETE 或 STATUS ~89% **不等于** ship。

**当前正确对外叙事**：  
「adapters = contracts 的 **scaffold/mock** 为主；**有限真路径** = redis live KV + exchange 公共 `server_time`；**非** Production Ready 业务适配器。」

---

## 8. 推荐路线：谁先做真实验证入口

原则：**一域一战役**；mock → `#[ignore]` 集成测 → 可选 `live` feature → 证据包；禁止九包并行真实现。

| 优先级 | 包 | 理由 | 建议最小真实验证入口 |
|--------|-----|------|----------------------|
| **DONE** | `redisx` live KV | #168 · feature `live` + conformance + optional CI | 扩展 TTL/PubSub live 另战役 |
| **DONE** | `binancex`/`okxx` 只读 time | #172 · parse + ignore live；dispatch workflow | 签名 / exchangeInfo / 私有 API 另战役 |
| **P1** | `postgresx` live Tx | mock 最完整；contracts `run_tx_commit_on_ok` 就绪 | Docker Postgres；begin/commit/rollback 对照 |
| **P3** | `kafkax` / `natsx` | EventBus mock 可用；运维成本高于 Redis | 本地 container；至少 once publish→subscribe 往返 + 失败注入 |
| **P4** | `clickhousex` / `ossx` / `taosx` | pure scaffold，先补 **命名 Mock\*** 与 contracts 深度，再接 SDK | 先 DEFER-1 式 mock 入口，再 live |

**横切前置（所有 live 战役共用）**：

1. 显式 feature：`live` / `integration` 默认关闭。  
2. 凭证仅 env / secret provider；禁止入库。  
3. 组合 `resiliencx`（超时·重试·熔断）与 `observex`（span），勿在 adapter 内重写。  
4. Exchange：**禁止**在无人工门禁时对 mainnet 写路径做 CI。  
5. 文档与 STATUS：引入真实 I/O 后成熟度标签应新增 `partial-live` 类，避免继续只靠布局分。

---

## 9. 结论一句话

**STATUS 上 adapters「看起来齐」；源码上全部是 contracts 接线用的内存桩 / mock。**  
作为「生产应用依赖」连接真实交易所或存储——**9/9 不可用**。  
可保留价值：trait 形状冻结、first-batch mock 语义、Venue override 门禁、离线 CI 绿灯。  
下一跳：按 §8 优先 `postgresx`/`redisx` 真后端集成入口，exchange 仅 testnet 只读。

---

## 10. 证据索引

| 资源 | 路径 |
|------|------|
| STATUS | `/home/workspace/infra.rs/STATUS.md`（或 worktree 内同名） |
| SSOT 对齐 | `docs/ssot/adapters-ssot-alignment.md` |
| contracts 对齐 | `docs/ssot/contracts-ssot-alignment.md` |
| mock workflow | `.github/workflows/contracts-live.yml`（offline mock） |
| Venue 门禁 | `crates/contracts/tests/venue_override_gate.rs` |
| 本 partial | `docs/report/2026-07-21/_partials/adapters.md`（本文件） |
