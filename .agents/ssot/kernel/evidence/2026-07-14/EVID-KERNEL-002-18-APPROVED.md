# EVID-KERNEL-002-18-APPROVED — Spec Status → Approved

| 字段 | 值 |
|------|-----|
| Residual | **RES-18-APPROVED** → **CLOSED** |
| Date | 2026-07-14 |
| Branch | `feat/kernel-002-e2-migrate-banned-apis` |
| Spec | `SPEC-KERNEL-002` |
| Packet | [approval-packet.md](../../plan/approval-packet.md) |

## 授权依据

用户显式指令（会话）：

```text
- [ ] **RES-18-APPROVED** Spec `Status: Approved`执行授权
```

解释为：**人审授权执行**将 Spec `Status` 从 `Proposed` 改为 `Approved`，并关闭 residual `RES-18-APPROVED`。

决策形态对齐 approval-packet **选项 B（有条件 Approved）**：

- 接受 L1 主路径 + G2 + 正式 DEFER/partial residual；
- 书面保留 OPEN：`RES-API-007` · `RES-TEST-014/015/016`；
- **禁止** registry `stable`，直至 §18.3/§18.4 策略另行关闭。

## 落盘变更

| 文件 | 变更 |
|------|------|
| `.agent/SSOT/kernel/spec/spec.md` | `Status: Proposed` → **`Approved`** |
| `.agent/SSOT/kernel/spec/kernel-complete-spec.md` | 同上（与 SSOT 镜像） |
| `residual-open.txt` | RES-18-APPROVED **CLOSED** |
| gate / matrix / review / goal / tasks / plan / todo / approval-packet | 对齐 §18.1 Approved PASS；OPEN 去掉 RES-18-APPROVED |

## 明确不宣称

- §18 **全勾**（18.3 branch/mutants/miri 仍 OPEN；18.4 version 策略仍 OPEN）
- registry **`stable`**（保持 **incubating**）
- version **0.1.1**（RES-API-007 仍 OPEN）
- crates.io publish

## 校验

```bash
rg -n '^Status:' .agent/SSOT/kernel/spec/spec.md .agent/SSOT/kernel/spec/kernel-complete-spec.md
# 期望: Status:         Approved

rg -n '^RES-18-APPROVED:' .agent/SSOT/kernel/evidence/2026-07-14/residual-open.txt
# 期望: CLOSED
```

## Decision 记录

```text
Decision: B (conditional Approved) — executed under explicit user authorization
RES-18-APPROVED: CLOSED
§18 full: still OPEN
registry: incubating
```
