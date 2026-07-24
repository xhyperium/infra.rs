# WebSocket 流规格

> Binance WebSocket 市场数据流矩阵（4 市场 × 12 流类型）
> 来源: 草案 spec.md §3 + arch1.md §5-6
> 审查: R6 (数据类型映射) + R9 (性能)

## 流矩阵

| 流名称 | Spot | UM Futures | CM Futures | Options | 更新频率 |
|--------|:----:|:----------:|:----------:|:-------:|---------|
| \<symbol\>@trade | ✓ | ✓ | ✓ | -- | 实时 |
| \<symbol\>@aggTrade | ✓ | ✓ | ✓ | -- | 实时 |
| \<symbol\>@kline_\<interval\> | ✓ | ✓ | ✓ | -- | 2s |
| \<symbol\>@depth20@100ms | ✓ | ✓ | ✓ | ✓ | 100ms |
| \<symbol\>@depth@100ms | ✓ | ✓ | ✓ | ✓ | 100ms |
| \<symbol\>@bookTicker | ✓ | ✓ | ✓ | -- | 实时 |
| \<symbol\>@ticker | ✓ | ✓ | ✓ | ✓ | 1s |
| \<symbol\>@markPrice | ✓ | ✓ | ✓ | ✓ | 3s |
| !forceOrder@arr | ✓ | ✓ | ✓ | -- | 实时 |
| \<symbol\>@optionTicker | -- | -- | -- | ✓ | 1s |
| \<symbol\>@index | -- | -- | -- | ✓ | 1s |
| \<symbol\>@optionPair | -- | -- | -- | ✓ | 1s |

## 流 URL 构造

- 单流: `wss://{base_url}/ws/{stream_name}`
- 组合流: `wss://{base_url}/stream?streams={s1}/{s2}/{s3}`
- 组合流限制: 每连接最多 1024 个流

## WebSocket 连接池

- 每 ProductLine 独立连接
- 连接生命周期:
  - 连接建立 → 订阅流列表 → 数据接收
  - Ping/pong 心跳 (3 分钟间隔)
  - **24h 强制断开**: T+23h 主动重连（防止服务端强制断开导致数据丢失）
  - 重连: 指数退避 `min(1s * 2^attempt + jitter, 60s)`
- 订阅恢复: 重连后自动重新订阅所有活跃流

## 消息帧格式

### 组合流包装

```json
{
  "stream": "btcusdt@trade",
  "data": { "e": "trade", "E": 123456789, "s": "BTCUSDT", ... }
}
```

### 数据事件类型（e 字段）

| e 字段值 | 含义 | 映射方向 |
|---------|------|---------|
| trade | 逐笔成交 | → Tick |
| aggTrade | 聚合成交 | → Tick |
| bookTicker | 最优报价 | → Quote |
| depthUpdate | 增量深度 | → OrderBook::Delta |
| kline | K 线 | → Bar |
| 24hrTicker | 24h 统计 | → TwentyFourHrTicker (缺失) |
| markPriceUpdate | 标记价格 | → MarkPrice (缺失) |
| optionTicker | 期权行情 | → OptionGreeks (缺失) |
| indexPriceUpdate | 指数价格 | → IndexPrice (缺失) |
| forceOrder | 强制平仓 | 待分类 |

## 错误处理

- 解析失败: 丢弃消息 + 计数器递增，不中断连接
- 流未知: 记录警告，不中断连接
- 连接断开: 自动重连 + 订阅恢复

## 深度同步（Model A — Binance 专用）

- Spot: `U <= lastUpdateId+1 <= u`, 然后验证 `U == prev_u+1`
- UM/CM: `U <= lastUpdateId <= u`, 然后验证 `pu == prev_u`
- 状态机: BUFFERING → SNAPSHOT_REQUESTED → ALIGNING → SYNCED → RESYNC
- 详见: `.agents/ssot/orderbook/spec/spec.md`

## 缺失流变体（待补充至 exchange-binance）

| 变体 | BinanceStream 枚举 | 优先级 |
|------|-------------------|:---:|
| aggTrade | 缺失 | P1 |
| markPrice | 缺失 | P1 |
| forceOrder | 缺失 | P2 |
| optionTicker | 缺失 | P2 |
| index | 缺失 | P2 |
| option_pair | 缺失 | P2 |
