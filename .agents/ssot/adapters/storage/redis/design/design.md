# adapters/storage/redis — Design（infra.rs）

## 模块形状

| 组件 | 职责 |
|------|------|
| Config | `FOUNDATIONX_REDISX_{ADDR,USERNAME,PASSWORD,DB,TLS,MODE,NODES,SENTINEL_MASTER}` / builder；密码 Debug 脱敏 |
| Pool / Client | 连接、超时、健康、close、multi-lane 池语义 |
| Error map | 驱动错误 → `kernel::XError` / `ErrorKind` |
| contracts 适配 | contracts::KeyValueStore（+ 可选 pubsub） |
| Structures | hash/list/set/sorted-set 一等 API |
| Streams | xadd/xadd_with_id/xread_block/xrange |
| Transactions | multi/exec/discard/watch；eval_sha Lua CAS |
| selfcheck | 11 项 Full check（0.3.15） |
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
6. client 配置 budget 后，`ReadOnly` 与固定输入 `Idempotent` 操作进入安全重试；
   `UnsafeSideEffect` 的多次尝试在 I/O 前拒绝，PUBLISH 永不自动重试

## 重试分类设计

- GET / EXISTS / PTTL / MGET → `ReadOnly`；无 TTL SET / MSET → `Idempotent`。
- 相对 TTL SET / DEL / PEXPIRE → `UnsafeSideEffect`；`max_attempts > 1` 在 operation future / driver 前拒绝。
- PUBLISH → `NeverAutomatic`，不接入自动预算重试。
- `RedisOperation::Set` 同时代表 SET 与 PSETEX，无法携带 TTL 参数，故查询面保守保持
  `AmbiguousWrite`；`RedisClient::set` 以实际 `ttl` 参数决定 `Idempotent` 或 `UnsafeSideEffect`。

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
