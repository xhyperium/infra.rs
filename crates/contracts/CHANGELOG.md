# Changelog — contracts

## [0.1.3] — 2026-07-23

### Changed

- `live` 兼容入口补充 `# Errors` 文档；`kv_set_then_commit_separate_resources` 收紧闭包捕获。

## [0.1.0] — 2026-07-21

### 新增

- R4 16 trait（含 `TxContext`）
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

## [0.1.2] — 2026-07-22

### Breaking

- 移除本 crate 内 Fake/Recording 公开类型（`FakeTx*` / `FakeEventBus` / `FakeKeyValueStore` /
  `FakeRepository` / `Recording*` / `InstrEvent`）
- Fake 与 per-trait suite 迁至独立 package **`contract-testkit`**
  （`crates/test-support/contracts`，仅 dev-dep）

### Added

- `TxRunError` / `TxRunResult` / `run_tx_lifecycle`：结构化保留事务生命周期失败
- ObjectStore / TimeSeriesStore / AnalyticsSink / PubSub 语义文档
- `venue_gate` 模块：保留 `VENUE_*_DEFAULT_MSG` / `is_default_*`
- 真实 `benches/hot_path`
- 公开 API 集成覆盖扩展
- `docs/API.md`

### Changed

- `TxRunner` / `TxContext` 明确为对象安全的生命周期面，不宣称业务操作原子绑定
- `LiveHandles::validate` 对 repo/account/venue_time 无句柄声明 fail-closed
- 旧 `run_tx_commit_on_ok` / `tx_kv_set` / `run_on_tx_context` 标记 deprecated 兼容
