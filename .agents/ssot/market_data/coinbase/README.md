# coinbase — Coinbase Exchange Market Data Adapter

**SSOT 根**: `.agents/ssot/coinbase/`  
**规格**: `spec/spec.md`
**crate 路径**: `crates/exchange/coinbase`
**package**: `exchange-coinbase`
**lib**: `exchange_coinbase`
**当前状态**: DTO/config 与 Advanced Trade endpoint 对齐（`new`/`from_config`）；全部方法仍 skeleton `Internal`，协议运行时待实现。

## 概述

Coinbase 交易所（Advanced Trade API）市场数据适配器模块的类型骨架。当前已在 `src/lib.rs` 中定义频道、订阅、产品、配置和连接状态类型；行情接入运行时尚未实现。

协议版本对应 Coinbase Advanced Trade API（原 Coinbase Pro 的升级版本）。

## 关键职责与状态

- [已存在] WebSocket 频道、订阅、产品、配置和连接状态的类型骨架
- [已存在] `VenueAdapter` 统一接口骨架
- [待实现] WebSocket 实时行情流接入与自动重连
- [待实现] REST API 历史数据/快照查询
- [待实现] Coinbase 协议到本仓域模型的映射
