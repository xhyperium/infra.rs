# hyperliquid — Hyperliquid Exchange Market Data Adapter

**SSOT 根**: `.agents/ssot/hyperliquid/`  
**规格**: `spec/spec.md`
**crate 路径**: `crates/exchange/hyperliquid`
**package**: `exchange-hyperliquid`
**lib**: `exchange_hyperliquid`
**当前状态**: DTO/config 与 `VenueAdapter` skeleton 已建立；全部方法返回 skeleton `Internal`，meta 映射/l2 full-refresh/订阅运行时待实现。

## 概述

Hyperliquid 是一个高性能 L1 DEX，专注于永续合约（Perpetual Futures）交易。本模块当前提供公开行情协议的类型骨架，运行时接入能力尚未实现。

Hyperliquid 采用独特的 Layer 1 架构，所有订单簿和撮合均在链上运行，通过其 InfoController 提供 WebSocket 和 REST API 接口供第三方读取行情数据。

## 关键职责与状态

- [已存在] WebSocket 流、Info 请求/响应、配置和连接状态的类型骨架
- [已存在] `VenueAdapter` 统一接口骨架
- [待实现] WebSocket 实时行情流接入与自动重连（InfoController 协议）
- [待实现] REST API 行情数据/快照查询
- [待实现] Hyperliquid 协议到本仓域模型的映射（coin → InstrumentKey）
