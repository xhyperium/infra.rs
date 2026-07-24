# Binance 适配器 — 设计决策

> 来源: 草案 arch1.md + 2.md + 10 轮审查 (R1-R10)

## 已有决策

### D-001: 产品线路由
> 状态: ✓ 已验证 (BN-ROUTE-001)

按 ProductLine 自动推导 REST/WS 基地址:

- Spot → api.binance.com / stream.binance.com
- Future (UM) → fapi.binance.com / fstream.binance.com
- Future (CM) → dapi.binance.com / dstream.binance.com
- Option → eapi.binance.com / estream.binance.com

### D-002: 深度增量序列验证
> 状态: 设计中 (BN-BOOK-001)

Model A (Binance): 快照+增量同步，U/u 连续性校验。
详见: `.agents/ssot/orderbook/spec/spec.md`

### D-003: WebSocket 连接池
> 状态: 设计中 (BN-WS-002)

- 每 ProductLine 一个持久 WS 连接
- 24h 主动重连 (T+23h 触发)
- 重连后自动恢复订阅列表

## 新增决策（审查后）

### ADR-001: Sink 策略 — 薄包装层 + infra.rs 适配器

**背景**: infra.rs 本仓已含全部 7 个存储适配器 (kafkax v0.4.0, redisx v0.3.15, natsx v0.3.5, ossx v0.4.1, postgresx v0.3.13, taosx v0.3.10, clickhousex v0.3.6)，默认客户端入口均已落地（结构完成度高；package stable / Cluster·EOS / 部分 live 证据 OPEN，详见 [adapters-ssot-alignment](../../../../../docs/ssot/adapters-ssot-alignment.md)）。

**决策**: market_data.rs 构建薄领域包装层 (sink-core/ + sink-adapters/)，将 MarketEvent 编码为对应适配器期望格式，而非重新实现存储连接逻辑。

**替代方案 (已拒绝)**: 独立实现 7 个 sink crate。拒绝理由: (1) 重复 infra.rs 已有工作 (2) 增加维护成本 (3) 无业务差异化价值。

**影响**: 节省 Phase 3 约 4-6 周工作量。

### ADR-002: 数据流 — adapter → broadcast channel → pipeline

**背景**: VenueAdapter::subscribe_* 当前返回 Result<(), AdapterError>，无数据交付机制。

**决策**: 适配器内部使用 `tokio::sync::broadcast` channel 将行情事件扇出给所有 pipeline 消费者。适配器拥有 channel 发送端，消费者持有接收端。

**替代方案 (已拒绝)**: (1) 回调 trait — Rust async 中生命周期复杂; (2) Stream trait — 单一消费者限制。

**影响**: 需扩展 VenueAdapter 或新增内部 API。所有消费者获得 Arc 零拷贝事件副本。

### ADR-003: 错误分类 — is_retryable() / is_transient()

**背景**: AdapterError (9 变体) 和 SinkError (5 变体) 均缺少重试分类方法，消费者被迫 match-hack。

**决策**: 在 `AdapterError` 和 `SinkError` 添加:
- `is_transient() -> bool` — 重试可能成功 (Network, RateLimit, Timeout)
- `is_retryable() -> bool` — 安全重试 (排除被 rejected 的请求如 InvalidRequest)

**影响**: Phase 1 在 domain_exchange 中实现。

### ADR-004: 安全 — 密钥管理

**背景**: R10 审查发现 3 个 P0 安全缺陷。

**决策**:
1. 使用 `secrecy::SecretString` 替代 `Option<String>` 存储凭据
2. BinanceConfig/Adapter 实现手动 Debug (脱敏)
3. from_config 不丢弃 secret_key
4. HMAC-SHA256 签名为独立模块

**影响**: Phase 1 修复 from_config + 手动 Debug; Phase 2 实现签名。

## 管道决策（未来）

### Pipeline-001: Cleanse 先于 Sink
> 已决定但未实现

pipeline-cleanse 在 sink 之前执行，确保存储只接收已验证数据。

### Pipeline-002: 编排层
> 已识别但未实现

未来需编排层 (collector 二进制或单独 crate) 启动 adapter → pipeline → sink 的生命周期。

### Pipeline-003: ossx WAL 优先
> 已决定但未实现

ossx (冷归档) 必须在事件发布到 broadcast channel 之前完成 WAL 写入，防止慢消费者丢失数据。
