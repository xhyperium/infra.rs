# kafkax 配置

## 环境变量前缀

`FOUNDATIONX_KAFKAX_*`

通过 `from_env()` / `connect_from_env()` 加载。缺省字段见 crate `config` 模块文档。

也可使用 `KafkaConfigBuilder` 链式构建（`build()` 调用 `validate`）。

## 安全

- SASL 用户名、密码 / AccessKey **不得**写入源码、日志、Debug 明文
- 远程 broker 必须 `TLS=true`；明文仅允许 loopback
- `TLS_CA_FILE` 可追加 PEM CA；未设置时使用 webpki roots
- SASL 仅支持 PLAIN；未知机制或凭据不完整会 fail-closed
- `CONNECT_TIMEOUT_MS` / `OPERATION_TIMEOUT_MS` 必须大于零
- `delivery_timeout` 使用配置值作为精确 produce deadline，不会静默放大
- 凭据轮换：更新 secret provider / 环境变量后重启进程

## 校验

非法配置在 `connect` 前 fail-fast（`ErrorKind::Invalid`）。

固定摘要 SASL_SSL 证据：`node scripts/kafka-tls-sasl-conformance.mjs`。
