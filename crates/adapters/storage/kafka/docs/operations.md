# kafkax 运维

## 健康检查

- **liveness**：进程存活即可
- **readiness**：调用池/客户端 `health`/`ping`（有 deadline）

## 故障

| 症状 | 处理 |
|------|------|
| connect 失败 | 检查 FOUNDATIONX_KAFKAX_* 与网络/认证 |
| DeadlineExceeded | 调高 timeout；查下游慢查询/背压 |
| Unavailable | 下游重启/鉴权；观察重连日志 |
| TLS 配置 Invalid | 当前构建不支持 TLS；停止上线，不能降级明文 |

## 升级 / 回滚

1. 发布前跑 `cargo test -p kafkax` 与 `node scripts/broker-conformance.mjs`
2. 升级：先滚动 canary 实例，观察错误率与延迟
3. 回滚：回退至上一 crate 版本；配置 schema 保持向后兼容（仅新增字段）

## 关闭

调用 `close(deadline)`：拒绝新请求并排空 in-flight。

## 语义限制

仅支持手动分区、单 owner 的应用 checkpoint；group/rebalance/fencing/native EOS 未实现。
produce 成功而 checkpoint 失败时会重复，业务必须按稳定幂等键去重。
