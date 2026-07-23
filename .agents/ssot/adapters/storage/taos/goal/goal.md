# adapters/storage/taos — Goal（infra.rs P0）

| 字段 | 值 |
|------|-----|
| package | `taosx` |
| 标题 | TDengine TimeSeries |
| 实现 | `crates/adapters/storage/taos` |
| 状态 | **Production-default 全 API 面已落地**（`0.3.10`）；gap 未完成 **0** |
| SSOT 路径 | `.agents/ssot/adapters/storage/taos/`（**不**另建 `taosx/`） |

## Outcome

在 infra.rs 提供可配置、可关闭、可 live/e2e/bench 验证的 TDengine TimeSeries 客户端：写查、流、批处理器、幂等重试、TMQ 闭环、metrics 导出、soak、HA-lite 故障转移。

## Acceptance

1. `cargo test -p taosx --all-targets` 离线绿
2. live：integration_all_api + e2e_klines + live_selfcheck Full
3. bench：api_matrix / hot_path 有界
4. gap register 未完成计数 = 0
5. 密钥仅 secrets 注入；产物写 `/home/workspace/data/taosx`

## Not in scope

- 平行 SSOT `taosx/` 树
- crates.io 发布宣告
- 量化全栈（K 线产品 schema 全家桶）
