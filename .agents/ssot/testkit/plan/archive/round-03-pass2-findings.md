> **历史 monorepo 记录**（infra.rs）：文中 archgate / `.architecture` 不构成本仓验收条件；本仓不移植 archgate。

# Round 3 Pass2

> Verifier: 计划完备性重检（非实现验收）  
> 对照: pass1 `round-03-findings.md` fail_count=8 · v1.1 I-CLK-* / T-CLK-021…025  
> 日期: 2026-07-14  
> 纪律: 部分覆盖 = OPEN

## 原 FAIL 关闭状态（逐条 CLOSED/OPEN + 证据文件引用）

### F3-1 — §7.3/§7.4 枚举属性（non_exhaustive + derives）

| 状态 | **OPEN**（部分覆盖） |
|------|----------------------|
| 已有 | **I-CLK-NE** `non_exhaustive`；**T-CLK-022** AC 含 I-CLK-NE |
| 缺口 | pass1 要求 **derive 集合**（Fault: Debug/Clone/Copy/PartialEq/Eq；Error: Debug + non_exhaustive）**未**写入 I-CLK-FAULT/ERR 或 T-CLK-003/004/022 AC |
| 判定 | non_exhaustive 已映射，derives 仍缺 → **OPEN** |

### F3-2 — §7.7 禁带符号纳秒 fetch_add

| 状态 | **CLOSED** |
|------|------------|
| 证据 | **I-CLK-NOSIGN** 禁 signed fetch_add/delta |
| 证据 | **T-CLK-022** AC=`I-CLK-NE/NOSIGN/NOREWRAP` |

### F3-3 — §7.7 禁 release 模式算术回绕

| 状态 | **CLOSED** |
|------|------------|
| 证据 | **I-CLK-NOREWRAP** 禁 release 回绕 |
| 证据 | **T-CLK-022**；与 I-TEST-MUT wrapping_* 可交叉 |

### F3-4 — §7.8 单调路径失败不改状态 + 禁 signed delta

| 状态 | **CLOSED** |
|------|------------|
| 证据 | **I-CLK-FAIL-ATOMIC** 失败不改状态；**I-CLK-NOSIGN** 覆盖 signed delta |
| 证据 | **T-CLK-023** AC=`I-CLK-FAIL-ATOMIC/NO-ONESHOT`（失败原子）；signed 由 T-CLK-022 |

### F3-5 — §7.9 禁 one-shot 队列 + scripted fault 准入

| 状态 | **OPEN**（部分覆盖） |
|------|----------------------|
| 已有 | **I-CLK-NO-ONESHOT** + **T-CLK-023**（禁 one-shot） |
| 已有 | inventory **I-CLK-SCRIPTED** 两消费者准入 |
| 缺口 | **无任何 Task AC 引用 I-CLK-SCRIPTED**；scripted fault 准入未任务化 |
| 判定 | 合并 FAIL 中 scripted 半支未闭合 → **OPEN** |

### F3-6 — §7.11 now() 锁失败 → Unavailable

| 状态 | **CLOSED** |
|------|------------|
| 证据 | **I-CLK-LOCK-UNAVAIL** |
| 证据 | **T-CLK-024** AC=`I-CLK-LOCK-UNAVAIL/POISON` |

### F3-7 — §7.11 monotonic() 锁中毒恢复分项

| 状态 | **OPEN**（部分覆盖） |
|------|----------------------|
| 已有 | **I-CLK-POISON**「mono 恢复分项」；**T-CLK-024** |
| 缺口 | 规范五条 **未逐条入库**：不在持锁执行调用方代码；poison 恢复 inner；不伪造零；不 panic；文档明确；且不得擅自改 `Clock::monotonic` 合同 |
| 判定 | 「分项」标签无 bullet → **OPEN** |

### F3-8 — §7 方法签名与返回类型矩阵

| 状态 | **OPEN**（部分覆盖） |
|------|----------------------|
| 已有 | **I-CLK-SIG** 方法名缩写表；**T-CLK-025**「逐方法」 |
| 缺口 | 无签名级 SSOT（如 `fn advance_wall(...) -> Result<Timestamp, ManualClockError>`、fault 三 API 均 Result 等）；仍为语义缩写 |
| 判定 | 无法仅凭 inventory 验收返回类型 → **OPEN** |

## 新发现 FAIL（若有）

无。

## 本轮结论 FAIL

## fail_count: 4
