# Binance 适配器 — SSOT

> 行情数据域 Binance 交易所适配器模块的单一可信源（SSOT）。
> 适配器 crate: `crates/exchange/binance/` (workspace member: exchange-binance)

## 状态

| 项目 | 状态 |
|------|------|
| 适配器骨架 | DTO / Config / VenueAdapter skeleton 已实现 |
| 网关验证 | BN-ROUTE-001 (产品线端点路由) 已验证 |
| WebSocket | **未实现** (全部 11 个 VenueAdapter 方法返回 skeleton) |
| REST | **未实现** |
| 管道 | 设计阶段 (参考 `.cargo/draft/binance/`) |
| 基础设施映射 | 见 `infra-deps.md` (infra.rs 依赖) |

## 目录结构

```
binance/
├── README.md              ← 本文件
├── goal/goal.md           — 目标（市场覆盖 + 里程碑）
├── design/design.md       — 设计决策（ADR）
├── spec/spec.md           — 正式规格（合约 + 门禁表）
├── infra-deps.md          — infra.rs 基础设施依赖映射 [NEW]
├── wires/
│   ├── websocket.md       — WebSocket 流矩阵 [NEW]
│   └── rest.md            — REST 端点矩阵 [NEW]
├── datatypes/
│   └── types.md           — 17 种数据类型映射 [NEW]
├── security/
│   └── signing.md         — HMAC-SHA256 签名与安全基线 [NEW]
├── pipeline/              — 管道规格（未来）
└── review/                — 审查记录
```

## 门禁统计

| 类别 | 数量 | 已验证 |
|------|:---:|:---:|
| 路由 (BN-ROUTE) | 1 | 1 |
| WebSocket (BN-WS) | 2 | 0 |
| 深度 (BN-BOOK) | 1 | 0 |
| REST (BN-REST) | 1 | 0 |
| 速率 (BN-RATE) | 1 | 0 |
| 安全 (BN-SEC) | 8 | 0 |
| 性能 (BN-PERF) | 5 | 0 |
| 管道 (BN-PIPE/CLEAN/GAP/BACK) | 5 | 0 |
| Sink (BN-SINK) | 7 | 0 |
| **总计** | **31** | **1** |
