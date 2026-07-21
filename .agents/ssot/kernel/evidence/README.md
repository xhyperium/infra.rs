> **历史 monorepo 记录**（infra.rs）：文中 archgate / `.architecture` 不构成本仓验收条件；本仓不移植 archgate。

# Evidence — kernel

## 权威（本 Goal 执行波）

**[2026-07-14/](2026-07-14/)** — SPEC-KERNEL-002 对齐 + 代码主路径 + 十轮终裁

| 文件 | 用途 |
|------|------|
| `EVID-KERNEL-002-R10-verdict.md` | 章级 §0–§18 状态 |
| `EVID-KERNEL-002-R10-round-log.txt` | 十轮 checklist · fail_rounds=0 |
| `residual-open.txt` | residual ID OPEN/CLOSED（mid 原义） |
| `manifest.json` | 机器摘要 |
| `cargo-kernel.txt` / `mono_check*.txt` | 命令证据 |
| `EVID-KERNEL-002-TEST-014-branch.md` | RES-TEST-014 branch cov（OPEN/DEFER） |
| `EVID-KERNEL-002-TEST-015-mutants.md` | RES-TEST-015 mutants（OPEN/DEFER） |
| `EVID-KERNEL-002-TEST-016-miri.md` | RES-TEST-016 miri（OPEN/DEFER） |
| `EVID-KERNEL-002-TEST-014-016-archgate-note.md` | KERNEL-* / API-002 旁注 |
| `*-mid.md` | 迁移中途审计快照（**非**当前真相） |

Ship：**PR [#235](https://github.com/xhyperium/infra.rs/pull/235)**

## 历史

`2026-07-13/` — SPEC-001 批次；**不得**继承为 002 §18 闭合。
