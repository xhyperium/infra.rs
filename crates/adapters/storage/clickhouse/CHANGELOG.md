# Changelog

## [0.3.3] — 2026-07-23

### Added（R1：负向验收闭合）

- `insert_json_each_row` / `insert_batch`：非法表名与非 object 行的拒绝路径现有单测锚定，且证明校验发生在任何网络请求**之前**（用必然连接失败的端口构造 pool，校验被跳过则测试会因网络错误而非 `Invalid` 失败）
- `insert_json_each_row` / `insert_batch`：空 `rows` 短路成功、不发起网络请求，已有单测覆盖
- `query_rows` 的 TabSeparated 解析（跳过空行、按 tab 拆列）补齐专项单测
- `map_http_error` 补齐完整分支覆盖：404→Missing、`Code: 57`（已存在）→Conflict、`Code: 81`（未知库）→Missing、5xx→Transient、403→Unavailable、未知 4xx→Invalid
- `read_error_prefix` 的 `ERROR_RESPONSE_CAPTURE_LIMIT`（4096 字节）截断边界补齐单测：超限响应体被截断，错误信息不包含截断之外的内容
- scaffold `ClickHouseAdapter`：多次 `sink` 累加（非覆盖）行为、`name`/`endpoint` 访问器与构造参数一致性、`local()` 默认端点均补齐单测

### Added（R2：对抗验证 / 边界回归）

- 背压边界：`max_in_flight=1` 时，第一个请求占住唯一许可并挂起，第二个并发请求在 `acquire_timeout` 到期后必须收到 `ErrorKind::DeadlineExceeded`（而非无限等待或被静默丢弃）
- `insert_batch` 分块的 HTTP 层证据：5 行、`max_rows_per_chunk=2` 必须产生 3 次独立 HTTP POST（2+2+1），而不是拼成一次请求

### Boundaries（不变，本轮未新增真实集群证据）

- 仍未运行真实 ClickHouse 集群；`https_conformance.rs` / `live_smoke.rs` 的 `#[ignore]` 场景未被本轮解除，真实 TLS/auth/deadline/并发 evidence 依旧 OPEN
- 本轮只补齐既有实现路径的单测锚点与对抗性边界回归，未变更任何生产逻辑分支

## [0.3.2] — 2026-07-22

### Added

- reqwest/rustls HTTPS 与可选 PEM CA；远程 HTTP 在配置阶段 fail-closed
- 严格环境变量解析和 TLS/timeout 校验
- 临时 CA/证书的本地 TLS 协议实验，覆盖受信 CA 成功与错误 CA fail-closed

### Boundaries

- 实验只证明客户端 HTTPS/CA，不声明真实 ClickHouse 集群 TLS、复制或 HA

## [0.3.1] — 2026-07-22

### Added

- `insert_batch(table, rows, BatchInsertOptions { max_rows_per_chunk })` 分块插入
- 纯函数 `chunk_ranges`（可测 chunk 尺寸）
- 池强化：`max_idle_per_host` / `max_in_flight`（默认 64）/ `acquire_timeout`
- `ClickHousePoolStats { in_flight, closed }`；关闭后拒绝新请求
- `ClickHouseConfig::validate`：`max_in_flight ≥ 1`

### Changed

- 版本 PATCH 0.3.0 → 0.3.1
- `connect` 使用配置的 `pool_max_idle_per_host`；请求经 Semaphore 背压

## [Unreleased]

### Added

- 生产默认：`ClickHouseConfig` / `ClickHousePool` HTTP 客户端（8123）
- `AnalyticsSink` 真实 insert 路径 + `query_text` / `query_rows` / `insert_json_each_row`
- live smoke（`#[ignore]`）与 `hot_path` bench
- feature `scaffold`：保留内存 `ClickHouseAdapter`

### Changed

- 收敛到 `xhyper-contracts::AnalyticsSink`；默认路径不再是 scaffold
