# natsx docs

**Package**：`natsx` · **lib**：`natsx` · **角色**：storage adapter（生产默认 `async-nats`）

本目录存放 **crate 级**设计 / 契约补充 / 迁移笔记。

## 入口

| 资源 | 路径 |
|------|------|
| 人类入口 | [../README.md](../README.md) |
| Agent 规则 | [../AGENTS.md](../AGENTS.md) |
| 变更日志 | [../CHANGELOG.md](../CHANGELOG.md) |

## 生产面（P0）

- `NatsPool`：connect / publish / subscribe / ping / health / close
- 认证：`FOUNDATIONX_NATS_{URL,USER,PASSWORD}`（兼容 `NATSX_*`）
- `NatsEventBus` / `EventBus for NatsPool`：Core NATS at-most-once
- live：`tests/live_event_bus.rs`（`#[ignore]`）
- bench：`benches/hot_path.rs`

## 延后

- JetStream durable / explicit ack
- NKey / JWT / mTLS 全矩阵
- queue group / request-reply 稳定扩展

## scaffold

`cargo test -p natsx --features scaffold` 导出旧内存实现。
