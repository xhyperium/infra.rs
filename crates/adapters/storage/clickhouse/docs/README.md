# clickhousex docs

**Package**：`clickhousex` · **lib**：`clickhousex` · **角色**：storage adapter（HTTP 生产默认）

本目录存放 **crate 级**设计 / 契约补充 / 迁移笔记。
不替代 rustdoc；不重复仓库根治理文档（见分层边界 `crates/AGENTS.md`）。

## 入口

| 资源 | 路径 |
|------|------|
| 人类入口 | [../README.md](../README.md) |
| Agent 规则 | [../AGENTS.md](../AGENTS.md) |
| 变更日志 | [../CHANGELOG.md](../CHANGELOG.md) |
| 本仓 SSOT 对齐 | [`docs/ssot/adapters-ssot-alignment.md`](../../../../../docs/ssot/adapters-ssot-alignment.md) |
| 上游 SSOT 镜像 | `.agents/ssot/adapters/storage/clickhouse/` |
| Workspace 总览 | [`docs/ssot/workspace-ssot-alignment.md`](../../../../../docs/ssot/workspace-ssot-alignment.md) |

## 公开 API（P0）

| 符号 | 说明 |
|------|------|
| `ClickHouseConfig` | env / 默认值；`password` Debug 脱敏 |
| `ClickHousePool` | `connect` / `client` / `ping` / `execute` / `query_text` / `query_rows` / `insert_json_each_row` / `ensure_analytics_table` / `close` |
| `ClickHouseClient` | `ClickHousePool` 类型别名 |
| `AnalyticsSink` impl | `sink(event, payload)` → `analytics_events` 表 |
| `ClickHouseAdapter` | 仅 `feature = "scaffold"` |

## 状态声明

- 默认路径为 **HTTP 生产客户端**（非 scaffold）。
- 未宣称：官方 `clickhouse` crate 原生协议、集群路由、强类型 RowBinary batcher、exactly-once。
