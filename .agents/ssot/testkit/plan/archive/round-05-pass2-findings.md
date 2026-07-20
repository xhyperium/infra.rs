# Round 5 Pass2

> Verifier: 计划完备性重检（非实现验收）  
> 对照: pass1 `round-05-findings.md` fail_count=5 · v1.1 I-CTC-EXTRA / T-CTC-018…023  
> 日期: 2026-07-14

## 原 FAIL 关闭状态（逐条 CLOSED/OPEN + 证据文件引用）

### F5-1 — §9.1 反原则：不是验证 mock 默认值

| 状态 | **CLOSED** |
|------|------------|
| 证据 | **I-CTC-PRIN**（I-CTC-EXTRA） |
| 证据 | **T-CTC-021** AC=`I-CTC-NO-ADAPTER/PRIN` |

### F5-2 — §9.2 Suite 禁止依赖具体 adapter crate

| 状态 | **CLOSED** |
|------|------------|
| 证据 | **I-CTC-NO-ADAPTER** |
| 证据 | **T-CTC-021**（与 PRIN 同任务） |

### F5-3 — §9.3 Fake / Sandbox / Real 三层验证 bullet 矩阵

| 状态 | **OPEN**（部分覆盖） |
|------|----------------------|
| 已有 | I-CTC-EXTRA 标签 `FAKE/SANDBOX/REAL/SEP`；**T-CTC-020**「Fake/Sandbox/Real 矩阵」 |
| 缺口 | **零**分项 bullet 入库：Fake 6 条（trait object / 配置 I/O / 错误注入 / 生命周期 / 调用记录 / 无外部 IO）；Sandbox 5 条；Real 6 条 |
| 缺口 | 首批 suite 仅 Fake 层时的 DEFER(Sandbox/Real) 条件未写入 residual |
| 判定 | 「矩阵」名无勾选行 → **OPEN** |

### F5-4 — §9.5 硬编码禁令全表 + 最小 profile / 禁复杂 DSL

| 状态 | **OPEN**（部分覆盖） |
|------|----------------------|
| 已有 | **I-CTC-MIN-PROFILE** + **T-CTC-022**（最小 profile 禁 DSL）→ 此半支可视为映射 |
| 已有 | **I-CTC-HC-TABLE** 标签 + 交叉 **T-DEL-009** |
| 缺口 | 硬编码禁令全表仍未展开（`server_time==0` / `stream 必须为空` / `balance 必须为空` / `order 必须 Pending`） |
| 判定 | 合并 FAIL 中硬编码半支未闭合 → **OPEN** |

### F5-5 — §9.6 ContractFailure 字段与禁 unwrap

| 状态 | **CLOSED** |
|------|------------|
| 证据 | **I-CTC-FAIL-FIELDS** / **I-CTC-NO-UNWRAP** |
| 证据 | **T-CTC-019**「ContractFailure 三字段 + 禁 unwrap」 |
| 说明 | 「三字段」对齐 §9.6 `contract` / `case` / `detail`；与 Result 失败可定位合同足够绑定 |

## 新发现 FAIL（若有）

无。

## 本轮结论 FAIL

## fail_count: 2
