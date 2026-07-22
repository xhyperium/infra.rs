# natsx

- 生产默认：`async-nats` → `NatsPool` + `EventBus`（Core NATS at-most-once）
- 依赖 `xhyper-contracts`（path+version）
- 旧内存实现在 feature `scaffold`
- JetStream durable pull/显式确认已落地；Cluster/HA/自动 DLQ 不在稳定承诺内
- Core EventBus 固定 at-most-once；禁止与 JetStream 持久语义混称
- 同客户端 broker 重启恢复连续三次真实实验失败；在 `infra-2d9.3.1` 关闭前保持 NO-GO
- operation deadline、有限 reconnect/capacity 与 stats 不等于恢复保证
- 凭据只来自环境 / 本地默认；禁止写入仓库密钥文件
