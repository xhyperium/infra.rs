# Changelog

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

## [Unreleased]

### Added

- 生产 `OssClient`：reqwest + OSS Signature V1
- `OssConfig` / Builder / `from_env`（`FOUNDATIONX_OSSX_*`）
- `delete_object`；`Debug` 脱敏
- live `#[ignore]` 测：`infra-draft/` 前缀 put/get/delete
- 微基准 `put_get`（配置 + 签名）

### Changed

- 默认路径为生产客户端；scaffold `OssAdapter` 移至 feature `scaffold`
