# Changelog

## [0.4.0] — 2026-07-23

### Added

- `OssPool` — 共享连接池（Arc + Semaphore + Atomic 指标），OssClient 保留向下兼容
- `CredentialProvider` trait + `StaticCredentialProvider` + STS token 支持
- 流式上传 `put_stream()` — 自动 multipart + 并发 part 上传
- 流式下载 `get_stream()` — Range/If-Match/If-None-Match + ByteStream
- `head()` — HEAD 请求获取 ObjectMeta（size/etag/checksum/content_type）
- SSE-S3 服务端加密（`x-oss-server-side-encryption: AES256`）
- 预签名 URL（`presign_url()` + `PresignOptions`）
- 健康检查 `health()` + 池统计 `stats()`
- `ObjectKey` / `ObjectMeta` / `ByteStream` / `UploadOptions` / `DownloadOptions` 类型
- `live_object_store.rs` — 12 项全 API 面 E2E 测试（真实阿里云 OSS 通过）

### Changed

- 版本 MINOR 0.3.3 → 0.4.0
- `byte_stream_from_bytes()` 辅助函数
- Content-Length header 强制发送（修复空对象 PUT）

### Boundaries

- package stable 未宣称；crates.io 未发布
- lifecycle / STS 临时凭证轮换 / 流式 TB 对象与 checksum 仍 OPEN

## [0.3.3] — 2026-07-23

### Added

- `tests/oss_conformance.rs`：401/403 经完整重试链路不重试；operation deadline 超时不残留；无界配置拒绝（loopback HTTP，离线）

### Boundaries

- 真实 OSS 多区域/跨账号/生命周期策略 live 未在本轮扩展
- 未宣称 package stable

## [0.3.2] — 2026-07-23

### Changed

- 远程明文 fail-closed，并增加对象/错误体/并发/缓冲硬上限。
- multipart 使用单一总 deadline、XML 安全编码和有界 orphan 补偿注册表。
- 版本 PATCH 0.3.1 → 0.3.2。

## [0.3.1] — 2026-07-22

### Added

- multipart API：`initiate_multipart` / `upload_part` / `complete_multipart` / `abort_multipart`
- 高层 `put_object_multipart(key, data, part_size)`（切分 + 完成；失败 abort）
- 签名子资源：`canonicalized_resource_with_subresources`（`?uploads` / `uploadId` / `partNumber`）
- 纯函数 `split_parts`（分片切分）
- `resiliencx` 重试：`with_retry` / `default_retry_config`；put/get/delete/multipart 全路径包裹
- 网络 / 5xx 映射为 `Transient` 以便重试；4xx 映射 `Invalid`（不重试）

### Changed

- 版本 PATCH 0.3.0 → 0.3.1
