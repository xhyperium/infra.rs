# adapters/storage/redis — Retrospective

## 做得好

- 默认生产路径与 scaffold 分离，避免“假生产”
- live 默认 ignore，保护离线 CI
- draft 合同入库 SSOT，可审计

## 教训

- 重试安全不能用“读/写”二分：无 TTL SET/MSET 可按固定输入幂等处理，相对 TTL 写入与返回值敏感写入
  必须 fail-closed；粗粒度操作枚举无法替代 client 参数分类。
- NATS 凭据以本机 conf 为准（md 可能过期）
- TDengine REST 端口 6041 而非 native 6030
- Kafka/NATS/CH/Taos bench 必须超时有界

## 后续

- Cluster / Sentinel / Streams full / pubsub 默认关闭
