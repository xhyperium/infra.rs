> **历史 monorepo 记录**（infra.rs）：文中 archgate / `.architecture` 不构成本仓验收条件；本仓不移植 archgate。

# Round 4 Pass2

> Verifier: 计划完备性重检（非实现验收）  
> 对照: pass1 `round-04-findings.md` fail_count=4 · v1.1 I-DEL-* / T-DEL-007…010  
> 日期: 2026-07-14

## 原 FAIL 关闭状态（逐条 CLOSED/OPEN + 证据文件引用）

### F4-1 — §8.1 external downstream 调用点 = 0

| 状态 | **CLOSED** |
|------|------------|
| 证据 | **I-DEL-GATES** 含 `I-DEL-EXT0` |
| 证据 | **T-DEL-007** AC=`I-DEL-*`（WS0/EXT0/FIX/SPEC） |
| 证据 | `residual-open.md`：`external downstream 宏 | monorepo 外 N/A；I-DEL-EXT0 | DEFER(accepted: workspace-only)` |
| 说明 | 书面 N/A 裁定 + 任务门槛齐全 |

### F4-2 — §8.2 禁止兼容/空壳替代宏 + 迁移方案矩阵

| 状态 | **OPEN**（部分覆盖） |
|------|----------------------|
| 已有 | **I-DEL-MOCK-PATHS 五路径** · **I-DEL-NO-REPL**；**T-DEL-008** |
| 缺口 | 「五路径」**未逐条展开**为规范 §8.2 五条（手写 fake / 消费方 impl trait / 多实现→contract-testkit / Arc\<Mutex\<Vec\<Call\>\>\> / 复杂 expectation 先证明） |
| 缺口 | 无 README/CHANGELOG 专条 Task 写清「无替代宏」之外的路径矩阵勾选行 |
| 判定 | 有 ID 无矩阵内容 → **OPEN** |

### F4-3 — §8.3 拆分时硬编码行为清除清单

| 状态 | **OPEN**（部分覆盖） |
|------|----------------------|
| 已有 | **I-DEL-HC**；**T-DEL-009** AC=`I-DEL-HC`（依赖 T-CTC-005） |
| 缺口 | 与 F1-3 / F5-4 同源：清除表 **未列出** stream 空 / server_time==0 / balance 空 / Pending / cancel 失败等可勾行 |
| 判定 | **OPEN** |

### F4-4 — §8.4 builder 命名规则

| 状态 | **CLOSED** |
|------|------------|
| 证据 | **I-FIXTURE-4** builder 须真实字段+验证 |
| 证据 | **T-DEL-010** AC=`I-FIXTURE-3/4` |

## 新发现 FAIL（若有）

无。

## 本轮结论 FAIL

## fail_count: 2
