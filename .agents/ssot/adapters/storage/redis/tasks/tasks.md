# adapters/storage/redis — Tasks

## P0（已关闭 · #188–#191）

- [x] 生产默认客户端与配置
- [x] unit / lib 测试
- [x] live `#[ignore]` 测试：`tests/live_kv.rs · tests/live_kv_conformance.rs`
- [x] bench：`benches/kv_hot_path.rs`
- [x] crate docs：usage / config / operations
- [x] SSOT landing + draft 快照
- [x] docs/ssot/redisx-ssot-alignment.md

## infra-2d9.3.6 当前交付

| 项目 | 状态 |
|------|------|
| Pub/Sub 同源配置与失败关闭测试 | 实现 |
| 重试副作用/原子性代码合同与失败测试 | 实现 |
| active SSOT / crate 文档统一 | 实现 |
| Cluster / Sentinel / TLS 真实 live | OPEN（未运行） |
| package stable 证据包 | OPEN |
