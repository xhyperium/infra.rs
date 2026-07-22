# adapters/storage/redis — Design（infra.rs）

## 模块形状

| 组件 | 职责 |
|------|------|
| Config | `FOUNDATIONX_REDISX_{ADDR,USERNAME,PASSWORD,DB,TLS,MODE,NODES,SENTINEL_MASTER}` / builder；密码 Debug 脱敏 |
| Pool / Client | 连接、超时、健康、close 语义 |
| Error map | 驱动错误 → `kernel::XError` / `ErrorKind` |
| contracts 适配 | contracts::KeyValueStore（+ 可选 pubsub） |
| scaffold feature | 进程内 mock / 旧适配器（非默认） |

## 依赖方向

```text
redisx → kernel + contracts（+ 驱动 crate）
禁止  kernel/types 反向依赖 adapters
```

## 不变量

1. 默认 feature = 生产路径；scaffold 可选
2. 外部 I/O 有 timeout / close 路径
3. 无硬编码生产密钥
4. 公共 API 中文文档 + 英文标识符
5. Pub/Sub 复用建池配置；禁止从 env 重建或拓扑降级
6. 自动预算重试仅用于只读操作；写重试必须显式 opt-in

## 拓扑与证据

| 路径 | 设计 | 证据裁定 |
|------|------|----------|
| Standalone | ConnectionManager | KV live 入口存在 |
| Cluster | ClusterConnection | 真实 live OPEN |
| Sentinel | 发现 master 后 ConnectionManager | 真实 live/failover OPEN |
| TLS | rustls 安全校验 | 真实握手 OPEN |
| Pub/Sub | Standalone 专用连接 | Cluster/Sentinel NO-GO |

## 参考

- 实现：`crates/adapters/storage/redis/src/`
- 用法：`crates/adapters/storage/redis/docs/usage.md` · `config.md` · `operations.md`
