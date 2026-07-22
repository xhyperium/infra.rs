# Changelog — contracts

## [0.1.0] — 2026-07-21

### 新增

- R4 15 trait（SSOT）
- 最小 contract-testkit：`fakes` 模块（`FakeTx*` / `FakeEventBus` / `FakeKeyValueStore` /
  `FakeRepository` / `RecordingInstrumentation` / `RecordingTxRunner`）
- First-batch trait 语义文档：`docs/contracts/*.md`
- Venue override 门禁：`tests/venue_override_gate.rs` + 默认中文错误常量
  （`VENUE_*_DEFAULT_MSG` / `is_default_*` helper）
- First-batch 合同套件：`tests/conformance_first_batch.rs`
- `public_surface` 覆盖 FakeKeyValueStore / FakeRepository / RecordingInstrumentation

### 破坏性

- 替换 xhyper-contracts

### 状态说明

- **非**整体 Production Ready；CT-8 其余 trait / 真实后端 / 编译期 override lint 仍 DEFER

## [Unreleased]

### Breaking

- 移除本 crate 内 Fake/Recording 公开类型（`FakeTx*` / `FakeEventBus` / `FakeKeyValueStore` /
  `FakeRepository` / `Recording*` / `InstrEvent`）
- Fake 与 per-trait suite 迁至独立 package **`contract-testkit`**
  （`crates/test-support/contracts`，仅 dev-dep）

### Added

- `venue_gate` 模块：保留 `VENUE_*_DEFAULT_MSG` / `is_default_*`
- 真实 `benches/hot_path`
- 公开 API 集成覆盖扩展
- `docs/API.md`

## [0.1.2] — 2026-07-23

### 修复（Additive Only）

- `LiveHandles::validate` 对无匹配句柄的 repo/account/venue_time 声明 fail-closed。
- 明确 `LiveContractProfile` 仅为接线意图，不是 readiness attestation。
- 新增准确命名 helper，消除 `bus_publish` 的 E2E 与 `tx_kv_set` 的原子性暗示；旧入口保留兼容。
- 增加公共失败路径测试。

### 状态

- 15 个 trait 方法无删除/签名变更；整体 Production Ready、交易业务 live、跨 backend 原子性与 E2E delivery 仍 NO-GO。
