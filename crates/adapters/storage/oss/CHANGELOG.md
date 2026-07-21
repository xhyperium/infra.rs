# Changelog

## [Unreleased]

### Added

- 生产 `OssClient`：reqwest + OSS Signature V1
- `OssConfig` / Builder / `from_env`（`FOUNDATIONX_OSSX_*`）
- `delete_object`；`Debug` 脱敏
- live `#[ignore]` 测：`infra-draft/` 前缀 put/get/delete
- 微基准 `put_get`（配置 + 签名）

### Changed

- 默认路径为生产客户端；scaffold `OssAdapter` 移至 feature `scaffold`
