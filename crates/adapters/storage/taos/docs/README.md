# taosx docs

**Package**：`taosx` · **lib**：`taosx` · **角色**：storage adapter（REST 生产默认）

本目录存放 **crate 级**设计 / 契约补充 / 迁移笔记。
不替代 rustdoc；不重复仓库根治理文档（见分层边界 `crates/AGENTS.md`）。

## 入口

| 资源 | 路径 |
|------|------|
| 人类入口 | [../README.md](../README.md) |
| Agent 规则 | [../AGENTS.md](../AGENTS.md) |
| 变更日志 | [../CHANGELOG.md](../CHANGELOG.md) |
| 本仓 SSOT 对齐 | [`docs/ssot/adapters-ssot-alignment.md`](../../../../../docs/ssot/adapters-ssot-alignment.md) |
| 上游 SSOT 镜像 | `.agents/ssot/adapters/storage/taos/` |
| Workspace 总览 | [`docs/ssot/workspace-ssot-alignment.md`](../../../../../docs/ssot/workspace-ssot-alignment.md) |

## 公开 API（P0）

| 符号 | 说明 |
|------|------|
| `TaosConfig` / `TsPrecision` | env / 默认值；精度探测与 ns↔库换算 |
| `TaosPool` | `connect` / `client` / `ping` / `exec_sql` / `ensure_stable` / `close` |
| `TaosClient` | `TaosPool` 类型别名 |
| `TaosExecResult` | REST 解析结果 |
| `TimeSeriesStore` impl | `write_series` / `query_series` |
| `TaosAdapter` | 仅 `feature = "scaffold"` |

## 状态声明

- 默认路径为 **REST 生产客户端**（`:6041/rest/sql`）。
- 未宣称：native WS 全量、TMQ、schemaless 完整面、exactly-once。
