# redisx docs

**Package**：`redisx` · **角色**：生产 Redis 客户端（P0 KV）+ 可选 scaffold

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
- Cluster / Sentinel / 完整 PubSub 合同：见 draft 后续里程碑
