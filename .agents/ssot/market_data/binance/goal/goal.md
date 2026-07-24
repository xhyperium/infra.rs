# Binance 适配器 — 目标

## 使命

将 Binance 四市场（现货/合约/期权）的公开市场数据接入 market_data.rs 统一行情数据域，提供规范化的实时行情数据分发能力。

## 市场覆盖

| 市场 | ProductLine | 符号 | 数据源 |
|------|------------|------|--------|
| 现货 | Spot | BTC, ETH, SOL, BNB | WebSocket + REST |
| U 本位合约 | Future (UM) | BTC, ETH, SOL, BNB | WebSocket + REST |
| 币本位合约 | Future (CM) | BTC, ETH | WebSocket + REST |
| 欧式期权 | Option | BTC, ETH | WebSocket + REST |

### 数据类型目标（17/17）

- ✓ 已存在规范类型: trade, aggTrade, bookTicker, depth, kline, depth 快照 (6)
- ✗ 需补充规范类型: 24hTicker, MarkPrice, OptionGreeks, IndexPrice (4)
- ⊘ 待分类: forceOrder (1)
- ✓ REST 类型: exchangeInfo, klines, aggTrades, listenKey (4)
- ✓ 连接管理: StreamConnection (1)
- ✓ 管理: SessionToken (1)

详见: `datatypes/types.md`

## 非功能目标

| 维度 | 目标 | 门禁 |
|------|------|------|
| 延迟 | WS → 规范化事件 < 50ms (P99) | BN-PERF-001 |
| 可用性 | 自动重连 + 订阅恢复, < 5s 恢复 | BN-WS-002 |
| 精度 | Decimal 无损, 禁止 f64 中转 | BN-WS-001 |
| 安全 | 密钥脱敏, HMAC 签名 (如用私有端点) | BN-SEC-001–008 |
| 测试覆盖 | 14 fixture + mock WS/REST | BN-WS-001, BN-REST-001 |

## 里程碑

| 里程碑 | 目标 | 预计 |
|--------|------|------|
| M1 | infra.rs 集成 (kernel + configx + resiliencx + testkit) | 1 周 |
| M2 | WebSocket + REST 实现 (VenueAdapter 方法) | 2 周 |
| M3 | 管道架构 (sink-core + 数据清洗) | 2 周 |
| M4 | 订单簿引擎 (Model A: Binance) | 2 周 |
| M5 | 历史回填 + 缺口补齐 | 2 周 |

## 成功标准

- [ ] 4 个市场的 12 种流类型正确采集并映射至 domain_market 规范类型
- [ ] VenueAdapter 11 个方法中至少 6 个（公开数据）返回真实数据
- [ ] WS 重连恢复率 > 99%（24h 运行）
- [ ] 所有 31 个门禁中至少 12 个标记为 verified
