# Changelog — marketd

遵循 [Keep a Changelog](https://keepachangelog.com/)，版本号见 `Cargo.toml`。

## [Unreleased]

### 变更

- 文档：README 补充「非职责」与「限制与安全」（crate-standard wave2）。


### 变更

- 文档：`main.rs` 补充 crate 级 `//!` 说明（crate-standard `crate-root-docs`）。
- composition：OS 关停路径同时 `trigger` kernel `ShutdownGuard`；async 循环仍用 `watch`（不调用阻塞 `wait`）。

### 新增

- 补充 crate 职责、维护约束与设计入口文档。
- 依赖 `kernel`：组合根消费 `ShutdownSignal` / `ShutdownGuard`（residual R8）。
