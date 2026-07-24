# evidence 本仓落地状态

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-21；**defer-close 复核 2026-07-22** |
| crate | `crates/infra/evidence` · `evidence` / lib `evidence` · **v0.1.1**（文档名 `xhyper-evidence` 为废弃别名） |
| 消费者 | `bootstrap`（注入） |
| 定位 | L1 审计证据**追加面**（append-only）；**非** 合规审计平台 / 远程 CA / 不可抵赖审计产品 |
| Canonical SSOT | `.agents/ssot/infra/evidence/spec/spec.md` ≡ `xhyper-evidence-complete-spec.md` |
| 历史入口 | `.agents/ssot/tools/evidence/README.md` 仅重定向；不得维护第二份 active spec |

## 结论

| 项 | 状态 | 证据 |
|----|------|------|
| `EvidenceAppender` / `EvidenceError` / `AppendReceipt` | **PASS** | `src/lib.rs` |
| `InMemoryEvidenceAppender` | **PASS** | 内存追加 |
| bootstrap re-export + `with_evidence` | **PASS** | bootstrap 对齐文 |
| `FileEvidenceAppender` | **PASS** | infra-s9t.7 最小文件持久化；#168 |
| 查询 API | **PASS** | `src/query.rs` · `EvidenceQuery` |
| 签名 wire | **PASS** | `src/sign.rs` · HMAC-SHA256 `sign_evidence` / `verify_evidence` |
| 远程传输 | **PASS（接口+Mock）** | `src/remote.rs` · `EvidenceTransport` / `RemoteEvidenceAppender` / `MockEvidenceTransport` |
| 合规产品 / 远程 CA / 不可抵赖审计平台 | **OPEN（诚实边界）** | 非本轮宣称 |
| LCOV 100% | **PASS** | cov-gate |
| Agent L5 | **未填** | 禁止 Agent 代签 |

## OBJECTIVE 处置（2026-07-22 defer-close）

| 项 | 前状态 | 现状态 | 证据 |
|----|--------|--------|------|
| 远程 wire | DEFER | **PASS（transport trait + mock）** | `crates/infra/evidence/src/remote.rs` |
| 签名 | DEFER | **PASS（HMAC 面）** | `crates/infra/evidence/src/sign.rs` |
| 查询 API | DEFER | **PASS** | `crates/infra/evidence/src/query.rs` |

## 验证

```bash
cargo test -p evidence -p bootstrap --all-targets
node scripts/quality-gates/cov-gate-100.mjs -p evidence --filter crates/infra/evidence/src
node scripts/quality-gates/check-ssot-current-state.mjs
```

## 双栏落地（2026-07-22 · STATUS 100% structure）

| 标尺 | 状态 |
|------|------|
| STATUS 结构完成度 | **100%**（layout+tests+content；非 Production Ready） |
| 声明面生产硬化 | 公共 API 集成测 + 热路径 bench + `docs/` 红线；**cov-gate-100 行覆盖** |
| 非宣称 | **禁止** workspace Production Ready / Agent L5 / 合规审计产品全量 |

自验证：`cargo test -p evidence --all-targets`；`node scripts/quality-gates/cov-gate-100.mjs -p evidence`。

## 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-22 | **defer-close**：query/sign/remote PASS；合规产品仍 OPEN |
| 2026-07-22 | 对齐 Cargo 真相：版本 `0.1.1`；明确 L1 审计证据追加面（append-only）；`xhyper-evidence` 仅废弃别名；SSOT 镜像指向 `.agents/ssot/infra/evidence/spec/` |
| 2026-07-22 | 冻结 `.agents/ssot/infra/evidence/spec/spec.md` 为唯一 active current-state 入口；`tools/evidence` 仅保留历史重定向 |
