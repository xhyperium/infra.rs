# tools SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| 域 | `tools/`（evidence · goalctl · xtask · verifyctl） |
| SSOT | `.agents/ssot/tools/**` |
| 本仓实现 | 见下表；**禁止**把文档 COMPLETE 当作 ship |
| 审计日期 | 2026-07-22 |
| 结论 | **tools SSOT 已本仓化**；`evidence` 最小面 + **goalctl / verifyctl 最小生产 CLI 已 member**（#188）；live env 构建器 #191；xtask **未**宣称 ship |

## 结论摘要

| 问题 | 状态 |
|------|------|
| tools SSOT 树 | **已就位**：`evidence` / `goalctl` / `xtask` / `verifyctl` |
| 路径约定 | 统一 `.agents/ssot/tools/**`（保留 `tools/` 层级） |
| `verifyctl` | **workspace member** `tools/verifyctl` · package `verifyctl`：plan / execute / report |
| `goalctl` | **workspace member** `tools/goalctl` · package `goalctl`：doctor / validate / compile |
| 本仓 crate 落地 | `crates/evidence`；`tools/goalctl`；`tools/verifyctl` |
| `tools/xtask` | **未** member · **未**宣称 ship |
| live 凭据工具 | `scripts/live/build-foundationx-env.mjs`（#191）+ `export-foundationx-env.sh` |

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
| goalctl | `.agents/ssot/tools/goalctl` | `tools/goalctl` | `goalctl` **0.2.0** | **最小 Goal→Contract**：doctor/validate/compile + fixtures + `VERSION` 单测；**非**完整 authority plane |
| xtask | `.agents/ssot/tools/xtask` | `tools/xtask` | — | **未** member |
| verifyctl | `.agents/ssot/tools/verifyctl` | `tools/verifyctl` | `verifyctl` **0.1.0** | **最小** plan/execute/report；schema `verification-plan/v1` · `verification-run/v1`；`aggregate_report`/`write_report` 单测；可选 `with-evidence` |

### 自验证（本仓证据）

```bash
cargo test -p goalctl -p verifyctl -p evidence --all-targets
cargo run -p goalctl -- doctor
cargo run -p goalctl -- validate tools/goalctl/tests/fixtures/good_goal.yaml
cargo run -p goalctl -- compile tools/goalctl/tests/fixtures/good_goal.yaml -o /tmp/contract.json
cargo run -p verifyctl -- plan --contract /tmp/contract.json --changed tools/verifyctl -o /tmp/plan.json
cargo run -p verifyctl -- execute /tmp/plan.json -o /tmp/run.json
cargo run -p verifyctl -- report /tmp/run.json
```

输出契约：

| 产物 | schema |
|------|--------|
| Goal Contract JSON | `goal-contract/v1` + 稳定 digest |
| VerificationPlan | `verification-plan/v1` + `plan_digest` |
| RunResult | `verification-run/v1` + `exit_code` / `output_digest` / `status` |

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
- verifyctl 变更感知闭包 / 远程 runner / 签名证据链 / 完整 V0–V3 矩阵
- xtask / gate 工作流编排

## 相关

| 文档 | 说明 |
|------|------|
| [workspace-ssot-alignment.md](./workspace-ssot-alignment.md) | 总览 |
| [evidence-ssot-alignment.md](./evidence-ssot-alignment.md) | evidence crate 落地矩阵 |
| [draft-gap-matrix.md](./draft-gap-matrix.md) | draft → 本仓 gap |
| [SSOT_SYNC_OPS.md](./SSOT_SYNC_OPS.md) | 同步操作 |
| [.agents/ssot/SSOT.md](../../.agents/ssot/SSOT.md) | R6/R7 |

## 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-22 | #188：goalctl/verifyctl 进入 workspace members |
| 2026-07-22 | #190：VERSION / report API 单测；公共 API 清单闭合 |
| 2026-07-22 | #191：`scripts/live/build-foundationx-env.mjs`；对齐文档刷新 |

## SSOT 树补充（2026-07-22）

| 路径 | 说明 |
|------|------|
| `tools/*/plan/infra-rs-landing.md` | 本仓 member 落地说明 |
| `tools/*/plan/infra-rs-draft-*.md` | `.cargo/draft` 入库只读快照 |
| `.agents/ssot/AGENTS.md` | 操作说明（非空） |
| `.agents/ssot/adapters/README.md` | adapters 索引 |

