# adapters/storage/clickhouse — Design（infra.rs）

## 模块形状

| 组件 | 职责 |
|------|------|
| Config | `FOUNDATIONX_CLICKHOUSEX_{HOST,HTTP_PORT,PORT,USER,PASSWORD,DATABASE}`；`HTTP_PORT` 优先且冲突 fail-closed；密码 Debug 脱敏 |
| Pool / Client | HTTP(S)、PEM CA、连接/请求/获取截止时间、健康、close 语义 |
| Error map | HTTP 状态 + 固定 ClickHouse 数字码 → `kernel::XError`；4096 字节解析上限；正文不外泄 |
| contracts 适配 | contracts::AnalyticsSink |
| scaffold feature | 进程内 mock / 旧适配器（非默认） |

## 依赖方向

```text
clickhousex → kernel + contracts（+ 驱动 crate）
禁止  kernel/types 反向依赖 adapters
```

## 不变量

1. 默认 feature = 生产路径；scaffold 可选
2. 外部 I/O 有 timeout / close 路径
3. 无硬编码生产密钥
4. 公共 API 中文文档 + 英文标识符
5. 远程 HTTP 不得静默降级；错误不得携带 SQL、payload 或认证正文

## 参考

- 实现：`crates/adapters/storage/clickhouse/src/`
- 用法：`crates/adapters/storage/clickhouse/docs/usage.md` · `config.md` · `operations.md`
