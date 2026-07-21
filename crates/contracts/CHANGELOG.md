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
