# adapters/storage/kafka — Retrospective

## 做得好

- 默认生产路径与 scaffold 分离，避免“假生产”
- live 默认 ignore，保护离线 CI
- draft 合同入库 SSOT，可审计

## 教训

- NATS 凭据以本机 conf 为准（md 可能过期）
- TDengine REST 端口 6041 而非 native 6030
- Kafka/NATS/CH/Taos bench 必须超时有界

## 后续

- EOS / transactional producer / schema registry / group coordinator 强依赖路径
