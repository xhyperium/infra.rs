# okx — OKX Exchange Market Data Adapter

**SSOT 根**: `.agents/ssot/okx/`  
**规格**: `spec/spec.md`
**crate 路径**: `crates/exchange/okx`
**package**: `exchange-okx`
**lib**: `exchange_okx`
**当前状态**: DTO/config/连接参数与 `VenueAdapter` skeleton 已建立；全部方法返回 skeleton `Internal`，`seqId`/保活/映射运行时待实现。

## 概述

OKX 交易所市场数据适配器模块的类型骨架。当前已在 `src/lib.rs` 中定义频道、REST 响应、配置和连接状态类型；行情接入运行时尚未实现。

## 关键职责与状态

- [已存在] WebSocket 频道、REST 响应、配置和连接状态的类型骨架
- [已存在] `VenueAdapter` 统一接口骨架
- [待实现] WebSocket 实时行情流接入与自动重连
- [待实现] REST API 历史数据/快照查询
- [待实现] OKX 协议到本仓域模型的映射
