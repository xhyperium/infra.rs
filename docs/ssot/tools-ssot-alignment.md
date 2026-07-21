# tools SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| 域 | `tools/`（evidence · goalctl · xtask · verifyctl） |
| SSOT | `.agents/ssot/tools/**` |
| 本仓实现 | 见下表；**禁止**把文档 COMPLETE 当作 ship |
| 审计日期 | 2026-07-22 |
| 结论 | **tools SSOT 已本仓化**；`evidence` 最小面 + **goalctl / verifyctl 最小生产 CLI 已 member**；xtask **未**宣称 ship |

## 结论摘要

| 问题 | 状态 |
|------|------|
| tools SSOT 树 | **已就位**：`evidence` / `goalctl` / `xtask` / `verifyctl` |
| 路径约定 | 统一 `.agents/ssot/tools/**`（保留 `tools/` 层级） |
| `verifyctl` | **workspace member** `tools/verifyctl` · package `verifyctl`：plan / execute / report（最小面） |
| `goalctl` | **workspace member** `tools/goalctl` · package `goalctl`：doctor / validate / compile（Goal→Contract + digest） |
| 本仓 crate 落地 | `crates/evidence`；`tools/goalctl`；`tools/verifyctl` |
| `tools/xtask` | **未** member · **未**宣称 ship |

## 目录

```text
.agents/ssot/tools/
├── README.md
├── evidence/              # → crates/evidence
├── goalctl/               # → tools/goalctl（最小编译器已落地）
├── xtask/                 # → tools/xtask（期望；未落地）
└── verifyctl/             # → tools/verifyctl（最小 plan/execute/report 已落地）
```

## 本仓可观察事实

| 子域 | SSOT 路径 | 本仓路径 | package | 本仓状态 |
|------|----------|----------|---------|----------|
| evidence | `.agents/ssot/tools/evidence` | `crates/evidence` | `evidence` | 最小面 + `FileEvidenceAppender`；远程/签名 DEFER |
| goalctl | `.agents/ssot/tools/goalctl` | `tools/goalctl` | `goalctl` | **最小 Goal→Contract**：doctor/validate/compile + fixtures；**非**完整 authority plane |
| xtask | `.agents/ssot/tools/xtask` | `tools/xtask` | — | **未** member |
| verifyctl | `.agents/ssot/tools/verifyctl` | `tools/verifyctl` | `verifyctl` | **最小** plan/execute/report；`VERIFYCTL_DRY`；可选 `with-evidence` |

## 验证

```bash
DST=.agents/ssot/tools
test -d "$DST/evidence" && test -d "$DST/goalctl" && test -d "$DST/xtask" && test -d "$DST/verifyctl"

cargo test -p goalctl -p verifyctl -p evidence

# 不得出现外仓仓库名字面量（needle = xhyper + '.' + rs）
needle=$'xhyper\x2ers'
! rg -q -F "$needle" "$DST" || { echo "FAIL: residual foreign repo name"; exit 1; }
```

## 明确未宣称

- goalctl 完整 monorepo index/reconcile/authority plane（旧 Phase-1 大实现已收敛为最小编译器）
- verifyctl 变更感知闭包 / 远程 runner / 签名证据链
- xtask / gate 工作流编排

## 相关

| 文档 | 说明 |
|------|------|
| [workspace-ssot-alignment.md](./workspace-ssot-alignment.md) | 总览 |
| [evidence-ssot-alignment.md](./evidence-ssot-alignment.md) | evidence crate 落地矩阵 |
| [SSOT_SYNC_OPS.md](./SSOT_SYNC_OPS.md) | 同步操作 |
| [.agents/ssot/SSOT.md](../../.agents/ssot/SSOT.md) | R6/R7 |
