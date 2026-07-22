# clickhousex 配置

## 环境变量前缀

`FOUNDATIONX_CLICKHOUSEX_*`

通过 `from_env()` / `connect_from_env()` 加载。缺省字段见 crate `config` 模块文档。

## 安全

- 密码 / AccessKey **不得**写入源码、日志、Debug 明文
- 远程地址必须 `TLS=true`；HTTP 仅允许 loopback
- `TLS_CA_FILE` 可追加 PEM CA；CA 仅在 TLS 模式允许
- 无效布尔值、端口、容量与零 deadline 均返回 `Invalid`
- 凭据轮换：更新 secret provider / 环境变量后重启进程

## 校验

非法配置在 `connect` 前 fail-fast（`ErrorKind::Invalid`）。

HTTPS 客户端证据：`node scripts/clickhouse-https-conformance.mjs`。该实验不是集群 live。
