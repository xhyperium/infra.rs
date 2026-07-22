# Changelog

本文件记录 `configx` 的变更。格式参考 [Keep a Changelog](https://keepachangelog.com/)。

## [0.1.0] — 2026-07-21

### Added

- 初始落地 active SSOT 0.1.0 合同：拥有型 `ConfigStore`
- API：`new` / `get` / `set` / `Default`
- 锁失败不对称：读中毒 → `None`；写中毒 → `XError::Invalid("配置锁已中毒")`
- 单元/集成/并发测试与 `examples/basic`
- 生产依赖仅 `xhyper-kernel`；`default = []`

## [Unreleased]

当前无新增条目。

## [0.1.2] — 2026-07-23

> 本节记录工作树中的 Cargo 版本与候选变更，不代表已创建 tag 或完成外部发布。

### Added

- 真实 `benches/hot_path`
- 公开 API 集成覆盖扩展
- `docs/API.md`
- `try_get`、`try_snapshot` 与 `ConfigSnapshot::try_capture` Result 路径，用于区分缺失与毒锁
- `ConfigSnapshot` 自定义 `Debug`：`secret:` 前缀键值脱敏
- `MemorySource` 自定义 `Debug`：`secret:` 前缀键值脱敏
- `try_get_secret`、`try_subset_snapshot` Result 辅助路径
- `ConfigWaitOutcome` 与显式 `wait_outcome / wait_timeout_outcome`
- 原子批量与 reload 并发测试、失败保留测试、watch 溢出与伪唤醒测试

### Changed

- `extend_pairs`、`LayeredConfig::apply_to/reload_into` 与 `merge_into` 改为单写锁提交完整批次
- `LayeredConfig` 在提交前完成全部 source load 与 key 校验
- `require_keys`、`require_nonempty` 和 `merge_into` 改用显式失败快照
- `ConfigWatch` generation 改用 checked overflow；`wait_timeout` 改用整次调用总 deadline
- watch mutation 使用独立 mutex 串行，reload 等待 store 时不再持有 state mutex
- timed wait 使用 state `try_lock`，锁竞争受总 deadline 限界
- KEY=VALUE parse 错误不再回显原始行
- 用户可见 `XError` context 统一为简体中文，关键错误测试精确断言 kind/context
- timed wait 在接受 generation 前二次裁定 deadline，late notify 不再返回 Changed
- reload 锁边界测试改用 per-watch phase hook + Barrier，移除轮询竞态

### Compatibility

- `ConfigStore::get` 与 `ConfigSnapshot::capture` 保留 poison 折叠语义
- `get_secret / subset_snapshot / wait / wait_timeout` 保留兼容折叠语义
- reload 仍为进程内、调用方手动触发；未新增自动 watcher、远端源或后台 runtime
