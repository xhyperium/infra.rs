# coinglass — Coinglass Crypto Market Data Adapter

**SSOT 根**: `.agents/ssot/coinglass/`
**规格**: `spec/spec.md`
**crate 路径**: `crates/exchange/coinglass`
**package**: `exchange-coinglass`
**lib**: `exchange_coinglass`
**当前状态**: DTO/config 默认 V4 base、REST-only `Unsupported` 与 connect/disconnect 已建立；可走 REST 的 `get_instruments`/`get_order_book` 仍是 skeleton `Internal`。

## 概述

Coinglass（曾用名 Bybt）是跨交易所加密市场数据聚合平台。本模块当前已建立聚合指标响应、限频配置和适配器类型骨架；REST 数据接入运行时尚未实现。

## 关键职责与状态

- [已存在] REST 响应包装、`CoinglassConfig`、`RateLimitConfig` 和适配器类型骨架
- [待实现] REST API 数据接入与定时拉取
- [待实现] 跨交易所数据聚合、Coinglass 协议到本仓域模型的映射
- [待实现] 交易所/标的符号映射与规范化
