# Changelog

本文件记录 `xhyper-configx` 的变更。格式参考 [Keep a Changelog](https://keepachangelog.com/)。

## [0.1.0] — 2026-07-21

### Added

- 初始落地 active SSOT 0.1.0 合同：拥有型 `ConfigStore`
- API：`new` / `get` / `set` / `Default`
- 锁失败不对称：读中毒 → `None`；写中毒 → `XError::Invalid("config lock poisoned")`
- 单元/集成/并发测试与 `examples/basic`
- 生产依赖仅 `xhyper-kernel`；`default = []`

## [Unreleased]

### Added

- 真实 `benches/hot_path`
- 公开 API 集成覆盖扩展
- `docs/API.md`
