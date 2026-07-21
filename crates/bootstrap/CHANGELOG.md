# Changelog

本文件记录 `xhyper-bootstrap`（lib 名 `bootstrap`）的变更。格式遵循 [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)，版本遵循 [Semantic Versioning](https://semver.org/spec/v2.0.0.html)。

版本号与 workspace `package.version` 对齐。

---

## [Unreleased]

### Added

- 初始落地：typed composition root（ADR-016）
  - `Bootstrap` / `PlatformContext` / `AppContext` / `BootstrappedApp` / `ShutdownController`
  - `MarketDataContext` / `ExecutionContext`
  - `BootstrapError` → `kernel::ErrorKind`（Missing / Invalid / Unavailable）
  - 四条 build 路径：`build` / `try_build` / `build_app` / `try_build_app`
  - `require_evidence` fail-closed（仅 `try_*` 强制）
- 可移植 `traits` 替面：`Instrumentation`、`EvidenceAppender`、六个有界上下文能力 trait、`NoopInstrumentation`
- 示例 `examples/minimal.rs`；集成测试 `tests/public_api.rs`
- 本仓对齐文档：`docs/bootstrap-ssot-alignment.md`

---

## [0.3.0] — 2026-07-21

（本版本随 workspace 首次引入本 crate。）
