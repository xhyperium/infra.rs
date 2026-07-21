# postgresx docs

**Package**：`postgresx` · **lib**：`postgresx` · **角色**：storage adapter scaffold

本目录存放 **crate 级**设计 / 契约补充 / 迁移笔记。
不替代 rustdoc；不重复仓库根治理文档（见分层边界 `crates/AGENTS.md`）。

## 入口

| 资源 | 路径 |
|------|------|
| 人类入口 | [../README.md](../README.md) |
| Agent 规则 | [../AGENTS.md](../AGENTS.md) |
| 变更日志 | [../CHANGELOG.md](../CHANGELOG.md) |
| 本仓 SSOT 对齐 | [`docs/ssot/adapters-ssot-alignment.md`](../../../../../docs/ssot/adapters-ssot-alignment.md) |
| 上游 SSOT 镜像 | `.agents/ssot/adapters/storage/postgres/` |
| Workspace 总览 | [`docs/ssot/workspace-ssot-alignment.md`](../../../../../docs/ssot/workspace-ssot-alignment.md) |

## 边界

- **放这里**：本 crate 设计决策、公开 API 契约补充、迁移 / 升级笔记
- **不放这里**：全仓治理、跨 crate SSOT 总览、CI 状态（见仓库根 `docs/{governance,ssot,status,decisions}/`）

## 状态声明

本 crate 当前为 **scaffold**：标准布局与 trait 接线可能已齐，**不等于**业务实现或 package stable。
以对齐矩阵与 `cargo metadata` 为准。

## 生产误用警示（infra-s9t.14）

**默认实现是进程内 scaffold/mock，不是生产客户端。**

- 禁止把 `*Adapter` 类型名当成已对接真实 Binance/Postgres/Redis/…
- 真实入口须有显式 feature（如 redisx `live`）与文档/CI 证据
- 详见 `docs/plans/artifacts/prod-consume-surface.md`
