# redisx 运维说明

## 生命周期

1. `RedisPool::connect`：建立 `ConnectionManager`（自动重连）并可选 `CLIENT SETNAME`
2. `pool.client()`：派生廉价 `Clone` 句柄
3. 命令路径：`acquire(Semaphore)` → in-flight++ → 命令超时 → in-flight--
4. `pool.close(deadline)`：置 closed，拒绝新请求，排空 in-flight

## 健康检查

| 级别 | 做法 |
|------|------|
| liveness | 进程 / 任务存活（应用侧） |
| readiness | `pool.ping()` 成功且 `stats().open == 1` |
| diagnostics | `stats()` + 脱敏 `endpoint()`（低频） |

## 指标建议（低基数）

- `redisx_inflight` / `redisx_waiters` / `redisx_open`
- 命令计数与延迟：按 `operation` + `outcome`，**禁止** key / channel / 完整 endpoint

## 故障行为

| 场景 | 行为 |
|------|------|
| 池耗尽 | `DeadlineExceeded`（acquire 超时） |
| 命令慢 | `DeadlineExceeded`（command 超时） |
| 短暂断连 | `ConnectionManager` 重连；调用方见 Transient/Unavailable |
| 认证失败 | `Unavailable` |
| `close` 后 | 新请求 `Unavailable` |

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

## 非目标（P0）

- Cluster / Sentinel 路由
- Streams / 分布式锁完整合同
- 默认自动重试非幂等写
