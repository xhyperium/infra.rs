# clickhousex 配置

## 环境变量前缀

`FOUNDATIONX_CLICKHOUSEX_*`

通过 `from_env()` / `connect_from_env()` 加载。缺省字段见 crate `config` 模块文档。

## 安全

- 密码 / AccessKey **不得**写入源码、日志、Debug 明文
- 生产应启用 TLS（dev 可明文，须在 ops 文档标明风险）
- 凭据轮换：更新 secret provider / 环境变量后重启进程

## 校验

非法配置在 `connect` 前 fail-fast（`ErrorKind::Invalid`）。
