# adapters/storage/postgres — Matrix

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| S-1 | workspace member `postgresx` | PASS | Cargo.toml `0.3.10` |
| S-2 | 生产默认导出 | PASS | Pool/Tx/Repository/COPY/acquire_with |
| S-3 | from_env / FOUNDATIONX_* | PASS | 含 TLS_CA/SNI/CLIENT_CERT/KEY |
| S-4 | 离线测试 | PASS | cargo test -p postgresx |
| S-5 | live ignore 入口 + dev live | PASS | tests/live_postgres.rs 11/11 |
| S-6 | bench 有界 | PASS | benches/query_hot_path.rs |
| S-7 | crate docs | PASS | docs/* |
| S-8 | SSOT 11 层 + landing | PASS | 本树 |
| S-9 | package stable | OPEN | publish=false |
| S-10 | DEFER 能力 | OPEN | 无限流式 COPY / migrations / read-replica |
| S-11 | deadline conformance | PASS | scripts/postgres-deadline-conformance.mjs |
| S-12 | 远程 Require TLS live | PASS | CA+SNI |
| S-13 | raw fail-closed live | PASS | live_raw_client_and_pool_fail_closed |
| S-14 | acquire_with | PASS | live |
| S-15 | 有界 COPY IN/OUT | PASS | live TEMP |
| S-16 | mTLS 客户端身份 | PASS | 离线构建 + 成对校验；服务端强制 live 部署依赖 |
