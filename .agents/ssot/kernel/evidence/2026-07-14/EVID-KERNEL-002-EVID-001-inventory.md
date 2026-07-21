> **历史 monorepo 记录**（infra.rs）：文中 archgate / `.architecture` 不构成本仓验收条件；本仓不移植 archgate。

# EVID-KERNEL-002-EVID-001 — §17 evidence tree inventory

| 字段 | 值 |
|------|-----|
| Residual | **RES-EVID-001** |
| Date | 2026-07-14 |
| Status | **CLOSED (partial-accepted)** |
| Spec | SPEC-KERNEL-002 §17 |
| Tree root | `.agents/ssot/kernel/evidence/2026-07-14/` |

## §17 要求的规范树

```text
evidence/kernel/<date>-<change-id>/
├── manifest.json
├── commands.log
├── fmt.log
├── clippy.log
├── test.log
├── coverage.json
├── mutants.json
├── archgate.json
├── public-api.diff
├── downstream-impact.md
└── verdict.md
```

本仓库实际布局为 **spec 附属 evidence 包**（非 `evidence/kernel/...` 字面路径），以  
`.agents/ssot/kernel/evidence/2026-07-14/` 为等价变更包根。下列对照 **语义等价物**，不伪装路径合规。

## 规范项 vs 现状

| §17 规范文件 | 状态 | 等价 / 现状说明 |
|--------------|------|-----------------|
| `manifest.json` | **PRESENT** | 有；字段为 campaign 扩展形（commit/toolchain 非严格最小 JSON） |
| `commands.log` | **MISSING** | 无单一 commands.log；分散于 `EVID-*-round-log.txt`、`cargo-kernel.txt`、`mono_check*.txt` |
| `fmt.log` | **MISSING** | 无独立日志；CI 绿见 `EVID-KERNEL-002-CI-*.md` |
| `clippy.log` | **MISSING** | 同上 |
| `test.log` | **PARTIAL** | `cargo-kernel.txt` / round-log 片段；非规范全量 tee |
| `coverage.json` | **MISSING** | 无 JSON 工件；line cover **文字**记于 `EVID-KERNEL-002-TEST-014-branch.md`（98.82% PASS）；branch **未测** |
| `mutants.json` | **MISSING** | 工具 ABSENT；见 `EVID-KERNEL-002-TEST-015-mutants.md`（OPEN DEFER） |
| `archgate.json` | **MISSING in-tree** | 包内无 JSON；旁注 `EVID-KERNEL-002-TEST-014-016-archgate-note.md` 引用 `/tmp/ag.json`（会话外） |
| `public-api.diff` | **MISSING** | 有 baseline `.architecture/api/kernel-public-api.txt`（RES-GATE-001）；本包无 diff 文件 |
| `downstream-impact.md` | **MISSING** | 下游结论散落 residual / R10 verdict；无独立 md |
| `verdict.md` | **PARTIAL** | `EVID-KERNEL-002-R10-verdict.md` + `R10b-verdict.md` 作 campaign verdict |

## 包内实际文件清单（2026-07-14 盘点）

```text
manifest.json
residual-open.txt
cargo-kernel.txt
mono_check.txt
mono_check_5x.txt
ssot-cmp.txt
EVID-KERNEL-002-CI-98af7c9c.md
EVID-KERNEL-002-CI-PR-fix.md
EVID-KERNEL-002-CLK009-TEST004.md
EVID-KERNEL-002-CODEX-REVIEW.md
EVID-KERNEL-002-G2-archgate-ci.md
EVID-KERNEL-002-G2-tests.md
EVID-KERNEL-002-R-code-gap-mid.md
EVID-KERNEL-002-R-test-gate-mid.md
EVID-KERNEL-002-R10-round-log.txt
EVID-KERNEL-002-R10-verdict.md
EVID-KERNEL-002-R10b-round-log.txt
EVID-KERNEL-002-R10b-verdict.md
EVID-KERNEL-002-TEST-014-016-archgate-note.md
EVID-KERNEL-002-TEST-014-branch.md
EVID-KERNEL-002-TEST-015-mutants.md
EVID-KERNEL-002-TEST-016-miri.md
EVID-KERNEL-002-DOWN-006.md          # 本会话新增
EVID-KERNEL-002-PERF-001.md          # 本会话新增
EVID-KERNEL-002-EVID-001-inventory.md # 本文件
```

## 明确缺失列表（partial 边界）

下列 **不得**假装已满足 §17 完整模板：

1. `commands.log` / `fmt.log` / `clippy.log`（规范命名的命令输出树）  
2. `coverage.json`（机器可读；branch 亦未测）  
3. `mutants.json`（依赖 RES-TEST-015 实跑）  
4. `archgate.json`（in-tree 拷贝）  
5. `public-api.diff`（相对 baseline 的本变更 diff）  
6. `downstream-impact.md`（独立文档）  
7. 路径字面 `evidence/kernel/<date>-<change-id>/`（本仓用 `.agents/ssot/kernel/evidence/<date>/`）

## partial-accepted 范围（本 residual 关闭语义）

**接受**：

- 2026-07-14 campaign 包存在且可追溯（manifest + 多份 EVID + residual ledger）。  
- 关键门禁结论以 **命令/CI evidence** 分散记录，而非手写 PASS 顶替（§17 禁止项仍遵守精神）。  
- 完整 §17 模板补齐 deferred 到：mutation/branch 工具可用 + 发布前归档流程（与 RES-TEST-014/015/016、§18 人审联动）。

**不接受 / 不声称**：

- §17 全树齐备  
- §18 Evidence 项可勾选完成  
- mutation/coverage SKIP = PASS（§17 明确禁止）

## 再开启 / 补齐条件

1. 按 §17 命名补齐缺失文件（可从 CI artifact 复制）；  
2. 或修订 spec 承认 `.agents/ssot/kernel/evidence/<date>/` + 本 inventory 为合法投影；  
3. `mutants.json` / branch coverage 仅在对应 residual 实跑后写入。

## Verdict

- **RES-EVID-001 → CLOSED (partial-accepted)**，缺失列表显式如上。  
- **不得**据此关闭 §18 或将 Spec Status 标 Approved。
