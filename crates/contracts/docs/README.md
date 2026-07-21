# contracts docs

**Package**：`xhyper-contracts` · **lib**：`contracts` · **角色**：adapter trait / Instrumentation 出口

本目录存放 **crate 级**设计 / 契约补充 / 迁移笔记。
不替代 rustdoc；不重复仓库根治理文档（见分层边界 `crates/AGENTS.md`）。

## 入口

| 资源 | 路径 |
|------|------|
| 人类入口 | [../README.md](../README.md) |
| Agent 规则 | [../AGENTS.md](../AGENTS.md) |
| 变更日志 | [../CHANGELOG.md](../CHANGELOG.md) |
| **First-batch 语义合同** | [contracts/](./contracts/)（CT-8 部分） |
| 本仓 SSOT 对齐 | [`docs/ssot/contracts-ssot-alignment.md`](../../../docs/ssot/contracts-ssot-alignment.md) |
| 上游 SSOT 镜像 | `.agents/ssot/contracts/` |
| Workspace 总览 | [`docs/ssot/workspace-ssot-alignment.md`](../../../docs/ssot/workspace-ssot-alignment.md) |

## 边界

- **放这里**：本 crate 设计决策、公开 API 契约补充、迁移 / 升级笔记
- **不放这里**：全仓治理、跨 crate SSOT 总览、CI 状态（见仓库根 `docs/{governance,ssot,status,decisions}/`）

## 公开 API 说明

详见 [API.md](./API.md)（公开消费面 + 最小用法）。

