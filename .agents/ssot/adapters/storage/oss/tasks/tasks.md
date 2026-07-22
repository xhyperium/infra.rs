# adapters/storage/oss — Tasks

## P0（已关闭 · #188–#191）

- [x] 生产默认客户端与配置
- [x] unit / lib 测试
- [x] live `#[ignore]` 测试：`tests/live_object_store.rs`
- [x] bench：`benches/put_get.rs`
- [x] crate docs：usage / config / operations
- [x] SSOT landing + draft 快照
- [x] docs/ssot/ossx-ssot-alignment.md
- [x] 远程明文 endpoint fail-closed
- [x] 对象/缓冲/错误体/in-flight/retry 硬上界与 deadline
- [x] multipart ETag XML、part size/count、abort/orphan 可审计边界

## OPEN / DEFER

- [ ] lifecycle / STS 临时凭证 / 流式 TB 对象与 checksum
- [ ] package stable 证据包（若 Lead 启动）
