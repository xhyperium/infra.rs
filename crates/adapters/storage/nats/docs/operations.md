# natsx 运维

## 健康检查

- **liveness**：进程存活即可
- **readiness**：调用池/客户端 `health`/`ping`（有 deadline）

## 故障

| 症状 | 处理 |
|------|------|
| connect 失败 | 检查 FOUNDATIONX_NATS_* 与网络/认证 |
| DeadlineExceeded | 调高 timeout；查下游慢查询/背压 |
| Unavailable | 下游重启/鉴权；观察重连日志 |
| broker 短时重启 | 固定入口与重连预算内自动恢复；观察 connected/disconnected 增量 |
| broker 长时不可用后 channel closed | 已耗尽 `max_reconnects`；重建应用级 client，并检查入口可达性与重连预算 |
| slow consumer 增长 | 检查 subscription capacity、消费延迟与丢失语义 |

## 升级 / 回滚

1. 发布前跑 `cargo test -p natsx` 与 `node scripts/broker-conformance.mjs`
2. 升级：先滚动 canary 实例，观察错误率与延迟
3. 回滚：回退至上一 crate 版本；配置 schema 保持向后兼容（仅新增字段）

## 关闭

调用 `close()`：在内部 deadline 内 flush，随后标记 closed；不承诺排空应用仍持有的全部业务消息。

`node scripts/nats-reconnect-conformance.mjs` 保持容器与动态 host ingress 不变，只重启容器内
broker 进程，连续验证同一 client 的连接与原 Core subscription 恢复。该结果不证明断线窗口
无消息丢失，也不证明 Cluster/HA、JetStream 持久投递或超过有限重连预算后的自动自愈。
