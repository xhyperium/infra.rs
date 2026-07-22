# kafkax 配置

## 环境变量前缀

`FOUNDATIONX_KAFKAX_*`

通过 `from_env()` / `connect_from_env()` 加载。缺省字段见 crate `config` 模块文档。

## 安全

- 密码 / AccessKey **不得**写入源码、日志、Debug 明文
- 当前 `rskafka` 构建未接入 TLS；`TLS=true` 会 fail-closed
- 需要 TLS 的部署在能力补齐并通过 live 证据前为 **NO-GO**，不得静默改用明文
- 凭据轮换：更新 secret provider / 环境变量后重启进程

## 校验

非法配置在 `connect` 前 fail-fast（`ErrorKind::Invalid`）。
