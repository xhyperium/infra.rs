<!-- infra.rs tools SSOT -->
> **本仓 SSOT**：`.agents/ssot/tools/goalctl/`  
> **无** `tools/goalctl` workspace member。下文 Phase/Approved 不得当作本仓交付证明。  
> 生产级 Goal/Spec：[`goal/goalctl-production-goal.md`](goal/goalctl-production-goal.md) · [`spec/goalctl-production-spec.md`](spec/goalctl-production-spec.md)  
> 对齐：[docs/ssot/tools-ssot-alignment.md](../../../../docs/ssot/tools-ssot-alignment.md)

# goalctl 规划包索引

```text
Package:            xhyper-goalctl 0.1.0
Path:               tools/goalctl
Status:             Phase 1 Complete（doctor…compile）
Implementation:     存在（workspace member）
Control plane:      禁止 .config/goal（见 CR）
Approval:           Foundation + Impl Phase1 Approved（2026-07-16）
```

## 阅读顺序

| 顺序 | 路径 | 作用 |
|------|------|------|
| 1 | [goal/Goal-终态目标.md](./goal/Goal-终态目标.md) | 北极星与 8 个系统问题 |
| 2 | [SPEC-终态规范.md](./SPEC-终态规范.md) | 终态一页纸 + PR-0A 指针 |
| 3 | [decisions/DECISION-PACK-001.md](./decisions/DECISION-PACK-001.md) | **10 项裁决（APPROVED / DECIDED）** |
| 4 | [docs/goal/.../CR-20260716-goalctl-foundation.md](../../../../docs/goal/change-requests/CR-20260716-goalctl-foundation.md) | 关联 CR（**Approved**） |
| 5 | [contracts/](./contracts/) | **CLI / state-dir / version 矩阵（PR-0A）** |
| 6 | [schemas/](./schemas/) | **Schema Registry（PR-0A）** |
| 7 | [docs/goal/schema/authority-policy.yaml](../../../../docs/goal/schema/authority-policy.yaml) | **Authority rank 机器 SSOT（D02）** |
| 8 | [spec/01–06](./spec/) | 工作流 / SPEC / DESIGN / PLAN / 治理 / PR 包摘要 |
| 9 | [spec/07-Goalctl-完整汇总.md](./spec/07-Goalctl-完整汇总.md) | 01–06 合订（保持与分册同步） |
| 10 | 两份 AUDIT | 历史风险地图；冲突以 DECIDED + 本目录 contracts/schemas 为准 |

## PR-0A 交付物（本轮）

| 制品 | 路径 |
|------|------|
| Authority Policy | `docs/goal/schema/authority-policy.yaml` |
| Schema Registry | `schemas/*.schema.json` + `schemas/README.md` |
| CLI 合同 | `contracts/CLI-CONTRACT.md` |
| 运行态合同 | `contracts/RUNTIME-STATE.md` |
| 版本能力矩阵 | `contracts/VERSION-CAPABILITY-MATRIX.md` |
| 权威表投影 | `docs/goal/00-authority-map.md` 新增 goalctl 行 |

## 当前事实

- **是**：规划合同 + **PR-1 只读实现**（`tools/goalctl`）。
- **不是**：Phase 1 完成（尚无 resolve/reconcile/compile）、Agent、required CI cutover。
- **对照运行面**：`docs/goal/tools/*`、`just goal-check` 仍为主；goalctl 不替换之。
- **可用命令**：`doctor` / `index` / `version`。

## 门闩

```text
DECISION-PACK-001 + Foundation CR  ✅
PR-0A Schema/Policy/CLI/state      ✅
实现 CR                            ✅ Approved
PR-1…PR-4 Phase 1 MVA              ✅ 0.1.0
  → Agent / Native Gate / required CI 另议（非 Phase 1）
```

```bash
cargo run -p xhyper-goalctl -- version
cargo run -p xhyper-goalctl -- reconcile --module goalctl --json
cargo run -p xhyper-goalctl -- compile --module goalctl --json
```

## 形状自检

```bash
test ! -d .config/goal
test -f docs/goal/schema/authority-policy.yaml
python3 -c "import yaml; yaml.safe_load(open('docs/goal/schema/authority-policy.yaml'))"
python3 - <<'PY'
import json
from pathlib import Path
for p in sorted(Path('.agents/ssot/tools/goalctl/schemas').glob('*.schema.json')):
    json.load(p.open())
    print('ok', p.name)
PY
just goal-check
```

## 维护

- 改 rank / class：改 `authority-policy.yaml` + CR（禁止只改 Rust）。
- 改 CLI 语义：升 `contracts/CLI-CONTRACT.md` 版本。
- 改输出字段：升对应 `schemas/*.schema.json`（D06 兼容规则）。
- 审计文是历史风险地图；**冲突时以已批准 CR + DECIDED + contracts/schemas 为准**。

---

## Kernel 布局导航（结构对齐附录）

> 以下为 `.agents/ssot/kernel/` 结构对齐附录；**不替代**上文战役/索引正文。
> Spec SSOT 入口：[`spec/spec.md`](spec/spec.md)。空层不宣称 Done。

| 管线层 | 路径 |
|--------|------|
| Goal | [goal/](goal/) |
| Spec | [spec/spec.md](spec/spec.md) |
| Design | [design/](design/) |
| Plan | [plan/](plan/) |
| Tasks | [tasks/](tasks/) |
| Prompt | [prompt/](prompt/) |
| **Code** | 见上文 / 实现 crate（禁止写在 `.agents/ssot/`） |
| Test | [test/](test/) |
| Review | [review/](review/) |
| Release | [release/](release/) |
| Retrospective | [retrospective/](retrospective/) |

| 横切 | 路径 |
|------|------|
| Matrix | [matrix/](matrix/) |
| Gate | [gate/](gate/) |
| Evidence | [evidence/](evidence/) |

