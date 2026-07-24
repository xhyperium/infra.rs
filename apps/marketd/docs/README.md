# marketd 设计入口

- 组合根：`src/composition.rs`
- 进程入口：`src/main.rs`
- 重启与检查点验收：`tests/process_checkpoint_restart.rs`

市场数据模型、端口和交付语义由 [`market_data`](../../../crates/services/market_data/README.md) 定义。
