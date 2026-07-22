# adapters/storage/clickhouse — Evidence

> 模块战役证据落盘处（≠ `crates/evidence` 生产库）。

## 本仓证据索引

| 类型 | 位置 |
|------|------|
| 单元/集成（离线） | `cargo test -p clickhousex --all-targets` 日志 |
| 失败路径（离线） | `tests/security_failures.rs`（loopback；响应正文脱敏） |
| HTTPS 客户端实验 | `tests/https_conformance.rs` + `scripts/clickhouse-https-conformance.mjs` |
| live | `crates/adapters/storage/clickhouse/tests/live_smoke.rs` 等 |
| bench | `crates/adapters/storage/clickhouse/benches/hot_path.rs` |
| landing | [../plan/infra-rs-landing.md](../plan/infra-rs-landing.md) |
| draft | [../plan/infra-rs-draft-spec-goal.md](../plan/infra-rs-draft-spec-goal.md) |
| 对齐 | `docs/ssot/clickhousex-ssot-alignment.md` |

有新的验证输出时按 `YYYY-MM-DD/` 建日目录归档。

本索引没有真实 ClickHouse TLS/auth/deadline/并发运行记录；这些 live 条款保持 OPEN。
