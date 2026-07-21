# ossx docs

**Package**：`ossx` · **lib**：`ossx` · **角色**：storage adapter scaffold

本目录存放 **crate 级**设计 / 契约补充 / 迁移笔记。
不替代 rustdoc；不重复仓库根治理文档（见分层边界 `crates/AGENTS.md`）。

## 入口

| 资源 | 路径 |
|------|------|
| 人类入口 | [../README.md](../README.md) |
| Agent 规则 | [../AGENTS.md](../AGENTS.md) |
| 变更日志 | [../CHANGELOG.md](../CHANGELOG.md) |
| 本仓 SSOT 对齐 | [`docs/ssot/adapters-ssot-alignment.md`](../../../../../docs/ssot/adapters-ssot-alignment.md) |
| 上游 SSOT 镜像 | `.agents/ssot/adapters/storage/oss/` |
| Workspace 总览 | [`docs/ssot/workspace-ssot-alignment.md`](../../../../../docs/ssot/workspace-ssot-alignment.md) |

## 边界

- **放这里**：本 crate 设计决策、公开 API 契约补充、迁移 / 升级笔记
- **不放这里**：全仓治理、跨 crate SSOT 总览、CI 状态（见仓库根 `docs/{governance,ssot,status,decisions}/`）

## 状态声明

本 crate **默认生产入口**为 `OssClient`（reqwest + OSS V1 签名，`FOUNDATIONX_OSSX_*`）。

- feature `scaffold`：进程内 `OssAdapter`（**非**生产）
- multipart / lifecycle / package stable：**未**宣称
- 以对齐矩阵与 `cargo test -p ossx` / live `#[ignore]` 证据为准

## 生产误用警示

- **不要**把 feature `scaffold` 的 `OssAdapter` 当成真实 OSS
- 密钥仅经环境变量 / `scripts/live/export-foundationx-env.sh` 注入；**禁止**提交 secret
- `Debug` 对 AccessKeySecret 脱敏
