# 支持矩阵声明（W0 冻结 · DEFER-6）

| 字段 | 值 |
|------|-----|
| Plan | [PLAN-CORE-PROD-002](../2026-07-21-core-crates-production-readiness.md) |
| Beads | `infra-asa.1`（W0）· 门禁 `infra-asa.6`（W5） |
| 冻结日期 | 2026-07-21 |
| 状态 | **Accepted: Linux-only CI**（明确关闭「未知跨平台」歧义） |

## 1. 官方支持（本轮签字面）

| 维度 | 支持值 | 证据 |
|------|--------|------|
| OS | **Linux**（CI：`ubuntu-latest`） | `.github/workflows/ci-rust.yml` 等 |
| Arch | **x86_64**（CI runner 默认） | GitHub-hosted runners |
| MSRV | **1.85** | 根 `Cargo.toml` `rust-version = "1.85"`；`ci-rust.yml` job `msrv` |
| Edition | 2021（workspace） | 根 `Cargo.toml` |

## 2. 明确不宣称

| 维度 | 声明 |
|------|------|
| macOS | **未**纳入本轮 Production 签字；不保证 CI 绿 |
| Windows | **未**纳入 |
| 非 x86_64（含 aarch64） | **未**纳入官方矩阵；欢迎社区反馈但不阻塞 L4 |
| 浏览器 / WASM | 非目标 |

若未来扩展矩阵：新增 CI job + 修订本表 + W5 再签字。

## 3. 与 crate 文档的关系

- 各核心 crate README 若未写平台，以**本表**为准。
- W5 可将本表摘要链入 `.agents/rules/` 或根 README（可选，非 W0 阻塞）。
- `publish = false` 不变；支持矩阵描述的是 **内部生产消费者** 环境，不是 crates.io。

## 4. 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-21 | W0：Accept 仅 Linux x86_64 + MSRV 1.85（关闭 DEFER-6 未知态） |
