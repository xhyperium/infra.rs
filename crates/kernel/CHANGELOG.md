# Changelog

本文件记录 `xhyper-kernel`（lib 名 `kernel`）的变更。格式遵循 [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)，版本遵循 [Semantic Versioning](https://semver.org/spec/v2.0.0.html)。

版本号与 workspace `package.version` 对齐。

---

## [Unreleased]

### Added

- 按 crates 子模块标准补齐 `README.md`、`AGENTS.md`、`examples/`、`docs/` 骨架

---

## [0.3.0] — 2026-07-21

### Added

- L0 语义：`error`（`ErrorKind` / `XError`）、`clock`（墙钟 / 单调钟）、`lifecycle`（关停信号）
- 集成测试：`public_api`、`api_compile`、`clock_contract`、`lifecycle_concurrency`
