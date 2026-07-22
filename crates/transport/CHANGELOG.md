# Changelog — transportx

本文件记录 transportx 的用户可见变更，遵循 [Keep a Changelog](https://keepachangelog.com/)。
版本号以对应 `Cargo.toml` 的 `[package] version` 为准。

## [Unreleased]

### Added

- 真实 `benches/hot_path`（`cargo bench -- --quick` 可测）
- 公开 API 集成覆盖扩展（`tests/public_api_surface.rs` 等）
- `docs/API.md`：公开消费面与最小用法

## [0.1.3] — 2026-07-23

### 安全与修复

- HTTP response 改为 chunk 流式累计，未知长度越界时立即中止。
- tungstenite 在解码/聚合前设置入站 frame/message 上限。
- URL userinfo 与全部 query value 的 Debug fail-closed 脱敏。
- 未接线的 `TlsConfig.sni=false` 明确拒绝；默认配置修正为 `sni=true`。
- 有界池新增配置校验与 RAII `HttpClientLease`。
- `Retry-After` 支持 delay-seconds 与 HTTP-date。

### 状态

- M3、企业 PKI/mTLS、WS 企业 TLS与完整业务 live 仍 NO-GO。

## [0.1.0] — 2026-07-21

### 新增

- 初始落地：`HttpDriver` / `WsConnector` / `WsConnection` 边界
- `TransportError` 重连语义变体（timeout / closed / rate-limited / protocol / I/O）
- `ReqwestHttpDriver`、`TungsteniteWsConnector` 真实驱动（驱动类型 crate-private）
- `MockHttpTransport`（`set_get` / `set_post` + `HttpDriver`）
- 遗留 `HttpTransport`（`#[deprecated(note = "use HttpDriver")]`）
- loopback 集成测试与 100% 行覆盖率门禁
