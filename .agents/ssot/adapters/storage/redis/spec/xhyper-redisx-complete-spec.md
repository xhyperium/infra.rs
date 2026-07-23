# redisx 实现规范

状态：当前 `0.3.5` active 合同。生产默认命令通道已实现；package stable、真实 Cluster、
Sentinel、TLS 与拓扑故障切换均未宣称通过。

## 1. 职责与范围

- 实现 `contracts::KeyValueStore`，生产默认类型为 `RedisPool` / `RedisClient`。
- `RedisMode::{Standalone,Cluster,Sentinel}` 均有命令连接代码路径；TLS 使用安全证书校验。
- `pubsub` feature 提供 `RedisPubSub` / `RedisPubSubFacade`，当前只支持 Standalone。
- `scaffold` feature 隔离进程内实现，禁止作为生产 Redis。

## 2. 配置与拓扑合同

| 能力 | 实现状态 | 证据状态 |
|------|----------|----------|
| Standalone 命令通道 | `ConnectionManager` | 既有 KV live；ACL 专项仍 OPEN |
| Cluster 命令通道 | `ClusterClient` / `ClusterConnection` | 仅离线配置与拒绝连接测试；真实 live OPEN |
| Sentinel 命令通道 | `async_master_for` 后连接 master | 仅离线配置与拒绝连接测试；真实 live/failover OPEN |
| TLS | `TcpTls { insecure: false }` / `TlsMode::Secure` | 构造测试 PASS；真实握手 OPEN |
| Pub/Sub | Standalone 专用订阅连接 + publish manager | live 测试默认 ignore；重连/必达 NO-GO |

`RedisPool` 保存建池时的完整 `RedisConfig`。`pool.subscribe` 必须复用该配置的端点、ACL、
db、TLS 与 timeout，禁止重新读取环境变量。Pub/Sub 在 Cluster / Sentinel 模式下必须在网络
I/O 前返回 `Invalid`；禁止静默降级到静态 Standalone 节点或把 Sentinel 种子当 master。

## 3. deadline、关闭与重连

- acquire 和 command timeout 分离；命令路径受池 Semaphore 背压。
- Standalone/Sentinel master 的 `ConnectionManager` 可自动重连，但本仓没有恢复/切换 live 证据。
- Pub/Sub 建连、订阅与 publish manager 使用同一配置中的 connect/command timeout。
- `close` 后拒绝新请求，并在调用方 deadline 内排空已计入的命令；Pub/Sub 独立会话不纳入该
  in-flight 计数。

## 4. 重试与原子性合同

| 操作 | 自动重试 | 原子性边界 | 响应丢失风险 |
|------|----------|------------|--------------|
| GET / EXISTS / PTTL / MGET | 配置 budget 时仅对 Transient 失败按预算安全重试 | 单命令；MGET 仅单节点/同 slot | `ReadOnly`，可安全重读 |
| 无 TTL SET | 配置 budget 时按预算安全重试 | 单命令原子 | 固定 key/value 为 `Idempotent`；响应仍可能丢失 |
| 相对 TTL SET / PSETEX | `max_attempts > 1` 时在 I/O 前拒绝；单次允许 | value + TTL 单命令原子 | 重试会重置 TTL 起点，按 `UnsafeSideEffect` 处理 |
| DEL | `max_attempts > 1` 时在 I/O 前拒绝；单次允许 | 单命令原子 | 重试会使返回值漂移，按 `UnsafeSideEffect` 处理 |
| PEXPIRE | `max_attempts > 1` 时在 I/O 前拒绝；单次允许 | 单命令原子 | 重试会重置 TTL 起点，按 `UnsafeSideEffect` 处理 |
| MSET | 配置 budget 时按预算安全重试 | Standalone/Cluster 同 slot 单命令 | 固定输入为 `Idempotent`；跨 slot 不承诺原子性 |
| PUBLISH | 不自动重试 | 无可靠投递原子性 | 重试可能重复消息，仍可能丢消息 |

`RedisOperation::{retry_safety,atomicity}` 是可测试的粗粒度合同。`RedisOperation::Set` 同时代表无 TTL
SET 与相对 TTL PSETEX，无法表达参数差异，因此保守保持 `AmbiguousWrite`；真实 `RedisClient::set`
按 `ttl` 参数细分：`None` 使用 `Idempotent`，`Some(_)` 使用 `UnsafeSideEffect`。客户端 deadline/断连
只表示没有收到确定响应，不能证明服务端未执行写命令。非 retryable 错误只能尝试一次且不得消耗
retry budget；PUBLISH 不进入自动预算重试。

## 5. 安全边界

- 密码仅来自显式配置/env，所有 `Debug` / endpoint 输出必须脱敏。
- `nodes` 中的 URL userinfo 同样必须在 endpoint 与配置错误中脱敏。
- TLS 只允许证书校验模式，拒绝 insecure URL。
- key、channel、payload、密码不得进入 metrics label 或诊断日志。
- live 测试默认 `#[ignore]`；不得读取、打印或提交真实凭据。

## 6. 验收

```bash
cargo fmt --all -- --check
cargo clippy -p redisx --all-targets --features pubsub -- -D warnings
cargo test -p redisx --all-targets --features pubsub
node scripts/quality-gates/check-workspace-deps.mjs
cmp .agents/ssot/adapters/storage/redis/spec/spec.md \
  .agents/ssot/adapters/storage/redis/spec/xhyper-redisx-complete-spec.md
```

当前最终本地结果：51 passed + 8 ignored；ignored live 测试需要外部 Redis，不作为默认 CI 通过证据。

真实 Cluster / Sentinel / TLS live 未执行时，对应矩阵保持 OPEN；不得以编译、构造测试、连接
拒绝测试或 ignored 入口替代真实拓扑证据。

## 7. 三轮加固（0.3.5）错误分类锚点

- `map_redis_error` 必须稳定映射：LOADING/IO→Transient；认证→Unavailable；ClusterDown→Unavailable；
  MOVED/ASK→Transient；ExecAbort→Conflict；NoScript→Missing。
- **OPEN 不变**：真实 Cluster / Sentinel / TLS live。
