# okx 设计文档

## 当前实现状态

当前 crate 位于 `crates/exchange/okx`，manifest 中的 package 名为 `exchange-okx`，lib 名为 `exchange_okx`。`src/lib.rs` 已建立 OKX 频道、REST 响应、交易对信息、配置、连接状态及 `VenueAdapter` 接口类型骨架；协议运行时尚未实现，不能据此认定 WebSocket 或 REST 已通过验证。

## 设计决策

### D1. instId 双向映射

OKX 使用 `instId`（如 `BTC-USDT-SWAP`），本仓使用 `InstrumentKey{product_line, symbol}`。适配器需维护 `instId ↔ InstrumentKey` 双向映射表。

### D2. Books 序列连续性与 checksum 边界

OKX 深度频道的 `action`、`seqId` 和 `prevSeqId` 语义必须按官方频道逐项固定。当前官方文档将 checksum 标记为 deprecated 且固定为 0，适配器不得执行旧 CRC32 算法；sequence 不连续或 reset 未按规则处理时丢弃并重新获取 snapshot。未来若官方重新启用有效完整性字段，必须先更新 evidence 与 fixture，不能假设所有频道拥有相同校验字段。

### D3. Ping 保活

OKX WebSocket 在连续无消息时需使用小于 30 秒的定时器发送字符串 `ping`，等待 `pong`；超过 30 秒无活动会断开。不能把 20 秒写成官方固定值。

## 当前与后续 crate 布局

当前实现仍是单文件类型骨架：

```
crates/exchange/okx/
├── Cargo.toml
└── src/
    └── lib.rs            # 当前类型骨架与 VenueAdapter 实现骨架
```

以下为协议运行时实现后的拆分计划，相关文件目前尚未建立：

### 后续拆分计划（待实现）

```
crates/exchange/okx/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── config.rs
│   ├── types.rs          # 特有类型（instId 等）
│   ├── ws/
│   │   ├── mod.rs
│   │   ├── stream.rs
│   │   └── client.rs
│   ├── rest/
│   │   ├── mod.rs
│   │   └── client.rs
│   └── mapping/
│       ├── mod.rs
│       └── okx.rs
└── tests/
    └── integration.rs
```
