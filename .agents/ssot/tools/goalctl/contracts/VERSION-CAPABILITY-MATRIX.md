# goalctl Version–Capability Matrix

```text
Document:  VERSION-CAPABILITY-MATRIX
Version:   1.0.0
Status:    ACTIVE（DECISION-PACK-001 §12）
Rule:      禁止每个 PR 都宣称「0.1.0 已实现」
```

## 1. 里程碑

| 里程碑 | Package version | 命令 | Feature maturity | 实现 CR |
|--------|-----------------|------|------------------|---------|
| PR-0A | —（无 crate） | 无 | Schema/Policy 形状 | Foundation CR 已覆盖形状 ✅ |
| PR-1 | `0.1.0-dev` | `version`, `doctor`, `index` | Experimental | [CR-20260716-goalctl-impl-phase1](../../../../../docs/goal/change-requests/CR-20260716-goalctl-impl-phase1.md) **Approved**；**已实现** |
| PR-2 | `0.1.0-dev` | + `resolve`, `artifact` | Experimental | 同上；**已实现** |
| PR-3 | `0.1.0-rc.1` | + `reconcile` | Candidate | 同上；**已实现** |
| PR-4 | `0.1.0` | + `compile` | **Phase 1 Complete** | 同上；**已实现（包版本 0.1.0）** |
| PR-5+ | `0.2.0-dev`… | Evidence / Harness / … | 另阶段 | 另 CR |

## 2. 标签语义

| 标签 | 含义 |
|------|------|
| `0.1.0-dev` | 可构建、有测试；**不**声称 Phase 1 完成 |
| `0.1.0-rc.1` | reconcile 可用；compile 可能仍 UNSUPPORTED |
| `0.1.0` | doctor/index/resolve/reconcile/compile **全部** 达 AC |
| `>=0.2.0` | Phase 1 之后能力；不得塞进 0.1.0 |

## 3. 未实现命令

当前二进制版本若调用未列入上表的命令：

```text
exit 10
diagnostic GC-UNSUPPORTED-COMMAND
ok=false
```

## 4. Phase 1 完成检查清单（摘录）

- [ ] 无 `.config/goal`
- [ ] Authority Policy 自 Git 加载（非硬编码 rank）
- [ ] state-dir 合同遵守
- [ ] 确定性 index/resolve/compile 输出
- [ ] 不可验证 P0/P1 AC → compile 失败
- [ ] Legacy 不得独立 RELEASED
- [ ] `cargo test -p xhyper-goalctl`、lint-deps、crate-standard PASS
- [ ] 独立 Review + Evidence

## 5. 对照运行面（直至 Cutover CR）

| 今日 | 未来 goalctl |
|------|----------------|
| `just goal-check` | Shadow 并行，不替换 |
| `docs/goal/tools/*` | Legacy / 对照 |
| `tools/goalctl` | **存在**（Phase 1.1 Truth Hardening 0.1.1） |

Cutover 将 required check 切到 goalctl 必须 **独立 CR**（D10）。
