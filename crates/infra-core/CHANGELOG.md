# Changelog

本文件记录 `infra-core` 的变更。格式遵循 [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)，版本遵循 [Semantic Versioning](https://semver.org/spec/v2.0.0.html)。

版本号与 workspace `package.version` 对齐。

---

## [Unreleased]

### Added

- 按 crates 子模块标准补齐 `README.md`、`tests/`、`examples/`、`docs/` 骨架

---

## [0.3.0] — 2026-07-21

### Added

- 初始公共 API：`Error`、`Result<T>`、`hello()`
- `Error` 的 serde 序列化 / 反序列化，IO 错误 `source` 链保留
