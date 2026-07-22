# clickhousex 配置

## 环境变量前缀

`FOUNDATIONX_CLICKHOUSEX_*`

通过 `from_env()` / `connect_from_env()` 加载。缺省字段见 crate `config` 模块文档。

端口主变量为 `HTTP_PORT`，兼容旧别名 `PORT`；只设置 `PORT` 时生效，同时设置且
值不一致时拒绝启动，避免配置漂移。

## 安全

- 密码 / AccessKey **不得**写入源码、日志、Debug 明文
- 远程地址必须 `TLS=true`；HTTP 仅允许 loopback
- `TLS_CA_FILE` 可追加 PEM CA；CA 仅在 TLS 模式允许
- 无效布尔值、端口、容量与零 deadline 均返回 `Invalid`
- 凭据轮换：更新 secret provider / 环境变量后重启进程
- HTTP 错误上下文不包含服务端正文，仅保留状态与可选数字错误码

## 校验

非法配置在 `connect` 前 fail-fast（`ErrorKind::Invalid`）。

HTTPS 客户端证据：`node scripts/clickhouse-https-conformance.mjs`。该实验不是集群 live。
