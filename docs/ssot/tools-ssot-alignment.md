# tools SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| 域 | `tools/`（evidence · goalctl · xtask · verifyctl） |
| SSOT | `.agents/ssot/tools/**` |
| 本仓实现 | 见下表；**禁止**把文档 COMPLETE 当作 ship |
| 审计日期 | 2026-07-21 |
| 结论 | **tools SSOT 已本仓化**；仅 `evidence` 最小面落地；goalctl / xtask / verifyctl **未**宣称 crate 落地 |

## 结论摘要

| 问题 | 状态 |
|------|------|
| tools SSOT 树 | **已就位**：`evidence` / `goalctl` / `xtask` / `verifyctl` |
| 路径约定 | 统一 `.agents/ssot/tools/**`（保留 `tools/` 层级） |
| `verifyctl` | 本仓扩展域；11 层布局 + 生产 Goal/Spec 已入树 |
| `goalctl` 生产增补 | `goal/goalctl-production-goal.md` · `spec/goalctl-production-spec.md` |
| 本仓 crate 落地 | **仅** `crates/evidence`（`xhyper-evidence`）最小面；见 [evidence-ssot-alignment.md](./evidence-ssot-alignment.md) |
| `tools/goalctl` · `tools/xtask` · `tools/verifyctl` | **未** workspace member · **未**宣称 ship |

## 目录

```text
.agents/ssot/tools/
├── README.md              # 本仓索引
├── evidence/              # → crates/evidence · xhyper-evidence
├── goalctl/               # → tools/goalctl（期望；未落地）
├── xtask/                 # → tools/xtask / infra-xtask（期望；未落地）
└── verifyctl/             # → tools/verifyctl（期望；未落地）
```

每个子域对齐 kernel 11 层布局（`goal/spec/design/plan/tasks/prompt/test/review/release/retrospective` + `matrix/gate/evidence` + `README.md`）。`goalctl` 另含 schemas / contracts / decisions 等规划制品。

## 本仓可观察事实

| 子域 | SSOT 路径 | 本仓路径 | package | 本仓状态 |
|------|----------|----------|---------|----------|
| evidence | `.agents/ssot/tools/evidence` | `crates/evidence` | `xhyper-evidence` | 最小面已落地（Appender + bootstrap 注入）；远程/签名 DEFER |
| goalctl | `.agents/ssot/tools/goalctl` | `tools/goalctl` | — | **未** member |
| xtask | `.agents/ssot/tools/xtask` | `tools/xtask` | — | **未** member |
| verifyctl | `.agents/ssot/tools/verifyctl` | `tools/verifyctl` | — | **未** member |

## 验证

```bash
DST=.agents/ssot/tools
test -d "$DST/evidence" && test -d "$DST/goalctl" && test -d "$DST/xtask" && test -d "$DST/verifyctl"

# 不得出现外仓仓库名字面量（needle = xhyper + '.' + rs）
needle=$'xhyper\x2ers'
! rg -q -F "$needle" "$DST" || { echo "FAIL: residual foreign repo name"; exit 1; }

# verifyctl 双镜像
cmp "$DST/verifyctl/spec/spec.md" "$DST/verifyctl/spec/xhyper-verifyctl-complete-spec.md"

# 本仓 evidence 实现
cargo test -p xhyper-evidence -p xhyper-bootstrap --all-targets
```

## 相关

| 文档 | 说明 |
|------|------|
| [workspace-ssot-alignment.md](./workspace-ssot-alignment.md) | 总览 |
| [evidence-ssot-alignment.md](./evidence-ssot-alignment.md) | evidence crate 落地矩阵 |
| [SSOT_SYNC_OPS.md](./SSOT_SYNC_OPS.md) | 同步操作（其他域） |
| [.agents/ssot/SSOT.md](../../.agents/ssot/SSOT.md) | R6/R7 |
| [.agents/ssot/tools/README.md](../../.agents/ssot/tools/README.md) | tools 索引 |
