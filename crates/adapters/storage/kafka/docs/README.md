# kafkax docs

**Package**：`kafkax` · **lib**：`kafkax` · **角色**：storage adapter（生产默认 `rdkafka`）

本目录存放 **crate 级**设计 / 契约补充 / 迁移笔记。
不替代 rustdoc；不重复仓库根治理文档。

## 入口

| 资源 | 路径 |
|------|------|
| 人类入口 | [../README.md](../README.md) |
| Agent 规则 | [../AGENTS.md](../AGENTS.md) |
| 变更日志 | [../CHANGELOG.md](../CHANGELOG.md) |
| 本仓 SSOT 对齐 | [`docs/ssot/adapters-ssot-alignment.md`](../../../../../docs/ssot/adapters-ssot-alignment.md) |

## 生产面（P0）

- `KafkaPool` + `KafkaProducer`（delivery report）+ `KafkaConsumer`
- SASL/TLS 配置：`FOUNDATIONX_KAFKAX_*`
- `KafkaEventBus`：`EventBus` at-most-once；`BusMessage.id` = `topic/partition/offset`
- live：`tests/live_event_bus.rs`（`#[ignore]`）
- bench：`benches/hot_path.rs`

## 已知环境注意

- live 测试在 group coordinator 不可用时改用 `assign` 验证 produce→fetch
- 生产环境应修复 coordinator，再依赖 `subscribe` / `EventBus::subscribe`

## 延后

- EOS / transactional producer
- schema registry
- 显式 ack 消费扩展（可靠面）

## scaffold

`cargo test -p kafkax --features scaffold` 导出旧内存 `KafkaAdapter` / `MockKafkaBus`。
