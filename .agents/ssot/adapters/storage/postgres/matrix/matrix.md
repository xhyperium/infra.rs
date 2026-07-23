# adapters/storage/postgres — Matrix

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| S-1 | workspace member `postgresx` | PASS | Cargo.toml `0.3.13` |
| S-2 | 生产默认导出 | PASS | Pool/Tx/Repository/COPY/Migrator/selfcheck |
| S-3 | from_env / FOUNDATIONX_* | PASS | 含 TLS 全项 |
| S-4 | 离线测试 | PASS | cargo test -p postgresx |
| S-5 | live | PASS | tests/live_postgres.rs 12/12 |
| S-6 | bench 有界 | PASS | benches/query_hot_path.rs |
| S-7 | crate docs | PASS | docs/* |
| S-8 | SSOT 11 层 | PASS | 本树 |
| S-9 | package stable | OPEN | publish=false |
| S-10 | DEFER | OPEN | 无限流式 COPY / read-replica / down migration |
| S-11 | deadline conformance | PASS | script |
| S-12 | 远程 Require TLS live | PASS | CA+SNI |
| S-13 | raw fail-closed live | PASS | live |
| S-14 | acquire_with | PASS | live |
| S-15 | 有界 COPY | PASS | live |
| S-16 | mTLS 客户端身份 | PASS | 离线 |
| S-17 | Migrator verify/apply | PASS | live checksum + advisory lock |
| S-18 | LIB-SELFCHECK §6.1 自验证 | PASS | selfcheck catalog + live Full |
| S-19 | SSOT/crate 合同文档对齐 0.3.12+ | PASS | dual-spec + 标准/operations/goal |
