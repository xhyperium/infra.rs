# adapters/storage/clickhouse — Matrix

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| S-1 | workspace member `clickhousex` | PASS | Cargo.toml |
| S-2 | 生产默认导出 | PASS | `ClickHousePool / ClickHouseClient HTTP(S)` |
| S-3 | from_env / FOUNDATIONX_* | PASS | `HTTP_PORT` 优先、兼容 `PORT`、冲突 fail-closed |
| S-4 | 离线测试 | PASS | `cargo test -p clickhousex --all-targets` |
| S-5 | live ignore 入口 | PASS | `tests/live_smoke.rs` |
| S-6 | bench 有界 | PASS | `benches/hot_path.rs（3s 有界）` |
| S-7 | crate docs | PASS | docs/usage·config·operations |
| S-8 | SSOT 11 层 + landing | PASS | 本树 |
| S-9 | package stable | OPEN | 未宣称 |
| S-10 | DEFER 能力 | OPEN | native 9000 protocol / cluster / ReplicatedMergeTree 运维面 |
| S-11 | HTTPS/CA/远程明文拒绝 | PASS | 本地 CA conformance + config 负向测试；≠ 集群 live |
| S-12 | 错误正文脱敏 | PASS | 4096 字节解析上限；`security_failures.rs` |
| S-13 | 真实 TLS/auth/deadline/并发 | OPEN | 当前无真实 ClickHouse 可复验证据 |
