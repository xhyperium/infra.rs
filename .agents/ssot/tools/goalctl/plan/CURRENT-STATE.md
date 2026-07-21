# CURRENT-STATE — xhyper-goalctl

| 字段 | 值 |
|------|-----|
| Package | `xhyper-goalctl` |
| Path | `tools/goalctl` |
| Version (campaign target) | `0.1.1`（Truth Hardening；开战役时基线 `0.1.0`） |
| Goal | `GOAL-GOALCTL-002` PROPOSED |
| Spec | `SPEC-GOALCTL-002` PROPOSED |
| Plan | `PLAN-GOALCTL-002-phase1.1-v1` |
| Updated | 2026-07-16 |

## 已实现（机器可调用）

| 命令 | 状态 | 备注 |
|------|------|------|
| `version` / `--version` | 可用 | |
| `doctor` | 可用 | 可 dirty 警告；非 enforcing snapshot |
| `index` | 可用 | HEAD + clean Cargo；非 HEAD fail-closed |
| `resolve` | 可用 | committed policy/authority |
| `artifact inspect\|index` | 可用 | 已 committed 读 |
| `reconcile` | 可用 | 已禁目录→VERIFIED/OK |
| `compile` | 可用 | 已 commit/tree + approval 内容校验 |
| `--source-commit` | 已全 subject-bound | |
| `--trust-level` | 已实现 | |

## 未实现（诚实）

- Evidence verify / harness / gate evaluate / shadow / replay
- Agent Writer、GitHub write、required CI
- Identity FULL（长期 DEGRADED 无 numeric id）
- Bootstrap trust root
- Schema codegen 双向一致
- 跨语言 canonical golden
- 完整 Goal→Spec→Plan→Task 编译（默认仍为模块模板；不得虚构验证 PASS）

## 已知风险

| 风险 | 战役处理 |
|------|----------|
| artifact live 污染 | T-P0-001 |
| reconcile 假阳性 | T-P0-002 |
| compile commit/tree 漂移 | T-P0-003 |
| 空壳 approval | T-P0-005 |
| 文档「尚未存在」误导 | T-P1-008 |

## 禁止

- `.config/goal`
- 目录存在 → VERIFIED / RELEASED / OK
- Writer 自批 / 自写 G0–G11 PASS
- 未 Cutover CR 替换 `just goal-check`

## 对照运行面

| 今日 SSOT | goalctl 角色 |
|-----------|--------------|
| `just goal-check` / `docs/goal/tools/*` | 不替换；shadow 未启用 |
| `tools/goalctl` | **存在**；Phase 1.1 硬化中 |
