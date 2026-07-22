# ossx 配置

## 环境变量前缀

`FOUNDATIONX_OSSX_*`

通过 `OssConfig::from_env()` / `OssClient::from_env()` 加载。

除 endpoint/bucket/AccessKey 外，还可配置 `REQUEST_TIMEOUT_MS`、
`OPERATION_DEADLINE_MS`、`ACQUIRE_TIMEOUT_MS`、`MAX_IN_FLIGHT`、
`MAX_OBJECT_BYTES`、`MAX_BUFFER_BYTES` 与 `MAX_ERROR_BODY_BYTES`。所有值在构建期校验；
零值或超过 crate 导出 `HARD_MAX_*` 的值会 fail-closed。

## 安全

- 密码 / AccessKey **不得**写入源码、日志、Debug 明文
- 远程 endpoint 强制 HTTPS；HTTP 仅允许 loopback 开发端点
- endpoint 禁止 userinfo、path、query 与 fragment
- 凭据轮换：更新 secret provider / 环境变量后重启进程

## 校验

非法配置在 `connect` 前 fail-fast（`ErrorKind::Invalid`）。当前凭据来自环境变量；STS 临时凭据
未落地，保持 OPEN。
