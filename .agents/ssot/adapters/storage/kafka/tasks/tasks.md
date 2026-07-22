# adapters/storage/kafka — Tasks

## P0（已关闭 · #188–#191）

- [x] 生产默认客户端与配置
- [x] unit / lib 测试
- [x] live `#[ignore]` 测试：`tests/live_event_bus.rs`
- [x] bench：`benches/hot_path.rs（3s 有界）`
- [x] crate docs：usage / config / operations
- [x] SSOT landing + draft 快照
- [x] docs/ssot/kafkax-ssot-alignment.md

## OPEN / DEFER

- [ ] EOS / transactional producer / schema registry / group coordinator 强依赖路径
- [ ] package stable 证据包（若 Lead 启动）
