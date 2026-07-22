# adapters/storage/oss — Design（infra.rs）

## 模块形状

| 组件 | 职责 |
|------|------|
| Config | 凭据 + timeout/deadline + 对象/缓冲/错误体/in-flight 上界；远程仅 HTTPS；Debug 脱敏 |
| Pool / Client | 共享 reqwest pool + Semaphore 背压 + acquire timeout + close/cancel 语义 |
| Error map | 驱动错误 → `kernel::XError` / `ErrorKind` |
| Multipart | 全程共享 deadline；RAII orphan registry；part/count/ETag XML；失败 abort |
| contracts 适配 | contracts::ObjectStore |
| scaffold feature | 进程内 mock / 旧适配器（非默认） |

## 依赖方向

```text
ossx → kernel + contracts（+ 驱动 crate）
禁止  kernel/types 反向依赖 adapters
```

## 不变量

1. 默认 feature = 生产路径；scaffold 可选
2. 外部 I/O 有 timeout / close 路径
3. 无硬编码生产密钥
4. 公共 API 中文文档 + 英文标识符
5. Initiate / Complete 响应不确定时不自动重放
6. live 未运行、STS/lifecycle 未落地时禁止宣称 package stable
7. orphan registry 至多 1024 条；溢出计数保持可观测，Debug 不暴露 key/UploadId

## 参考

- 实现：`crates/adapters/storage/oss/src/`
- 用法：`crates/adapters/storage/oss/docs/usage.md` · `config.md` · `operations.md`
