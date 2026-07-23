# postgresx docs

**Package**：`postgresx` · **lib**：`postgresx` · **版本**：`0.3.6`（`publish = false`，未宣称 package stable） ·
**角色**：Postgres 生产连接池 + SQL API

## 入口

| 文档 | 路径 |
|------|------|
| 用法 | [usage.md](./usage.md) |
| 配置 | [config.md](./config.md) |
| 运维 | [operations.md](./operations.md) |
| 人类入口 | [../README.md](../README.md) |
| Agent 规则 | [../AGENTS.md](../AGENTS.md) |
| 变更日志 | [../CHANGELOG.md](../CHANGELOG.md) |
| 本仓 SSOT 对齐 | [`docs/ssot/adapters-ssot-alignment.md`](../../../../../docs/ssot/adapters-ssot-alignment.md) |

## 状态

- **默认**：`PostgresPool` 生产路径（`deadpool-postgres` + `tokio-postgres`）
- **可选**：feature `scaffold` → 进程内 mock/adapter
- Live 测试 `#[ignore]`；不作为 CI 默认绿证据
