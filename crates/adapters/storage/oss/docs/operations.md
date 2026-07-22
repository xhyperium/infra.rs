# ossx 运维

## 健康检查

- **liveness**：进程存活即可
- **readiness**：以受 deadline 约束的最小对象操作或部署侧探针验证；当前无公开 `health/ping` API

## 故障

| 症状 | 处理 |
|------|------|
| connect 失败 | 检查 FOUNDATIONX_OSSX_* 与网络/认证 |
| DeadlineExceeded | 调高 timeout；查下游慢查询/背压 |
| Unavailable | 下游重启/鉴权；观察重连日志 |
| Cancelled | 客户端已 close；禁止继续提交请求 |
| `orphan_risk=true` | abort 失败；从 `multipart_orphan_audits()` 取关联 ID 并补偿 |

## 升级 / 回滚

1. 发布前跑 `cargo test -p ossx` 与 live（如可达）
2. 升级：先滚动 canary 实例，观察错误率与延迟
3. 回滚：回退至上一 crate 版本；配置 schema 保持向后兼容（仅新增字段）

## 关闭

调用 `close()`：关闭 in-flight Semaphore，拒绝新请求并唤醒等待者为 `Cancelled`。已在执行的 HTTP
请求随其 future 生命周期结束。高层 multipart future 被 drop 后，RAII guard 会把 key/UploadId
写入有界进程内审计注册表；调用方读取 `multipart_orphan_audits()` 后调用 `abort_multipart()`，
成功时对应记录自动移除。进程崩溃仍需服务端 lifecycle 兜底，该能力保持 OPEN。

## 重试

默认最多 3 次、固定 100ms 退避；配置硬上界为 10 次。operation deadline 覆盖完整重试过程。
Initiate/Complete 因响应不确定性只发起一次，禁止自动重放制造 orphan 或错误完成判断。
`put_object_multipart` 的 initiate、全部 part、complete 共用同一剩余 deadline，不逐片重置。
