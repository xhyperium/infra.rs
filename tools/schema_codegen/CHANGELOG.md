# Changelog — schema_codegen

遵循 [Keep a Changelog](https://keepachangelog.com/)，版本号见 `Cargo.toml`。

## [Unreleased]

### 变更

- 文档：README 补充「非职责」与「限制与安全」（crate-standard wave2）。
- SQL：`DECIMAL` / `NUMERIC` 映射为 `decimalx::Decimal`（ADR-006；禁止 `f64`）。显式 `DOUBLE`/`REAL`/`FLOAT*` 仍映射浮点。


### 修正

- 更新文档以反映已实现的四类输入生成器，并补充维护入口。
