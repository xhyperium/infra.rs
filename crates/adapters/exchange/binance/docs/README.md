# binancex docs

**Package**：`binancex` · **lib**：`binancex` · **角色**：exchange adapter 生产默认 REST+WS

本目录存放 **crate 级**设计 / 契约补充 / 迁移笔记。
不替代 rustdoc；不重复仓库根治理文档（见分层边界 `crates/AGENTS.md`）。

## 入口

| 资源 | 路径 |
|------|------|
| 人类入口 | [../README.md](../README.md) |
| Agent 规则 | [../AGENTS.md](../AGENTS.md) |
| 变更日志 | [../CHANGELOG.md](../CHANGELOG.md) |
| 本仓 SSOT 对齐 | [`docs/ssot/adapters-ssot-alignment.md`](../../../../../docs/ssot/adapters-ssot-alignment.md) |
| 上游 SSOT 镜像 | `.agents/ssot/market_data/binance/` |
| Workspace 总览 | [`docs/ssot/workspace-ssot-alignment.md`](../../../../../docs/ssot/workspace-ssot-alignment.md) |

## 边界

- **放这里**：本 crate 设计决策、公开 API 契约补充、迁移 / 升级笔记
- **不放这里**：全仓治理、跨 crate SSOT 总览、CI 状态（见仓库根 `docs/{governance,ssot,status,decisions}/`）

## 状态声明

本 crate 为 **生产默认 REST+WS 协议路径**（#210+#214）：注入 `HttpDriver`+凭证走签名 REST；注入 `WsConnector` 解析公共行情。
**不等于** package stable / L5 / crates.io。以对齐矩阵与 `cargo metadata` 为准。

## 生产误用警示（infra-s9t.14）

**默认（无注入）是进程内 mock / 空流；注入后为协议路径。**

- 禁止无注入时把返回的 `Open` 当成真实成交
- live 仅 `#[ignore]` 公共 server_time；签名交易不进默认 CI
- 详见 crate 根 README 行为分层表与 `docs/ssot/adapters-ssot-alignment.md`
