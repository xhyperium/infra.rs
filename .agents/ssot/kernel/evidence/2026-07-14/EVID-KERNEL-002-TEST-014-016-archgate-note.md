> **历史 monorepo 记录**（infra.rs）：文中 archgate / `.architecture` 不构成本仓验收条件；本仓不移植 archgate。

# EVID-KERNEL-002 — archgate KERNEL-* 旁注（Quality 会话）

| 字段 | 值 |
|------|-----|
| Date | 2026-07-14 |
| Scope | 确认 KERNEL-* 与 **KERNEL-API-002 缺席** |
| Live re-run | **SKIP**（Executor 无 Shell）；引用既有 JSON 产物 |

## 既有 `cargo run -p archgate -- --json` 产物

路径：`/tmp/ag.json` / `/tmp/ag2.json`（内容一致摘要）

- `kernel_rules`：**13** 条，全部 `"ok": true`
- `kernel_internal_count`: **8**（ERR-001 baseline）

| Rule ID | ok |
|---------|-----|
| KERNEL-DEP-001 | true |
| KERNEL-DEP-002 | true |
| KERNEL-FEATURE-001 | true |
| KERNEL-API-001 | true |
| KERNEL-TIME-001 | true |
| KERNEL-TIME-002 | true |
| KERNEL-TIME-003 | true |
| KERNEL-ERR-001 | true |
| KERNEL-ERR-002 | true |
| KERNEL-SERDE-001 | true |
| KERNEL-ASYNC-001 | true |
| KERNEL-UNSAFE-001 | true |
| KERNEL-LIFECYCLE-001 | true |

## KERNEL-API-002

- **Absent** from `kernel_rules` JSON keys.
- **Absent** from `tools/archgate/src/kernel_rules.rs`（仅 KERNEL-API-**001**）。
- **Absent** from `tools/archgate/docs/README.md` 规则表。
- Residual **RES-GATE-009** 已登记：§12.2 KERNEL-API-002 未机器强制 → **OPEN**。

## 诚实边界

- 本会话 **未**重新 `cargo run -p archgate`；以上为磁盘既有 JSON + 源码交叉核对。
- 不因本旁注关闭 RES-TEST-014/015/016 或 RES-GATE-009。
