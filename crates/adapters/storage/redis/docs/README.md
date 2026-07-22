# redisx docs

**Package**：`redisx` · **版本**：`0.3.4` 未发布候选 ·
**角色**：生产 Redis 客户端（P0 KV）+ 可选 scaffold

| 文档 | 路径 |
|------|------|
| 使用 | [usage.md](./usage.md) |
| 配置 | [config.md](./config.md) |
| 运维 | [operations.md](./operations.md) |
| 人类入口 | [../README.md](../README.md) |
| Agent 规则 | [../AGENTS.md](../AGENTS.md) |
| 变更日志 | [../CHANGELOG.md](../CHANGELOG.md) |

## 状态

- **默认路径**：真实 `redis` 异步客户端（`RedisPool` / `RedisClient`）
- **scaffold feature**：进程内 HashMap，仅测试/迁移
- 命令通道已有 Cluster / Sentinel / TLS 代码路径，但三者真实 live 证据仍 OPEN
- Pub/Sub 仅 Standalone；复用池配置，Cluster / Sentinel 失败关闭
- 参数化 retry safety 与原子性边界见 [operations.md](./operations.md)
