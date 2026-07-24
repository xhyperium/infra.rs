# Binance 适配器 — 规格

> Binance 适配器的正式 SSOT 合约，包含门禁矩阵。
> 来源: 草案 spec.md + arch1.md + 2.md + 10 轮审查 (R1-R10)
> 最后更新: 2026-07-24

---

## 1. 路由规格

### 1.1 产品线识别

BN-ROUTE-001 (已验证): 提供 ProductLine 即可正确构造 REST 和 WS 基地址。

```rust
fn endpoints_for_product_line(line: ProductLine) -> Result<(String, String), AdapterError>
```

详见: `design/design.md` D-001

---

## 2. WebSocket 规格

### 2.1 流订阅

BN-WS-001 (pending): 14 fixture JSON 文件覆盖所有流类型，验证 Wire → Domain 映射。

### 2.2 连接生命周期

BN-WS-002 (pending): Ping/pong 心跳、24h 主动重连、订阅自动恢复。

### 2.3 完整流矩阵

详见: `wires/websocket.md`

---

## 3. 深度同步规格

BN-BOOK-001 (pending): 快照到增量的 U/u 连续性验证。

详见: `.agents/ssot/orderbook/spec/spec.md` (Model A: Binance)

---

## 4. REST 规格

BN-REST-001 (pending): exchange info / depth / klines / trades 端点 mock 测试。

详见: `wires/rest.md`

---

## 5. 限流规格

BN-RATE-001 (pending): 429/418 处理，每 ProductLine TokenBucket 限流。

---

## 6. 安全规格

详见: `security/signing.md`

| Gate ID | 名称 | 状态 |
|---------|------|:---:|
| BN-SEC-001 | from_config 不丢弃 secret_key | specified |
| BN-SEC-002 | 日志脱敏 (手动 Debug) | specified |
| BN-SEC-003 | HMAC-SHA256 签名 | deferred |
| BN-SEC-004 | SecretString 密钥存储 | specified |
| BN-SEC-005 | 每市场 TokenBucket 限流 | specified |
| BN-SEC-006 | TLS 默认启用 | specified |
| BN-SEC-007 | recvWindow 防重放 | specified |
| BN-SEC-008 | 无硬编码凭据 | specified |

---

## 7. 性能规格

| Gate ID | 名称 | 状态 |
|---------|------|:---:|
| BN-PERF-001 | Channel 容量表 (EventBus, sink 队列, WS 帧) | specified |
| BN-PERF-002 | 每 sink 反压策略 (有界/无界) | specified |
| BN-PERF-003 | 订单簿内存预算 (每符号 BTreeMap) | specified |
| BN-PERF-004 | WS 连接限制 (每市场 1024 流) | specified |
| BN-METR-001 | Phase 2 指标最小集 (Prometheus) | specified |

---

## 8. 管道规格（未来）

| Gate ID | 名称 | 状态 |
|---------|------|:---:|
| BN-PIPE-001 | 规范化器 (所有 17 种类型 → MarketEvent) | specified |
| BN-PIPE-002 | EventBus fan-out (Arc broadcast) | specified |
| BN-CLEAN-001 | L0-L3 数据清洗规则 | specified |
| BN-GAP-001 | 3 级缺口检测 + REST 补齐 | specified |
| BN-BACK-001 | Vision/REST 双通道回填 + checkpoint | specified |

---

## 9. Sink 规格（未来, deferred）

| Gate ID | 名称 | infra.rs 适配器 |
|---------|------|---------------|
| BN-SINK-KFK | Kafka sink | kafkax v0.4.0 |
| BN-SINK-RDS | Redis sink | redisx v0.3.15 |
| BN-SINK-NATS | NATS sink | natsx v0.3.5 |
| BN-SINK-OSS | OSS/S3 sink | ossx v0.4.1 |
| BN-SINK-PGS | PostgreSQL sink | postgresx v0.3.13 |
| BN-SINK-TAO | TDengine sink | taosx v0.3.10 |
| BN-SINK-OLS | ClickHouse sink | clickhousex v0.3.6 |

---

## 10. 门禁汇总

| 类别 | 数量 | 已验证 | Pending | Specified | Deferred |
|------|:---:|:---:|:---:|:---:|:---:|
| 路由 | 1 | 1 | 0 | 0 | 0 |
| WebSocket | 2 | 0 | 2 | 0 | 0 |
| 深度同步 | 1 | 0 | 1 | 0 | 0 |
| REST | 1 | 0 | 1 | 0 | 0 |
| 限流 | 1 | 0 | 1 | 0 | 0 |
| 安全 | 8 | 0 | 0 | 6 | 2 |
| 性能 | 5 | 0 | 0 | 5 | 0 |
| 管道 | 5 | 0 | 0 | 5 | 0 |
| Sink | 7 | 0 | 0 | 0 | 7 |
| **总计** | **31** | **1** | **5** | **16** | **9** |
