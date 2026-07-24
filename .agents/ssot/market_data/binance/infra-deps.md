# Binance 基础设施依赖映射

> 记录 market_data（Binance 适配器）对 infra.rs **本仓内**公共基础模块的依赖关系。
> infra.rs 即本仓自身（非外部仓库）：下列模块均为 workspace member，以 **intra-workspace path 依赖**复用，**禁止**以 git/tag 外部依赖形式引入。
> 最后更新: 2026-07-24
> 审查来源: R3 (infra.rs 依赖可行性审查)

## 核心基础模块

| infra.rs Crate | 版本 | 用途 | 引入阶段 |
|---------------|------|------|---------|
| kernel | v0.3.1 | 时间/生命周期 (Clock, Lifecycle) | Phase 1 |
| configx | v0.1.2 | 配置管理 (MemoryConfigStore) | Phase 1 |
| resiliencx | v0.1.2 | 重试 (retry_async), 熔断器 (CircuitBreaker), 限流 (TokenBucket - 待提交) | Phase 1 |
| contracts | v0.1.4 | R4 trait 出口层 (跨 crate 契约) | Phase 2 |
| testkit | v0.1.3 | 确定性测试支持 (ManualClock) | Phase 1 (dev) |

## 数据存储适配器（优先复用本仓现有模块）

> 下列 7 个适配器均为 infra.rs 本仓 workspace member（路径 `crates/adapters/storage/<name>`）。
> market_data 构建**薄领域包装层**（见 `design/design.md` ADR-001），将 MarketEvent 编码为各适配器期望格式，**不重新实现存储连接**。
> 完成度为结构进度（默认客户端入口已落地）；package stable / Cluster·EOS / 部分 live 证据仍 OPEN，详见 [adapters-ssot-alignment](../../../../docs/ssot/adapters-ssot-alignment.md)。

| infra.rs Crate | 版本 | 存储后端 | 协议 | 结构状态 |
|---------------|------|---------|------|---------|
| kafkax | v0.4.0 | Apache Kafka | Kafka binary (rskafka) | 默认客户端已落地；native EOS NO-GO |
| redisx | v0.3.15 | Redis | RESP (redis-rs) | 全公开 API + live/E2E；Cluster/Sentinel/TLS live OPEN |
| natsx | v0.3.5 | NATS | NATS protocol (async-nats) | Core/JetStream；断线窗口无回放、Cluster/HA NO-GO |
| ossx | v0.4.1 | S3/MinIO | S3 REST (object_store) | 有界 multipart/retry/orphan；dev live PASS |
| postgresx | v0.3.13 | PostgreSQL | PostgreSQL wire (tokio-postgres) | Pool/Tx/COPY/Migrator/mTLS；package stable OPEN |
| taosx | v0.3.10 | TDengine | TDengine REST/Schemaless | Production-default 全 API + gap-zero register |
| clickhousex | v0.3.6 | ClickHouse | ClickHouse HTTP | HTTP(S)+PEM CA+insert_batch；真实集群 TLS OPEN |

## 交易所适配器（本仓已有）

| infra.rs Crate | 版本 | 状态 |
|---------------|------|------|
| binancex | v0.3.2 | 签名 REST + 公共 WS 解析/注入；交易 **NO-GO** |

> 决策待定: exchange-binance 是（A）基于 binancex 构建薄包装层，还是（B）独立实现。
> 如选 A，sink 策略自动匹配（直接复用 infra.rs sink 能力）；如选 B，需要薄包装层将域类型适配至 infra.rs sink。

## 引入方式（intra-workspace path 依赖）

infra.rs 是本仓 workspace 自身；下列模块与 `crates/market_data` 同处一个 workspace，按本仓约定用 **path 依赖**复用（package 名无 `xhyper-` 前缀；intra-workspace 允许内联 version）：

```toml
# crates/market_data/Cargo.toml（或 sink 包装层 crate）
# —— 核心基础模块 ——
kernel      = { path = "../kernel",                version = "0.3.1" }
configx     = { path = "../infra/configx",         version = "0.1.2" }
resiliencx  = { path = "../infra/resiliencx",      version = "0.1.2" }
contracts   = { path = "../contracts",             version = "0.1.4" }
testkit     = { path = "../testkit",               version = "0.1.3" }   # [dev-dependencies]

# —— 数据存储/消息适配器（优先复用现有模块，薄包装层）——
kafkax      = { path = "../adapters/storage/kafka",      version = "0.4.0" }
redisx      = { path = "../adapters/storage/redis",      version = "0.3.15" }
natsx       = { path = "../adapters/storage/nats",       version = "0.3.5" }
ossx        = { path = "../adapters/storage/oss",        version = "0.4.1" }
postgresx   = { path = "../adapters/storage/postgres",   version = "0.3.13" }
taosx       = { path = "../adapters/storage/taos",       version = "0.3.10" }
clickhousex = { path = "../adapters/storage/clickhouse", version = "0.3.6" }
```

> 第三方依赖（非本仓 crate）仍须 `{ workspace = true }` 引用根 `[workspace.dependencies]` 声明（见根 `AGENTS.md` 依赖集中管理）。

## 本仓 follow-up

| Issue | 模块 | 需求 | 优先级 |
|-------|------|------|:---:|
| INFRA-001 | resiliencx | TokenBucket 限流器 | P0 |
| INFRA-002 | kernel / schedulex | 通用 BatchCollector<T> | P1 |
| INFRA-003 | decimalx | round_to() / truncate_to() 精度辅助 | P2 |

> 上述为本仓内部增强需求（非向外部仓库提 Issue）；落地后同步本表与对应 design/spec。
