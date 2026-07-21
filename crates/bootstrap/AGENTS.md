# AGENTS.md — bootstrap

> 完整行为准则与架构约束以仓库根 [AGENTS.md](../../AGENTS.md) 与 [CONSTITUTION.md](../../CONSTITUTION.md) 为准。

## 本 crate 约束

- **R3.1 豁免边界**：本 crate 是 workspace 唯一允许「知道全局」的组合根；其它 L1 不得复制此模式作为业务依赖。
- **ADR-016**：typed composition（`PlatformContext` / `AppContext` / `BootstrappedApp`）；**禁止** runtime gate / 字符串 Service Locator。
- 生产依赖仅 `xhyper-kernel`；`contracts` / `observex` / `evidence` 全量 crate **DEFER**，用 `traits` 模块可移植对象安全替面。
- 禁止新增 `register` / `resolve` / 公开 `Gate` 类型。
- 对应 SSOT：`.agents/ssot/infra/bootstrap/spec/spec.md`。

## 目录

```text
crates/bootstrap/
├── Cargo.toml
├── src/{lib,error,bounded,traits}.rs
├── examples/minimal.rs
├── docs/
├── tests/public_api.rs
├── CHANGELOG.md
├── AGENTS.md
└── README.md
```

## 验证

```bash
cargo test -p xhyper-bootstrap --all-targets
cargo clippy -p xhyper-bootstrap --all-targets -- -D warnings
```
