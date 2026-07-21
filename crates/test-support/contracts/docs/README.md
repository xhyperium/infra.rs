# contract-testkit docs

**Package**：`contract-testkit` · **lib**：`contract_testkit` · **角色**：T0 test-support（仅 dev-dep）

本目录存放 **crate 级**设计 / 契约补充。不替代 rustdoc；不重复仓库根治理文档。

## 入口

| 资源 | 路径 |
|------|------|
| 人类入口 | [../README.md](../README.md) |
| Agent 规则 | [../AGENTS.md](../AGENTS.md) |
| 变更日志 | [../CHANGELOG.md](../CHANGELOG.md) |
| testkit SSOT 对齐 | [`docs/ssot/testkit-ssot-alignment.md`](../../../../docs/ssot/testkit-ssot-alignment.md) |
| contracts SSOT 对齐 | [`docs/ssot/contracts-ssot-alignment.md`](../../../../docs/ssot/contracts-ssot-alignment.md) |
| 权威规格 §3.2 | `.agents/ssot/testkit/spec/spec.md` |

## 边界

- **放这里**：本 crate Fake/suite 边界、消费方式、与 contracts 的依赖关系
- **不放这里**：trait 语义正文（见 `crates/contracts/docs/`）、全仓 SSOT 总览

## 公开 API 说明

详见 [API.md](./API.md)。
