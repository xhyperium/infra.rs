# adapters/storage/taos — Tasks

## P0（已关闭 · #188–#191）

- [x] 生产默认客户端与配置
- [x] unit / lib 测试
- [x] live `#[ignore]` 测试：`tests/live_smoke.rs`
- [x] bench：`benches/hot_path.rs（3s 有界）`
- [x] crate docs：usage / config / operations
- [x] SSOT landing + draft 快照
- [x] docs/ssot/taosx-ssot-alignment.md

## OPEN / DEFER

- [x] REST SQL / WS reachability 真实边界与远程 TLS/auth fail-closed
- [x] Decimal NCHAR(64+) 无损路径、存量 DOUBLE schema fail-closed
- [x] response/batch/query/in-flight/close 硬上界与离线测试
- [x] 固定 digest 隔离 live conformance 入口
- [ ] Native SQL / WS 认证长会话 / 全超表治理 / HA 集群
- [ ] 自动幂等重试与部分批次失败后的重复写语义
- [ ] package stable 证据包（若 Lead 启动）
