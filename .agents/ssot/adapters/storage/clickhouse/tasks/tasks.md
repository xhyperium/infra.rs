# adapters/storage/clickhouse — Tasks

## P0（已关闭 · #188–#191）

- [x] 生产默认客户端与配置
- [x] unit / lib 测试
- [x] live `#[ignore]` 测试：`tests/live_smoke.rs`
- [x] bench：`benches/hot_path.rs（3s 有界）`
- [x] crate docs：usage / config / operations
- [x] SSOT landing + draft 快照
- [x] docs/ssot/clickhousex-ssot-alignment.md
- [x] 0.3.2 HTTP(S) + PEM CA + 远程 HTTP fail-closed
- [x] `HTTP_PORT` / `PORT` 兼容与冲突拒绝
- [x] 错误响应读取有界且不回显 SQL/payload/认证正文
- [x] loopback 失败路径与本地 HTTPS conformance

## OPEN / DEFER

- [ ] native 9000 protocol / cluster / ReplicatedMergeTree 运维面
- [ ] 真实 ClickHouse TLS/auth/deadline/并发脱敏证据
- [ ] package stable 证据包（若 Lead 启动）
