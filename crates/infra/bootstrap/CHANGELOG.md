# Changelog

本文件记录 Cargo package / lib `bootstrap` 的变更。格式遵循
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/)，版本遵循
[Semantic Versioning](https://semver.org/spec/v2.0.0.html)。本 workspace 各 package
独立版本；`bootstrap` 当前为 `0.3.3` 且 `publish = false`。

## [Unreleased]

当前无新增条目。

## [0.3.3] — 2026-07-23 未发布候选

- 与 main 的 `ContractStoreSet` typed storage contracts 整合后完成最终回归：60 passed + 1 ignored。
- 最终错误文本修复后 root 串行 coverage 为 `963 / 963`、zeros 0、100.0000%；`975 / 975` 与
  `961 / 961` 为中间树历史。
- 本节只记录 Cargo 当前版本与候选证据，不代表 tag、外部发布或最终 reviewer/verifier 结论。

## [0.3.2] — 2026-07-23

> 本节记录工作树中的 Cargo 版本与候选变更，不代表已创建 tag 或完成外部发布。

### Added

- `ContractStoreSet` 提供正式 `contracts::KeyValueStore` / `EventBus` 固定 typed 槽位，
  并由 `Bootstrap`、`PlatformContext` 与 `AppContext` 暴露只读接线面。
- 固定摘要 Redis/NATS 真实组合实验通过；具体 adapter 仅作为 dev-dependencies，
  不表示跨资源事务或通用 Service Locator。
- `AppContext::trigger_shutdown` / `AppContext::graceful_shutdown` 与
  `BootstrappedApp::graceful_shutdown`。
- 关停组合测试：signal-before-drain、批内 LIFO、多结果、兼容转移与并发快照。
- 第 1、2、3 轮问题、修复与验证记录。

### Fixed

- `Bootstrap::build` / `try_build` 不再丢弃唯一 `ShutdownGuard`；四条成功 build
  路径的产物均保有关停所有权。
- 文档不再把同步 drain 描述为具备 timeout / 取消安全，并明确阻塞、panic 与
  register/drain 线性化边界。
- package、直接依赖、测试路径与构建校验事实按 Cargo 和源码真相对齐。
- ownerless `AppContext` 的 graceful shutdown 改为 fail-closed：signal 未触发时返回
  `MissingDependency("shutdown_guard")` 且不执行 hook；外部预触发后允许 drain。
- drain 注册 mutex 中毒不再伪装为配置非法，改为保留 source 的
  `DependencyUnavailable("drain")`。
- `BootstrapError` 三类 Display 与转换后的 `XError` context 统一为简体中文；
  DependencyUnavailable 顶层不内插任意 source 文本，source 链继续结构化保留；bootstrap 自有
  drain 错误与示例输出同步中文化。

## [0.3.1] — 2026-07-22

### Added

- 真实 `benches/hot_path`（`cargo bench -- --quick` 可测）。
- 公开 API 集成覆盖扩展（`tests/public_api_surface.rs` 等）。
- `docs/API.md`：公开消费面与最小用法。
- typed composition root（ADR-016）：
  - `Bootstrap` / `PlatformContext` / `AppContext` / `BootstrappedApp` / `ShutdownController`；
  - `MarketDataContext` / `ExecutionContext`；
  - `BootstrapError` → `kernel::ErrorKind`（Missing / Invalid / Unavailable）；
  - 四条 build 路径与 `require_evidence` fail-closed；
  - `StoreSet` 类型化接线与 `AsyncDrain` 同步 LIFO hook。
- 可移植 `traits` 替面、示例与本仓 SSOT 对齐文档。

### Changed

- ADR-005 注入链闭合：`Instrumentation` re-export `contracts::Instrumentation`，
  `Bootstrap::new` 默认使用 `observex::TracingInstrumentation`；
  `NoopInstrumentation` 保留为显式静默实现。

## [0.3.0] — 2026-07-21

- 本 workspace 首次引入 `bootstrap` package。
