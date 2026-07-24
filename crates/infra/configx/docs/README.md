# configx docs

**Package / lib**：`configx` / `configx` · **版本**：`0.1.2` · **角色**：L1 进程内字符串配置组件

本目录存放 **crate 级**设计 / 契约补充 / 迁移笔记。
不替代 rustdoc；不重复仓库根治理文档（见分层边界 `crates/AGENTS.md`）。

## 入口

| 资源 | 路径 |
|------|------|
| 人类入口 | [../README.md](../README.md) |
| Agent 规则 | [../AGENTS.md](../AGENTS.md) |
| 变更日志 | [../CHANGELOG.md](../CHANGELOG.md) |
| 本仓 SSOT 对齐 | [`docs/ssot/configx-ssot-alignment.md`](../../../docs/ssot/configx-ssot-alignment.md) |
| 上游 SSOT 镜像 | `.agents/ssot/infra/configx/` |
| Workspace 总览 | [`docs/ssot/workspace-ssot-alignment.md`](../../../docs/ssot/workspace-ssot-alignment.md) |

## 边界

- **放这里**：本 crate 设计决策、公开 API 契约补充、迁移 / 升级笔记
- **不放这里**：全仓治理、跨 crate SSOT 总览、CI 状态（见仓库根 `docs/{governance,ssot,status,decisions}/`）

## 公开 API 说明

详见 [API.md](./API.md)（公开消费面 + 最小用法）。

文档中的 reload 均指调用方显式触发的进程内操作；不表示自动文件 watcher 或远端配置中心。
