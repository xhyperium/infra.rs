# natsx

- 生产默认：`async-nats` → `NatsPool` + `EventBus`（Core NATS at-most-once）
- 依赖 `xhyper-contracts`（path+version）
- 旧内存实现在 feature `scaffold`
- JetStream 不在本 P0 稳定承诺内
- 凭据只来自环境 / 本地默认；禁止写入仓库密钥文件
