# kafkax

- 生产默认：`rdkafka` → `KafkaPool` + `EventBus` facade（at-most-once）
- `BusMessage.id` = `topic/partition/offset`
- 依赖 `xhyper-contracts`（path+version）
- 旧内存实现在 feature `scaffold`
- 凭据只来自环境 / 本地默认；禁止写入仓库密钥文件
