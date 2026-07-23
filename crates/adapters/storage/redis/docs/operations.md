# redisx 运维说明

## 生命周期

1. `RedisPool::connect`：建立 `ConnectionManager`（自动重连）并可选 `CLIENT SETNAME`
2. `pool.client()`：派生廉价 `Clone` 句柄
3. 命令路径：`acquire(Semaphore)` → in-flight++ → 命令超时 → in-flight--
4. `pool.close(deadline)`：置 closed，拒绝新请求，排空 in-flight

## 健康检查

| 级别 | 做法 |
|------|------|
| liveness | `pool.liveness()`（未关闭）或进程 / 任务存活 |
| readiness | `pool.readiness().await`（内部 `ping`）成功；`stats().open == command_lanes()`（逻辑 lane 数 = `max_in_flight`，关闭时为 0） |
| diagnostics | `stats()` + `metrics_snapshot()` + 脱敏 `endpoint()` + `reconnect_max_delay()` / `tcp_keepalive()`（低频） |

## 指标（0.3.9）

内置进程内累计（`RedisPool::metrics_snapshot` → `RedisMetricsSnapshot`）：

| 字段 | 含义 |
|------|------|
| `commands_ok` | `with_conn*` 闭包 `Ok` |
| `commands_err` | 闭包 `Err` |
| `commands_timeout` | 命令预算耗尽 / 总 deadline 在 acquire 后耗尽 |
| `acquire_timeout` | 排队 acquire 超时（含 0 预算） |
| `rejected_closed` | 池已关闭拒绝 |

**不是** OpenTelemetry / Prometheus 导出器；宿主可采样后导出。建议低基数：

- `redisx_inflight` / `redisx_waiters` / `redisx_open`（来自 `stats()`）
- 命令计数：按 `outcome`，**禁止** key / channel / 完整 endpoint

## 故障行为

| 场景 | 行为 |
|------|------|
| 池耗尽 | `DeadlineExceeded`（acquire 超时） |
| 命令慢 | `DeadlineExceeded`（command 超时） |
| 短暂断连 | `ConnectionManager` 重连；调用方见 Transient/Unavailable |
| 认证失败 | `Unavailable` |
| `close` 后 | 新请求 `Unavailable` |

## 重试与原子性

| 操作 | 自动预算重试 | 原子性边界 | 失败后含义 |
|------|--------------|------------|------------|
| GET / EXISTS / PTTL / MGET | budget 下仅 Transient；`ReadOnly` | 单命令；MGET 仅单节点/同 slot | 可安全重读 |
| 无 TTL SET | budget 下仅 Transient；`Idempotent` | 单命令原子 | 固定 key/value 可重放；响应仍可能丢失 |
| 相对 TTL SET / PSETEX | 多次尝试在 I/O 前拒绝；单次允许 | value + TTL 单命令原子 | 重试会重置 TTL 起点 |
| DEL | 多次尝试在 I/O 前拒绝；单次允许 | 单命令原子 | 重试会使返回值漂移 |
| PEXPIRE | 多次尝试在 I/O 前拒绝；单次允许 | 单命令原子 | 重试会重置 TTL 起点 |
| MSET | budget 下仅 Transient；`Idempotent` | Standalone/Cluster 同 slot 单命令 | 固定输入可重放；不承诺跨 slot 原子性 |
| PUBLISH | 不自动重试 | 无可靠投递原子性 | 避免自动重复消息；仍可能丢消息 |

`RedisOperation::{retry_safety,atomicity}` 是粗粒度可测试入口。`RedisOperation::Set` 无法携带 TTL
参数，故保持 `AmbiguousWrite`；`RedisClient::set` 按 `ttl = None` / `Some(_)` 分别选择
`Idempotent` / `UnsafeSideEffect`。超时只说明客户端未收到确定结果，不能证明 Redis 未执行写命令。

## Pub/Sub 拓扑

- `pool.subscribe` 复用建池时的完整 `RedisConfig`，不重新读取环境变量。
- Standalone 继承相同端点、ACL、db、TLS、连接超时和响应超时。
- Cluster / Sentinel 当前返回 `Invalid`，不得降级到 Standalone 或把 Sentinel 种子当 master。
- 断线重订阅与消息必达没有实现证据，维持 NO-GO。
- `into_message_stream`：断线静默结束；`into_result_message_stream`：末尾一次 `Err(Unavailable)`（0.3.9）。

## Live 验证

```bash
export FOUNDATIONX_REDISX_ADDR=127.0.0.1:6379
export FOUNDATIONX_REDISX_USERNAME=default
export FOUNDATIONX_REDISX_PASSWORD=...   # 勿回显
export FOUNDATIONX_REDISX_DB=0
export FOUNDATIONX_REDISX_TLS=false

cargo test -p redisx -- --ignored
cargo bench -p redisx --bench kv_hot_path
```

CI：`.github/workflows/redisx-live.yml`（service redis；可用 `REDIS_URL`）。

## 尚未闭合

- 真实 Cluster / Sentinel / TLS live 与故障切换（无拓扑 env 时 soft-skip，禁止 mock 伪绿）
- package stable 宣告与 crates.io 发布
- Pub/Sub 自动重连/必达（产品 NO-GO；可靠通道用 Streams）
- 跨模块 SelfValidator / HTTP / Prometheus 导出框架（OOS；池内 `metrics_snapshot` 已有）

## 分布式锁（0.3.8）

- `lock_acquire(key, ttl)` → `RedisLock { token, fence }`
- 释放/续租用 Lua compare-and-*；**关键写必须携带并校验 fence 单调性**
- Redis 锁不是正确性银弹；见 draft §1.3
