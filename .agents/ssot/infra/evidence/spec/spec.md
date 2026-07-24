# `evidence` 当前实现规范

| 字段 | 值 |
|---|---|
| Status | 当前 `0.1.1`：L1 审计证据**追加面**（append-only）；**非** 合规审计平台 / 远程 CA / 不可抵赖审计产品 |
| Package / lib | `evidence` / `evidence`（别名 `xhyper-evidence` 仅作废弃兼容标签 / dual-mirror 文件名） |
| Path | `crates/infra/evidence` |
| Layer | L1 Infra |
| Authority | 本文件是 active current-state spec |
| Implementation snapshot | `1b80898e0425cf7dc0f787c0f663154c24c8bb37`（本轮审计起点 `origin/main`） |
| Document state | 当前分支候选；合并后以 PR merge SHA 追溯文档版本 |
| Verified at | 2026-07-22；`cargo test -p evidence --all-targets` 与 current-state 门禁 |

## 1. 定位与边界

`evidence` 是 L1 **审计证据追加面**（append-only）：提供证据的追加、查询、HMAC 签名与可注入的远程传输，构成审计证据链的最小权威面。由 `bootstrap` 注入（可选 / 必需模式）。

**登记 ≠ 合规审计产品**：本 crate 仅提供追加与签名原语，不宣称远程 CA、不可抵赖审计平台或完整合规审计产品全量。

非目标：通用合规审计平台、远程 CA、不可抵赖审计产品全量、持久化恢复产品矩阵。

## 2. 当前依赖

| 依赖 | 当前用途 |
|---|---|
| `sha2`（workspace） | HMAC-SHA256 签名 wire 自实现（无 hmac crate） |

生产 dep 仅 `sha2`；无 async runtime 强制依赖。

## 3. 当前公开 API

| 类型 | 当前职责 |
|---|---|
| `EvidenceAppender` | 追加证据 trait（append-only 面） |
| `EvidenceError` | 追加 / 查询 / 签名错误 |
| `AppendReceipt` | 追加回执 |
| `InMemoryEvidenceAppender` | 内存追加实现 |
| `FileEvidenceAppender` | 最小文件持久化追加（infra-s9t.7；#168） |
| `EvidenceQuery` | 查询 API |
| `sign_evidence` / `verify_evidence` | HMAC-SHA256 签名 wire |
| `EvidenceTransport` / `RemoteEvidenceAppender` / `MockEvidenceTransport` | 远程传输面（接口 + Mock；非生产 CA） |

## 4. 当前成熟度与开放项

- `[KNOWN] LCOV 100%` 行覆盖达成（cov-gate）。
- 合规产品 / 远程 CA / 不可抵赖审计平台：**OPEN（诚实边界）**，非本轮宣称。
- Agent L5 人签模板：**未填**，禁止 Agent 代签。

反例条件：源码出现真实远程 CA 签名或宣称不可抵赖审计平台时，“最小追加面”结论失效。

## 5. 验收

```bash
cargo test -p evidence
cargo check -p evidence --all-targets
cargo clippy -p evidence --all-targets -- -D warnings
cargo fmt -- --check
node scripts/quality-gates/cov-gate-100.mjs -p evidence
```

通过条件：API / 依赖与源码一致；不把追加面冒充合规审计平台。

## 6. 追溯

- `docs/ssot/evidence-ssot-alignment.md`
- `crates/infra/evidence/{Cargo.toml,src/}`
