# taosx 配置

## 环境变量前缀

`FOUNDATIONX_TAOSX_*`

通过 `from_env()` / `connect_from_env()` 加载。缺省字段见 crate `config` 模块文档。

| 变量 | 默认 | 说明 |
|------|------|------|
| `HOST` | `127.0.0.1` | 禁止 scheme/userinfo/path |
| `PORT` | `6041` | REST；native 6030 不用于 SQL |
| `DATABASE` | `infra_draft` | 连接时 `CREATE DATABASE IF NOT EXISTS` |
| `USER` / `PASSWORD` | `root` / 空 | 远程必须非空密码 |
| `TLS` | false | 远程必须 true |
| `TIMEOUT_MS` | 10000 | 请求超时 |
| `PRECISION` | 探测 | 可选强制 `ms`/`us`/`ns` |
| `TRANSPORT` | rest | `rest` 或 `ws`/`native`（仅探测） |
| `MAX_IN_FLIGHT` | 64 | 硬上限 1024 |
| `BATCH_MAX_ROWS` / `BATCH_MAX_BYTES` | 500 / 1MiB | 硬上限 10000 / 8MiB |
| `MAX_RESPONSE_BYTES` / `MAX_QUERY_ROWS` | 8MiB / 10000 | 硬上限 64MiB / 100000 |
| `CLOSE_TIMEOUT_MS` | 5000 | 硬上限 30000 |
| `ACQUIRE_TIMEOUT_MS` | 5000 | 池许可获取超时 |

## 安全

- 密码 / AccessKey **不得**写入源码、日志、Debug 明文
- 远程地址强制 TLS 且 user/password 非空；仅严格 loopback 可明文
- host 不允许 scheme、userinfo、路径、query 或 fragment；REST 禁止 redirect
- 凭据：`scripts/live/export-foundationx-env.sh --env dev -- <cmd>` 注入子进程
- 凭据轮换：更新 secret provider / 环境变量后重启进程

## 校验

非法/空白环境值、零 deadline 或超过 `HARD_MAX_*` 的资源值在 `connect` 前 fail-fast
（`ErrorKind::Invalid`）。

NativeWs 只做 WSS/WS 握手可达性探测，不证明认证。
