# kafkax 运维

## 健康检查

- **liveness**：进程存活即可
- **readiness**：调用池/客户端 `health`/`ping`（有 deadline）

## 故障

| 症状 | 处理 |
|------|------|
| connect 失败 | 检查 FOUNDATIONX_KAFKAX_* 与网络/认证 |
| DeadlineExceeded | 调高 timeout；查下游慢查询/背压 |
| Unavailable | 下游重启/鉴权；当前版本需由宿主重建客户端，不承诺自动重连 |
| TLS/CA 配置 Invalid | 检查 TLS 开关与 PEM CA；不能降级远程明文 |
| SASL Invalid/Unavailable | 仅支持 PLAIN；核对机制与轮换后的凭据 |

## 升级 / 回滚

1. 发布前跑 `cargo test -p kafkax`、broker conformance 与 TLS/SASL conformance
2. 升级：先滚动 canary 实例，观察错误率与延迟
3. 回滚：回退至上一 crate 版本；配置 schema 保持向后兼容（仅新增字段）

## 关闭

调用 `close(deadline)`：先标记 closed 并拒绝新请求，再广播取消到 producer、控制面 I/O、
consumer 与 EventBus 后台任务，最后等待在途操作守卫释放。若 deadline 到期，返回
`DeadlineExceeded`，pool 仍保持关闭；可再次调用以继续等待。

consumer 与 EventBus 使用固定容量有界队列。队列满时等待下游形成背压；关闭信号优先
打断该等待。关闭不提交 ALO pending，也不把未确认消息升级为已确认。

## 语义限制

仅支持手动分区、单 owner 的应用 checkpoint；group/rebalance/fencing、自动重连、native EOS 未实现。
produce 成功而 checkpoint 失败时会重复，业务必须按稳定幂等键去重。
