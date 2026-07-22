# adapters/storage/kafka — Design（infra.rs）

## 模块形状

| 组件 | 职责 |
|------|------|
| Config | `FOUNDATIONX_KAFKAX_{BROKERS,SASL_MECHANISM,SASL_USERNAME,SASL_PASSWORD,TLS}` / builder；密码 Debug 脱敏 |
| Pool / Client | 连接、超时、健康、close 语义 |
| Error map | 驱动错误 → `kernel::XError` / `ErrorKind` |
| contracts 适配 | contracts::EventBus（at-most-once） |
| scaffold feature | 进程内 mock / 旧适配器（非默认） |

## 依赖方向

```text
kafkax → kernel + contracts（+ 驱动 crate）
禁止  kernel/types 反向依赖 adapters
```

## 不变量

1. 默认 feature = 生产路径；scaffold 可选
2. 外部 I/O 有 timeout / close 路径
3. 无硬编码生产密钥
4. 公共 API 中文文档 + 英文标识符

## 参考

- 实现：`crates/adapters/storage/kafka/src/`
- 用法：`crates/adapters/storage/kafka/docs/usage.md` · `config.md` · `operations.md`
