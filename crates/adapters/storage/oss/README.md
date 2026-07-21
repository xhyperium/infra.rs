# ossx

阿里云 OSS 对象存储适配 — 实现 `contracts::ObjectStore`。

## 生产入口（默认）

| 类型 | 说明 |
|------|------|
| `OssConfig` / `OssConfigBuilder` | 配置；`OssConfig::from_env()` 读 `FOUNDATIONX_OSSX_*` |
| `OssClient` | `connect` / `from_env` / `put_object` / `get_object` / `delete_object` / `close` |
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

`Debug` 对 secret 脱敏（`<redacted>`）。

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
