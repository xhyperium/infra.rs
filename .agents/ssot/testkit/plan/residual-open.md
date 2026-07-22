> **历史 residual 快照（2026-07-14，非当前台账）**：本文件只记录 `xhyper-testkit 0.1.1` ship 时点。当前 residual 见 [../spec/spec.md](../spec/spec.md) §8 与 [../gate/gate.md](../gate/gate.md) §2，其中 allocator/计数器确定性仍为 OPEN。

# Residual Open — SPEC-TESTKIT-002

| 字段 | 值 |
|------|-----|
| Spec | **Stable** |
| Package | testkit **0.1.1** |
| 更新 | 2026-07-14 |

## DEF

| ID | 状态 | 说明 |
|----|------|------|
| DEF-001…009 | CLOSED | 见 Approved ship |
| DEF-010 | **CLOSED** | mutants missed=0；Miri PASS；CI testkit-quality |

## Stable

**CLAIMED** 2026-07-14 — registry `status=stable` · SSOT Status=Stable · CI job `testkit-quality`.

## 仍可选（非阻塞 Stable）

| 项 | 状态 |
|----|------|
| full branch coverage ≥90% 独立机控 | OPTIONAL（line≥95% 已强制） |
| event_bus / KV suite 扩展 | **CLOSED**（Fake suite + redisx/kafkax/natsx mock 接线） || integration harness | INFRA-010+ |
