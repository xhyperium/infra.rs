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
| `FOUNDATIONX_POSTGRESX_ACQUIRE_TIMEOUT_MS` | 池等待截止时间，默认 `5000` | 否 |
| `FOUNDATIONX_POSTGRESX_OPERATION_TIMEOUT_MS` | SQL/事务截止时间，默认 `10000` | 否 |
| `FOUNDATIONX_POSTGRESX_TLS_CA_FILE` | 额外 PEM CA/服务端证书路径（叠加 webpki 公共根） | 否 |
| `FOUNDATIONX_POSTGRESX_TLS_SERVER_NAME` | TLS SNI/证书校验名（host 为 IP 且证书为 DNS CN 时必填） | 否 |

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

## TLS 策略

- loopback / Unix socket 可显式 `disable`；`prefer` 也只允许本机
- 远程地址必须 `require`，否则连接前 `Invalid`
- `require` 使用 rustls + webpki 公共根；可通过 `TLS_CA_FILE` **叠加**企业/自签 PEM（仍强制校验，无 insecure 旁路）
- host 为 IP 且证书 CN/SAN 为 DNS 名时，设置 `TLS_SERVER_NAME`（内部 `hostaddr=IP` + `host=server_name`）
- 本版仍不支持 mTLS 客户端证书

## 安全

- **禁止**把密码、完整 `DATABASE_URL` 写入 git / 日志
- `PostgresConfig` 的 `Debug` 对 password / URL 脱敏
- 生产密钥走环境或 secret provider，不进配置仓库
- `operation_timeout` 同时下发 PostgreSQL `statement_timeout`；客户端 deadline 仍是最终边界

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
