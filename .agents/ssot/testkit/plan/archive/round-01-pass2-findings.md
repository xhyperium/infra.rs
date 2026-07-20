# Round 1 Pass2

> Verifier: 计划完备性重检（非实现验收）  
> 对照: pass1 `round-01-findings.md` · v1.1 `spec-inventory.md#I-PATCH-v1.1` · `tasks.md` 补丁表 · `plan.md` v1.1.0  
> 日期: 2026-07-14  
> 纪律: 实现 ABSENT 不计 FAIL；仅计划遗漏；**部分覆盖 = OPEN**

## 原 FAIL 关闭状态（逐条 CLOSED/OPEN + 证据文件引用）

### F1-1 — §0 架构图正交平面

| 状态 | **CLOSED** |
|------|------------|
| 证据 | `spec-inventory.md` **I-1-ARCH-DIAGRAM**（Production vs Test graph 正交平面） |
| 证据 | `tasks.md` **T-DOC-004** AC=`I-1-ARCH-DIAGRAM` |
| 说明 | 交付物与 I-* 均已入库；实现仍 TODO 不影响本轮 |

### F1-2 — §1 不稳定来源完整清单

| 状态 | **CLOSED** |
|------|------------|
| 证据 | `spec-inventory.md` **I-1-IMPLICIT** 枚举 13 bullet（墙钟…吞错宏） |
| 证据 | `tasks.md` **T-INV-002** DONE（I-1-IMPLICIT + harness OOS 入 residual） |
| 说明 | 全文清单已脱离 I-DET 子集，可勾选 |

### F1-3 — §2.2 provider 宏硬编码行为全表

| 状态 | **OPEN**（部分覆盖） |
|------|----------------------|
| 已有 | `I-CTC-EXTRA` 含 `HC-TABLE` 名；`I-DEL-GATES` 含 `I-DEL-HC 硬编码清除表`；`T-DEL-009` AC=`I-DEL-HC` |
| 缺口 | inventory **未展开**规范硬编码 bullet：`stream 必须为空` / `server_time==0` / `position/balance 必须为空` / `query_order==Pending` / `invalid venue cancel 必须失败`；亦无隐藏 dep 迁移清单行 |
| 判定 | 仅有「HC-TABLE」标签 ≠ 可勾清除矩阵；按严格规则 **OPEN** |

### F1-4 — §3.3 Integration Harness 职责边界

| 状态 | **CLOSED** |
|------|------------|
| 证据 | **I-HARNESS-OOS** 完整职责表（Docker/Compose · Redis/Kafka/PG/TDengine · testnet · 网络故障 · 进程 kill · 真实端口 · 凭据 · Evidence artifact）+ 禁止进 testkit core |
| 证据 | `residual-open.md` DEFER 项引用 I-HARNESS-OOS |

### F1-5 — §3.4 Fixture 共享路径与两消费者准入

| 状态 | **CLOSED** |
|------|------------|
| 证据 | **I-FIXTURE-1…4**：领域 crate 管理 · 两消费者才允许 `test-support/fixtures/<schema>` · 禁 FixtureBuilder 回流 · builder 须真实字段+验证 |
| 证据 | **T-DEL-010** AC=`I-FIXTURE-3/4`（命名/回流） |
| 说明 | 归属与两消费者门闩已有可勾 I-*；实现级 freeze 由 I-FIXTURE 合同约束 |

## 新发现 FAIL（若有）

无（本轮无超出 pass1 范围的新计划遗漏）。

## 本轮结论 FAIL

## fail_count: 1
