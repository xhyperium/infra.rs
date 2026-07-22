# redisx 配置

## 环境变量

| 变量 | 默认 | 说明 |
|------|------|------|
| `REDIS_URL` | （空） | 若设置则**覆盖**下列分项（含 scheme / 认证 / db） |
| `FOUNDATIONX_REDISX_ADDR` | `127.0.0.1:6379` | `host:port` |
| `FOUNDATIONX_REDISX_USERNAME` | `default` | ACL 用户；空串表示不发 username |
| `FOUNDATIONX_REDISX_PASSWORD` | （空） | 密码；**勿**提交到 git / 日志 |
| `FOUNDATIONX_REDISX_DB` | `0` | 逻辑库 |
| `FOUNDATIONX_REDISX_TLS` | `false` | `true/false`；`true` 使用证书校验 TLS，拒绝 insecure |
| `FOUNDATIONX_REDISX_MODE` | `standalone` | `standalone` / `cluster` / `sentinel` |
| `FOUNDATIONX_REDISX_NODES` | （空） | Cluster / Sentinel 逗号分隔种子；空则回退 `ADDR` |
| `FOUNDATIONX_REDISX_SENTINEL_MASTER` | （空） | Sentinel 服务名；Sentinel 模式必填 |

## URL 示例

```text
redis://127.0.0.1:6379/0
redis://default:SECRET@127.0.0.1:6379/0
```

密码仅经 `RedisConfig` 私有字段持有；`Debug` 与 `display_endpoint()` 输出 `***`。

## Builder 调优项

| 方法 | 默认 | 含义 |
|------|------|------|
| `connect_timeout` | 5s | 建连超时 |
| `command_timeout` | 3s | 单命令超时（含网络） |
| `acquire_timeout` | 3s | 获取 in-flight 许可超时 |
| `max_in_flight` | 256 | Semaphore 上限 |
| `client_name` | None | `CLIENT SETNAME` |
| `mode` | Standalone | 命令通道可选 Cluster / Sentinel；其真实 live 证据仍 OPEN |

## 安全

- 禁止把密码写入配置仓库、日志、metrics label、错误明文以外的可导出结构
- `NODES` 中即使使用带 userinfo 的 Redis URL，`Debug`、`endpoint()` 与配置错误也必须隐藏密码
- 生产建议最小 ACL；若 KV 与 Pub/Sub 共用 `RedisPool`，两者严格复用同一 ACL / TLS 配置
- Pub/Sub 当前仅接受 Standalone；Cluster / Sentinel 不会静默使用种子节点
- TLS 使用 `tokio-rustls-comp` 安全校验路径；当前没有真实 TLS live 证据，不得宣称已通过生产握手
