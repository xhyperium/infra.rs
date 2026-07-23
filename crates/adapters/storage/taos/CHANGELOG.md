# Changelog

## [0.3.9] — 2026-07-23

### Added

- **流式查询** `TaosQueryStream` / `query_series_stream`
- **异步批写** `WriteBatcher`（push/flush/close）
- **幂等重试** `RetryPolicy` + `write_batch_idempotent`
- **TMQ 闭环** `TmqConsumer`（CREATE TOPIC 可降级为源表水位轮询）
- **Prometheus 文本** `TaosMetricsSnapshot::to_prometheus_text` / `metrics_prometheus`
- **WS SQL 短会话** `exec_sql_ws`；**Native TCP 探测** `probe_native_tcp`
- **HA-lite** `TaosConfig.hosts` 故障转移
- **soak** `run_soak` → `/home/workspace/data/taosx/soak`（`TAOSX_SOAK_SECS`）
- 测试：`integration_all_api` / `e2e_klines` / `live_selfcheck` Full（tmq 可 PASS）
- bench：`api_matrix`
- gap 清零表：`docs/ssot/taosx-gap-register.md`（未完成=0）

### Boundaries

- 档次 **Production-default**（`publish=false`）；**不**宣告 crates.io package-stable
- CREATE TOPIC 无权限时 TMQ 降级为源表轮询（selfcheck 仍可闭环）
- 24h soak 用 `TAOSX_SOAK_SECS=86400` 外挂调度；默认短时

## [0.3.8] — 2026-07-23

### Added

- **自验证 `taosx::selfcheck`**（LIB-SELFCHECK-SPEC / `.cargo/draft/verifyctl.md` §6.7）
  - 模型：`CheckLevel` / `CheckStatus` / `CheckItem` / `ValidationReport` / `CheckDescriptor`
  - `TaosValidator` + `Validatable`：catalog 9 项；Basic / ReadWrite / Full 级别与短路
  - 资源命名 `_sc_{token}` + 运行后 `DROP STABLE`；配置 skip / baseline / expected_precision
  - 检查：`ping`、`insert_query`、`stable_ddl`、`auto_subtable`、`tag_filter`、`interval_window`、`last_row`、`tmq_subscribe`（Skipped）、`db_config`
  - live：`tests/live_selfcheck.rs`（默认 ignore）

### Boundaries

- **不是** `tools/verifyctl`（Goal Contract 变更验证）
- **未**实现跨模块 `SelfValidator`、HTTP 探针、Prometheus 导出
- `tmq_subscribe` 诚实 Skipped（本 crate 无 TMQ 客户端）
- HA / Native SQL / 自动幂等重试 / package stable 仍 NO-GO

## [0.3.7] — 2026-07-23

### Added

- `TaosHealth` 与 `TaosPool::health()` / `liveness()`
- readiness 未就绪时返回 `ready=false`（信封 `Ok`），便于编排探针
- metrics：`health_ready` / `health_not_ready`
- live：`live_health_ready`

### Boundaries

- 不宣称 package stable；HA / Native SQL / 自动幂等重试仍 NO-GO

## [0.3.6] — 2026-07-23

### Added

- `TaosMetricsSnapshot` 与 `TaosPool::metrics()`：sql/write/query/ping 有界计数
- 进程级 `ws_probe_totals()`；`connect_native_ws` 记成功/失败
- live：`live_native_ws_handshake` + `live_metrics_after_write_query`

### Boundaries

- 非 OTLP/Prometheus 导出；完整 RED 远程指标仍 NO-GO
- Native WS 仍仅握手/关闭探测，不执行 SQL

## [0.3.5] — 2026-07-23

### Added

- `BatchWriteReport` / `BatchWritePartialError`：多 chunk 写入可定位 accepted/failed
- `write_batch_report` / `write_batch_chunked_report` / `write_batch_chunked_outcome`
- 部分成功单测：第二 chunk 失败时 `accepted=1` 结构化报告

### Changed

- `write_batch` / `write_batch_chunked` 仍返回 `()`，内部委托报告 API
- 公开 API 表面测试注入假密码验证 Debug 脱敏（N-1）

### Boundaries

- **不**自动重试已提交 chunk（幂等重试仍 NO-GO）
- 不宣称 package stable / Native SQL / HA

## [0.3.4] — 2026-07-23

### Added

- 公开 API 表面测试扩面：crate-root 常量、`TaosConfig` URL/`from_env`、池同步面方法全点名
- 十轮 draft 审查与最终缺口矩阵：`docs/report/2026-07-23/taosx-ten-round-review.md`
- 真实 dev live 证据（`export-foundationx-env` + `live_smoke` 2/2）与 SSOT matrix S-5b

### Changed

- SSOT 对齐文档 version 与 package 同步为 `0.3.4`；明确 **不** 另建 `adapters/storage/taosx/` SSOT 树
- crate docs 补充真实配置 live / 有界 bench 命令（密钥仅环境注入）

### Boundaries

- 仍不宣称 package stable / Native SQL / HA / 幂等自动重试 / 24h soak

## [0.3.3] — 2026-07-23

### Changed

- `query_series` 缺表空集路径改为依赖类型化 `ErrorKind::Missing`（`map_taos_code`），去掉驱动文案子串匹配

### Added

- 精度配置与探测结果不一致时 `connect` fail-closed 单测
- 缺表 Missing vs 非 Missing 错误传播对抗单测
- `tests/taos_conformance.rs`：远程明文拒绝、响应上界、schema 冲突、close/背压等离线边界

### Boundaries

- 未宣称 package stable / 真实集群 HA

## [0.3.2] — 2026-07-23

### Changed

- 远程明文、空认证与 URL 注入配置 fail-closed；REST 禁止 redirect
- bid/ask 改为 NCHAR(64) Decimal 文本，并拒绝存量 DOUBLE schema
- 为响应、SQL batch、query rows、in-flight 与 close drain 增加硬上界
- 子表名改用 symbol 完整十六进制编码，消除清洗碰撞
- 明确 NativeWs 仅握手可达性探测；Native SQL / HA / 幂等重试仍 NO-GO

### Added

- scale=18 / 大 mantissa / 正负 Decimal 往返测试
- 固定 TDengine 镜像 digest 的隔离 live conformance 脚本

## [0.3.1] — 2026-07-22

### Added

- 显式批量写入：`write_batch` / `write_batch_chunked` + 纯函数 `build_insert_sql_chunks`
- `TransportMode { Rest, NativeWs }`；`native_ws_url` / `connect_native_ws`（真实 WS 握手尝试）
- 池强化：`max_in_flight` Semaphore + `TaosPoolStats { in_flight, closed }`
- acquire 超时 → `DeadlineExceeded`；关闭后拒绝新请求
- `TaosConfig::validate`（`max_in_flight` / `batch_max_rows` ≥ 1）

### Changed

- 版本 PATCH 0.3.0 → 0.3.1
- `TimeSeriesStore::write_series` 委托 `write_batch`

## [Unreleased]

### Added

- 生产默认：`TaosConfig` / `TaosPool` REST 客户端（6041）
- `TimeSeriesStore` 真实 write/query + 库精度探测
- live smoke（`#[ignore]`）与 `hot_path` bench
- feature `scaffold`：保留内存 `TaosAdapter`

### Changed

- 收敛到 `xhyper-contracts::TimeSeriesStore`；默认路径不再是 scaffold
