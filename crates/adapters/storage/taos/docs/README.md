# taosx docs

| 文档 | 说明 |
|------|------|
| [usage.md](usage.md) | 快速使用与公开 API |
| [config.md](config.md) | 配置与环境变量 |
| [operations.md](operations.md) | 运维与 live |
| [../../../../docs/ssot/taosx-ssot-alignment.md](../../../../docs/ssot/taosx-ssot-alignment.md) | SSOT 对齐 |
| [../../../../docs/report/2026-07-23/taosx-ten-round-review.md](../../../../docs/report/2026-07-23/taosx-ten-round-review.md) | 十轮审查 |

## 生产默认面

`TaosPool` REST `TimeSeriesStore`（package `0.3.5`；含 `BatchWriteReport`）。

密钥仅 `FOUNDATIONX_*` 环境变量；默认 `cargo test` 离线绿灯；live 测试 `#[ignore]`。
SSOT 路径：`.agents/ssot/adapters/storage/taos/`（**不**另建 `taosx/`）。
