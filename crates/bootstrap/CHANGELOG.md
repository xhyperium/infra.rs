# Changelog

本文件记录 `xhyper-bootstrap`（lib 名 `bootstrap`）的变更。格式遵循 [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)，版本遵循 [Semantic Versioning](https://semver.org/spec/v2.0.0.html)。

版本号与 workspace `package.version` 对齐。

---

## [Unreleased]

### Added

- 真实 `benches/hot_path`（`cargo bench -- --quick` 可测）
- 公开 API 集成覆盖扩展（`tests/public_api_surface.rs` 等）
- `docs/API.md`：公开消费面与最小用法

### Changed

- **ADR-005 注入链闭合**：
  - `Instrumentation` 改为 re-export `contracts::Instrumentation`（移除本地 trait 分叉）
  - `Bootstrap::new` 默认注入 `observex::TracingInstrumentation`（不再默认 `NoopInstrumentation`）
  - 新增生产依赖 `xhyper-contracts`、`xhyper-observex`
  - 公开 re-export `TracingInstrumentation`；`NoopInstrumentation` 保留为可选静默实现

### Added

- 初始落地：typed composition root（ADR-016）
  - `Bootstrap` / `PlatformContext` / `AppContext` / `BootstrappedApp` / `ShutdownController`
  - `MarketDataContext` / `ExecutionContext`
  - `BootstrapError` → `kernel::ErrorKind`（Missing / Invalid / Unavailable）
  - 四条 build 路径：`build` / `try_build` / `build_app` / `try_build_app`
  - `require_evidence` fail-closed（仅 `try_*` 强制）
- 可移植 `traits` 替面：`EvidenceAppender`、六个有界上下文能力 trait、`NoopInstrumentation`
- 示例 `examples/minimal.rs`；集成测试 `tests/public_api.rs`
- 本仓对齐文档：`docs/ssot/bootstrap-ssot-alignment.md`

---

## [0.3.0] — 2026-07-21

（本版本随 workspace 首次引入本 crate。）
