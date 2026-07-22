# redisx 实现规范

状态：当前 `0.3.3` active 合同。生产默认命令通道已实现；package stable、真实 Cluster、
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
| GET / EXISTS / PTTL / MGET | 仅 Transient + budget | 单命令；MGET 仅单节点/同 slot | 可安全重读 |
| SET / PSETEX | 默认否；仅 `set_with_budget` 显式 opt-in | value + TTL 单命令原子 | 可能已写入；重试重置 TTL |
| DEL | 否 | 单命令原子 | 可能已删除；重试返回值漂移 |
| PEXPIRE | 否 | 单命令原子 | 可能已生效；重试重置 TTL 起点 |
| MSET | 否 | Standalone/Cluster 同 slot 单命令 | 跨 slot 不承诺原子性 |
| PUBLISH | 否 | 无可靠投递原子性 | 重试可能重复消息，仍可能丢消息 |

`RedisOperation::{retry_safety,atomicity}` 是可测试合同。客户端 deadline/断连只表示没有收到
确定响应，不能证明服务端未执行写命令。非 retryable 错误只能尝试一次且不得消耗 retry budget。

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

真实 Cluster / Sentinel / TLS live 未执行时，对应矩阵保持 OPEN；不得以编译、构造测试、连接
拒绝测试或 ignored 入口替代真实拓扑证据。
