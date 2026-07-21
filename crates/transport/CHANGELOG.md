# Changelog — transportx

本文件记录 transportx 的用户可见变更，遵循 [Keep a Changelog](https://keepachangelog.com/)。
版本号以对应 `Cargo.toml` 的 `[package] version` 为准。

## [Unreleased]

### Added

- 真实 `benches/hot_path`（`cargo bench -- --quick` 可测）
- 公开 API 集成覆盖扩展（`tests/public_api_surface.rs` 等）
- `docs/API.md`：公开消费面与最小用法


## [0.1.0] — 2026-07-21

### 新增

- 初始落地：`HttpDriver` / `WsConnector` / `WsConnection` 边界
- `TransportError` 重连语义变体（timeout / closed / rate-limited / protocol / I/O）
- `ReqwestHttpDriver`、`TungsteniteWsConnector` 真实驱动（驱动类型 crate-private）
- `MockHttpTransport`（`set_get` / `set_post` + `HttpDriver`）
- 遗留 `HttpTransport`（`#[deprecated(note = "use HttpDriver")]`）
- loopback 集成测试与 100% 行覆盖率门禁
