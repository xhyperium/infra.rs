# EVID-KERNEL-002-TEST-015 — mutation testing

| 字段 | 值 |
|------|-----|
| Date | 2026-07-14 |
| Residual | **RES-TEST-015** |
| Status | **OPEN / DEFER**（**不得 CLOSED**） |
| Spec | SPEC-KERNEL-002 §11 / §18.3 mutation score ≥90% |
| §18 | **仍 OPEN** |

## 工具可用性

| 工具 | 状态 | 证据 |
|------|------|------|
| `cargo-mutants` / `cargo mutants` | **ABSENT** | `/home/claude/.cargo/bin/` 无 `cargo-mutants`；`command -v` 等价静态检查：无该二进制 |
| 本会话短跑 `cargo mutants -p kernel --timeout 30` | **SKIP** | 工具未装；且 Executor 无 Shell |

## 目标命令（未执行）

```bash
which cargo-mutants || cargo mutants --version
cargo mutants -p kernel --timeout 30 2>&1 | tee /tmp/kernel-mutants.txt
```

`/tmp/kernel-mutants.txt`：**未生成**（未跑）。

## 相对门槛判定

| 门槛 | 实测 | 判定 |
|------|------|------|
| mutation score ≥ 90% | **未跑** | **DEFER OPEN** |
| 存活变异 = 0 或有豁免清单 | **无清单** | 不满足闭合 |

## DEFER 理由

1. **工具缺失**：环境未安装 `cargo-mutants`。
2. **耗时预算**：即使安装，全量 mutants 可能超 5–10 min 会话上限；短跑 `--timeout 30` 仅可作烟雾，不能单独证明 ≥90% score。
3. **无 shell**：本子代理无法 `cargo install cargo-mutants` 或执行短跑。

## 诚实结论

- **不得**宣称 RES-TEST-015 CLOSED。
- 状态保持 **OPEN / DEFER**。
- 关闭条件（仍有效）：安装 mutants → 对 `kernel` 跑通 → 报告 score ≥90% 或存活变异=0 / 书面豁免。

## 建议复跑（父代理 / 本地）

```bash
cargo install cargo-mutants --locked   # 若策略允许
cargo mutants -p kernel --timeout 30 2>&1 | tee /tmp/kernel-mutants.txt
# 完整分：去掉过紧 timeout 或按文档加 jobs；结果写入本 evidence
```
