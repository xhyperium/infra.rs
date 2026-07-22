# kafkax

- 生产默认：纯 Rust `rskafka` → `KafkaPool` + `EventBus` facade（at-most-once）
- 应用级 ALO 仅限手动分区、单 owner、单调 checkpoint；无 group/rebalance/fencing
- `ProduceThenCheckpoint*` 非原子且存在重复窗口；禁止称 EOS
- TLS + PEM CA 已接入；远程明文必须 fail-closed；SASL 仅批准 PLAIN
- `BusMessage.id` = `topic/partition/offset`
- 依赖 `xhyper-contracts`（path+version）
- 旧内存实现在 feature `scaffold`
- 凭据只来自环境 / 本地默认；禁止写入仓库密钥文件
- group/rebalance/fencing/native EOS 仍 NO-GO
