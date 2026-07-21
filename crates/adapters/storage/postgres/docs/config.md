# postgresx 配置

## 环境变量

| 变量 | 说明 | 必填 |
|------|------|------|
| `DATABASE_URL` | `postgres://user:pass@host:port/db?sslmode=disable` | 可选；**若设置则优先** |
| `FOUNDATIONX_POSTGRESX_HOST` | 主机 | 无 URL 时必填 |
| `FOUNDATIONX_POSTGRESX_PORT` | 端口，默认 `5432` | 否 |
| `FOUNDATIONX_POSTGRESX_DATABASE` | 数据库名 | 无 URL 时必填 |
| `FOUNDATIONX_POSTGRESX_USER` | 用户 | 无 URL 时必填 |
| `FOUNDATIONX_POSTGRESX_PASSWORD` | 密码 | 否（默认空） |
| `FOUNDATIONX_POSTGRESX_SSLMODE` | `disable` / `prefer` / `require` | 否（默认 `disable`） |
| `FOUNDATIONX_POSTGRESX_MAX_POOL_SIZE` | 池上限，默认 `16` | 否 |
| `FOUNDATIONX_POSTGRESX_APPLICATION_NAME` | `application_name` | 否 |

加载入口：`PostgresConfig::from_env()`。

## 示例

```bash
export FOUNDATIONX_POSTGRESX_HOST=127.0.0.1
export FOUNDATIONX_POSTGRESX_PORT=5432
export FOUNDATIONX_POSTGRESX_DATABASE=app
export FOUNDATIONX_POSTGRESX_USER=app
export FOUNDATIONX_POSTGRESX_PASSWORD='***'
export FOUNDATIONX_POSTGRESX_SSLMODE=disable
```

或：

```bash
export DATABASE_URL='postgres://app:***@127.0.0.1:5432/app?sslmode=disable'
```

## TLS 限制（当前构建）

- 驱动：`tokio-postgres` + `NoTls` + `deadpool-postgres`
- `sslmode=disable`：支持
- `sslmode=prefer`：降级为 NoTls（文档诚实声明）
- `sslmode=require`（及 verify-*）：`connect` 返回 `ErrorKind::Invalid`，需后续 TLS feature

## 安全

- **禁止**把密码、完整 `DATABASE_URL` 写入 git / 日志
- `PostgresConfig` 的 `Debug` 对 password / URL 脱敏
- 生产密钥走环境或 secret provider，不进配置仓库

## 代码构建

```rust
use postgresx::PostgresConfig;

# fn demo() -> kernel::XResult<()> {
let cfg = PostgresConfig::builder()
    .host("127.0.0.1")
    .database("app")
    .user("app")
    .password(std::env::var("FOUNDATIONX_POSTGRESX_PASSWORD").unwrap_or_default())
    .build()?;
# let _ = cfg;
# Ok(())
# }
```
