# adapters/storage/taos — Design（infra.rs）

## 模块形状

| 组件 | 职责 |
|------|------|
| Config | 远程 TLS/auth fail-closed；资源参数受 `HARD_MAX_*` 限制；密码 Debug 脱敏 |
| Pool / Client | REST SQL、响应/请求/查询上界、RAII in-flight、deadline drain close |
| WS probe | `/rest/ws` 有界握手与关闭；不执行 SQL、不证明认证 |
| Error map | 驱动错误 → `kernel::XError` / `ErrorKind` |
| contracts 适配 | contracts::TimeSeriesStore（ts 纳秒 epoch） |
| scaffold feature | 进程内 mock / 旧适配器（非默认） |

## 依赖方向

```text
taosx → kernel + contracts（+ 驱动 crate）
禁止  kernel/types 反向依赖 adapters
```

## 不变量

1. 默认 feature = 生产路径；scaffold 可选
2. 外部 I/O 有 timeout / close 路径
3. 无硬编码生产密钥
4. 公共 API 中文文档 + 英文标识符
5. Decimal 只以 NCHAR(64+) 文本落库；存量 DOUBLE schema 拒绝
6. Native SQL、HA 集群全矩阵与 package stable 保持 NO-GO；幂等重试（`RetryPolicy`）与 HA-lite（`hosts`）已有 live 证据

## 参考

- 实现：`crates/adapters/storage/taos/src/`
- 用法：`crates/adapters/storage/taos/docs/usage.md` · `config.md` · `operations.md`
