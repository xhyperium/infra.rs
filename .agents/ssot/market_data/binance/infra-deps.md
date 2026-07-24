# Binance 基础设施依赖映射

> 记录 market_data.rs (Binance 适配器) 与 infra.rs 公共基础模块之间的依赖关系。
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

## 数据存储适配器

> 以下全部模块在 infra.rs 中已存在（98-100% 完成度）。
> market_data.rs 构建**薄领域包装层**，将 MarketEvent 编码为适配器期望的格式，而非重新实现存储连接。

| infra.rs Crate | 版本 | 存储后端 | 协议 | 状态 |
|---------------|------|---------|------|:---:|
| kafkax | v0.4.0 | Apache Kafka | Kafka binary protocol (rskafka) | 100% |
| redisx | v0.3.15 | Redis | RESP (redis-rs) | 100% |
| natsx | v0.3.5 | NATS | NATS protocol (async-nats) | 100% |
| ossx | v0.4.1 | S3/MinIO | S3 REST (object_store) | 100% |
| postgresx | v0.3.13 | PostgreSQL | PostgreSQL wire (tokio-postgres) | 100% |
| taosx | v0.3.10 | TDengine | TDengine REST/Schemaless | 100% |
| clickhousex | v0.3.6 | ClickHouse | ClickHouse HTTP | 98% |

## 交易所适配器（infra.rs 已存在）

| infra.rs Crate | 版本 | 代码行数 | 状态 |
|---------------|------|---------|:---:|
| binancex | v0.3.2 | 1,965 LOC | 98% |

> 决策待定: exchange-binance 是（A）基于 binancex 构建薄包装层，还是（B）独立实现。
> 如选 A，sink 策略自动匹配（直接使用 infra.rs sink）；如选 B，需要薄包装层将域类型适配至 infra.rs sink。

## 引入方式

```toml
# 根 Cargo.toml [workspace.dependencies]
xhyper-kernel = { git = "https://github.com/xhyperium/infra.rs.git", package = "kernel", tag = "v0.3.18" }
xhyper-configx = { git = "https://github.com/xhyperium/infra.rs.git", package = "configx", tag = "v0.3.18" }
resiliencx = { git = "https://github.com/xhyperium/infra.rs.git", package = "resiliencx", tag = "v0.3.18" }
xhyper-contracts = { git = "https://github.com/xhyperium/infra.rs.git", package = "contracts", tag = "v0.3.18" }
xhyper-testkit = { git = "https://github.com/xhyperium/infra.rs.git", package = "testkit", tag = "v0.3.18" }
kafkax = { git = "https://github.com/xhyperium/infra.rs.git", package = "kafkax", tag = "v0.3.18" }
redisx = { git = "https://github.com/xhyperium/infra.rs.git", package = "redisx", tag = "v0.3.18" }
# ... (其余 sink 同理)
```

> 注意: 目前仅 kernel 有独立 per-crate tag (v0.3.1)。其他 crate 需使用 workspace 级别 tag (v0.3.18)。
> 已向 infra.rs 提交 Issue 请求所有依赖 crate 的 per-crate tag。

## infra.rs 待提交需求

| Issue | 模块 | 需求 | 优先级 |
|-------|------|------|:---:|
| INFRA-001 | resiliencx | TokenBucket 限流器 | P0 |
| INFRA-002 | kernel / schedulex | 通用 BatchCollector<T> | P1 |
| INFRA-003 | decimalx | round_to() / truncate_to() 精度辅助 | P2 |
