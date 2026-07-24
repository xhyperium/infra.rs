# AGENTS.md — transportx

> 完整行为准则与架构约束以仓库根 [AGENTS.md](../../AGENTS.md) 与 [docs/constitution/](../../docs/constitution/) 为准。

## 分支纪律

严禁在 `main` 上开发；编辑前确认位于 worktree 或 feature branch。

## 本 crate 约束

- **R3 禁止依赖其他 L1 crate**：不得依赖 `configx` / `observex` / `resiliencx` / `schedulex` / `bootstrap`。
- **已有真实驱动**：`ReqwestHttpDriver`（`reqwest`）、`TungsteniteWsConnector`（`tokio-tungstenite`），以及 `MockHttpTransport`。
- 不实现重试/熔断/调度；不成为 bootstrap 组合根。
- 依赖：`kernel` + `async-trait` + `bytes` + `thiserror` + `reqwest` + `tokio` + `tokio-tungstenite` + `futures-util` + `httpdate`。
- 实现合同：`.agents/ssot/transport/spec/spec.md`。
- **未达 M3**：无真实 TLS/认证/连接池生产闭环证据；不得宣称为生产就绪。

## 目录

```text
crates/infra/transport/
├── Cargo.toml
├── src/lib.rs
├── tests/          # mock / HTTP driver / WS loopback
├── examples/
├── docs/
├── README.md
├── AGENTS.md
└── CHANGELOG.md
```

## 质量门禁

```bash
cargo test -p transportx --all-targets
cargo clippy -p transportx --all-targets --all-features -- -D warnings
node scripts/quality-gates/cov-gate-100.mjs -p transportx --filter crates/infra/transport/src
```
