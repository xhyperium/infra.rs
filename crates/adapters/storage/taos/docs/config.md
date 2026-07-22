# taosx 配置

## 环境变量前缀

`FOUNDATIONX_TAOSX_*`

通过 `from_env()` / `connect_from_env()` 加载。缺省字段见 crate `config` 模块文档。

## 安全

- 密码 / AccessKey **不得**写入源码、日志、Debug 明文
- 远程地址强制 TLS 且 user/password 非空；仅严格 loopback 可明文
- host 不允许 scheme、userinfo、路径、query 或 fragment；REST 禁止 redirect
- 凭据轮换：更新 secret provider / 环境变量后重启进程

## 校验

非法/空白环境值、零 deadline 或超过 `HARD_MAX_*` 的资源值在 `connect` 前 fail-fast
（`ErrorKind::Invalid`）。

资源默认/硬上限见 crate README。NativeWs 只做 WSS/WS 握手可达性探测，不证明认证。
