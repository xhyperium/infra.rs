# taosx 运维

## 健康检查

- **liveness**：进程存活即可
- **readiness**：调用池/客户端 `health`/`ping`（有 deadline）

## 故障

| 症状 | 处理 |
|------|------|
| connect 失败 | 检查 FOUNDATIONX_TAOSX_* 与网络/认证 |
| DeadlineExceeded | 调高 timeout；查下游慢查询/背压 |
| Unavailable | 下游重启/鉴权；观察重连日志 |

## 升级 / 回滚

1. 发布前跑 `cargo test -p taosx` 与 live（如可达）
2. 升级：先滚动 canary 实例，观察错误率与延迟
3. 回滚：回退至上一 crate 版本；配置 schema 保持向后兼容（仅新增字段）

## 关闭

调用 `close()`：原子拒绝新请求，并在 `CLOSE_TIMEOUT_MS` 内等待 RAII in-flight 排空；
超时返回 `DeadlineExceeded`，池保持 closed，重复 `close()` 可继续等待。

## 数据兼容

- bid/ask 必须为 `NCHAR(64+)`；检测到旧 `DOUBLE` stable 时拒绝写查，需由受控迁移处理
- 单次响应、SQL/batch、query rows 与并发均有配置上限和不可突破的编译期硬上限
- 多 chunk 写入不做内部重试；部分成功后的整批幂等重试尚无证据，保持 NO-GO
