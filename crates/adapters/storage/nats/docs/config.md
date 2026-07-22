# natsx 配置

## 环境变量前缀

`FOUNDATIONX_NATS_*`

通过 `from_env()` / `connect_from_env()` 加载。缺省字段见 crate `config` 模块文档。

## 安全

- 密码 / AccessKey **不得**写入源码、日志、Debug 明文
- 远程地址必须 `TLS_POLICY=require`；disable/prefer 仅允许 loopback
- URL userinfo 被拒绝；用户名与密码使用独立环境字段
- `OPERATION_TIMEOUT_MS`、`SUBSCRIPTION_CAPACITY`、`CLIENT_CAPACITY`、
  `MAX_RECONNECTS`、`RECONNECT_MAX_DELAY_MS` 必须为非零有界值
- 凭据轮换：更新 secret provider / 环境变量后重启进程

## 校验

非法配置在 `connect` 前 fail-fast（`ErrorKind::Invalid`）。

有限 reconnect 配置同时表达恢复窗口与资源上限：固定入口在预算内恢复已通过真实 broker
实验；超过 `MAX_RECONNECTS` 后驱动关闭命令通道，调用方必须重建 client。
