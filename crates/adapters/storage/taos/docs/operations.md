# taosx 运维

## 健康检查

- **liveness**：`TaosPool::liveness()` — 仅看本地池是否仍接受请求（不访问网络）
- **readiness**：`TaosPool::health().await` — 本地 open + 有 deadline 的 `SELECT SERVER_VERSION()`
  - 返回 `TaosHealth { ready, precision, server_version, stats, metrics, detail }`
  - 未就绪时 `ready=false` 且 `Ok(...)`（非 `Err`），便于编排探针
  - `detail` 为中文短句，不含密钥

## 故障

| 症状 | 处理 |
|------|------|
| connect 失败 | 检查 `FOUNDATIONX_TAOSX_*` 与网络/认证；远程须 TLS |
| health.ready=false | 看 `detail`；查下游/鉴权/超时 |
| DeadlineExceeded | 调高 timeout；查下游慢查询/in-flight 饱和 |
| Unavailable | 下游重启/鉴权；观察 tracing / metrics |
| Conflict（schema） | 存量 DOUBLE stable 需受控迁移为 NCHAR(64+) |
| Invalid（配置） | 校验 host/port/HARD_MAX 与精度声明 |

## Live 与凭据

1. 凭据仅来自 secret provider / `ZoneCNH/sre/secrets/env/dev.md`（经脚本解析）
2. 推荐：`scripts/live/export-foundationx-env.sh --env dev -- <cmd>`
3. 脚本将 REST 端口固定为 **6041**（非 native 6030）
4. **禁止**把密码写入 git、PR、日志、Debug 明文
5. prod 远程主机必须 TLS；本 crate 对远程明文 fail-closed

## 升级 / 回滚

1. 发布前：`cargo test -p taosx --all-targets` + live（如可达）+ clippy
2. 升级：先 canary，观察错误率与延迟
3. 回滚：回退 crate 版本；配置仅允许新增字段默认值

## 关闭

调用 `close()`：原子拒绝新请求，并在 `CLOSE_TIMEOUT_MS` 内等待 RAII in-flight 排空；
超时返回 `DeadlineExceeded`，池保持 closed，重复 `close()` 可继续等待。
关闭后 `liveness()==false` 且 `health().ready==false`。

## 数据兼容

- bid/ask 必须为 `NCHAR(64+)`；检测到旧 `DOUBLE` stable 时拒绝写查
- 单次响应、SQL/batch、query rows 与并发均有配置上限和编译期硬上限
- 多 chunk 写入不做内部重试；可用 `BatchWriteReport` 定位部分成功

## 诚实 NO-GO

- Native SQL / FFI / 6030 协议会话
- WebSocket SQL 长会话与认证证明（仅握手探测）
- HA / Cluster / leader failover 矩阵
- package stable / crates.io publish
- 24h soak / 远程 OTLP RED 导出
