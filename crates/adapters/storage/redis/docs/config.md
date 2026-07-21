# redisx 配置

## 环境变量

| 变量 | 默认 | 说明 |
|------|------|------|
| `REDIS_URL` | （空） | 若设置则**覆盖**下列分项（含 scheme / 认证 / db） |
| `FOUNDATIONX_REDISX_ADDR` | `127.0.0.1:6379` | `host:port` |
| `FOUNDATIONX_REDISX_USERNAME` | `default` | ACL 用户；空串表示不发 username |
| `FOUNDATIONX_REDISX_PASSWORD` | （空） | 密码；**勿**提交到 git / 日志 |
| `FOUNDATIONX_REDISX_DB` | `0` | 逻辑库 |
| `FOUNDATIONX_REDISX_TLS` | `false` | `true/false`；P0 构建未链 tls feature 时 connect 返回 Invalid |

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
| `mode` | Standalone | Cluster/Sentinel P0 拒绝 |

## 安全

- 禁止把密码写入配置仓库、日志、metrics label、错误明文以外的可导出结构
- 生产建议最小 ACL；KV 与 Pub/Sub 可用不同用户
- TLS：当前 workspace `redis` 依赖未启用 tls feature；明文需显式 `tls=false`
