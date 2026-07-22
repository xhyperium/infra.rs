# ossx

阿里云 OSS 对象存储适配 — 实现 `contracts::ObjectStore`。

## 生产入口（默认）

| 类型 | 说明 |
|------|------|
| `OssConfig` / `OssConfigBuilder` | 配置；`OssConfig::from_env()` 读 `FOUNDATIONX_OSSX_*` |
| `OssClient` | ObjectStore + delete/multipart；共享连接池、Semaphore 背压、总 deadline |
| 签名 | OSS REST V1：`Authorization: OSS AccessKeyId:Signature`（HMAC-SHA1） |

```bash
# 加载密钥（勿把 secret 写进仓库）
source scripts/live/export-foundationx-env.sh /path/to/oss.env

cargo test -p ossx
cargo test -p ossx --test live_object_store -- --ignored --nocapture
```

### 环境变量

| 变量 | 必填 | 说明 |
|------|------|------|
| `FOUNDATIONX_OSSX_ENDPOINT` | 是 | 如 `https://oss-ap-northeast-1.aliyuncs.com` |
| `FOUNDATIONX_OSSX_BUCKET` | 是 | bucket 名 |
| `FOUNDATIONX_OSSX_ACCESS_KEY_ID` | 是 | AccessKeyId |
| `FOUNDATIONX_OSSX_ACCESS_KEY_SECRET` | 是 | AccessKeySecret |
| `FOUNDATIONX_OSSX_REGION` | 否 | 默认 `ap-northeast-1` |
| `FOUNDATIONX_OSSX_REQUEST_TIMEOUT_MS` | 否 | 单请求超时，默认 30000 |
| `FOUNDATIONX_OSSX_OPERATION_DEADLINE_MS` | 否 | 含重试的总 deadline，默认 90000 |
| `FOUNDATIONX_OSSX_ACQUIRE_TIMEOUT_MS` | 否 | in-flight 许可超时，默认 5000 |
| `FOUNDATIONX_OSSX_MAX_IN_FLIGHT` | 否 | 默认 64，硬上界 1024 |
| `FOUNDATIONX_OSSX_MAX_OBJECT_BYTES` | 否 | 默认 512 MiB；当前 Bytes API 还受缓冲上限约束 |
| `FOUNDATIONX_OSSX_MAX_BUFFER_BYTES` | 否 | 默认/硬上界 512 MiB |
| `FOUNDATIONX_OSSX_MAX_ERROR_BODY_BYTES` | 否 | 默认 64 KiB，硬上界 1 MiB |

远程 endpoint 必须是 HTTPS；HTTP 仅允许 loopback 开发端点。`Debug` 对 secret 脱敏。

## Multipart 边界

- 非末片至少 100 KiB，单片至多 512 MiB，总片数至多 10000。
- 对象 key 最长 1023 个 UTF-8 字节，避免 URL/审计记录无界增长。
- Complete ETag 会进行 XML escaping；重复/越界 part number 会 fail-closed。
- part/complete 失败会尝试 abort；abort 也失败时返回含 `orphan_risk=true` 的 `Conflict`。
- 高层 multipart 全程共享一个总 deadline。drop future 后，RAII guard 会把 key/UploadId 写入
  最多 1024 条的进程内 orphan 审计注册表；`multipart_orphan_audits()` 可取回并调用
  `abort_multipart()` 补偿，成功后记录自动移除。队列溢出通过计数器保持可见。
- lifecycle、STS、流式 TB 对象与 package stable 仍为 OPEN。

## Scaffold（可选 feature）

```toml
ossx = { path = "...", features = ["scaffold"] }
```

`OssAdapter` 为进程内 HashMap，**不是**生产客户端。

## Live 测试

对象 key 前缀：`infra-draft/`。网络/鉴权失败时测试 **panic 暴露真实错误**，不 mock 通过。

## Bench

```bash
cargo bench -p ossx --bench put_get
```

文档：[docs/usage.md](docs/usage.md) · [docs/config.md](docs/config.md) · [docs/operations.md](docs/operations.md)
