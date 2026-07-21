> **历史 monorepo 记录**（infra.rs）：文中 archgate / `.architecture` 不构成本仓验收条件；本仓不移植 archgate。

# Round 7 Pass2

> Verifier: 计划完备性重检（非实现验收）  
> 对照: pass1 `round-07-findings.md` F7-1…3 · v1.1 I-CI-* / T-GATE-013/014 / T-CTC-018  
> 日期: 2026-07-14

## 原 FAIL 关闭状态（逐条 CLOSED/OPEN + 证据文件引用）

### F7-1 — §16.2 contract-testkit CI 命令无接线 Task

| 状态 | **OPEN**（部分覆盖） |
|------|----------------------|
| 已有 | **I-CI-CTC**「§16.2 三条命令」；**T-GATE-013** AC=`I-CI-CTC` |
| 缺口 | 三条命令 **未展开**为可粘贴验收：`cargo clippy -p contract-testkit …` / `cargo test -p contract-testkit` / `cargo test -p contract-testkit --test negative_implementations` |
| 缺口 | `plan.md` §6.3 **仍仅 §16.1**，无 Contract-testkit 子节 |
| 缺口 | `negative_implementations` 测试目标名 **未**写入 tasks AC 字面 |
| 判定 | Task 存在但 AC/主文仍「见 §16.2」级压缩 → **OPEN** |

### F7-2 — §16.4 Nightly 五项无 Task / 映射

| 状态 | **OPEN**（部分覆盖） |
|------|----------------------|
| 已有 | **I-CI-NIGHTLY**「§16.4 五项 nightly」；**T-GATE-014** |
| 缺口 | 五项 **未展开**：full mutation · Miri · property extended · broken-implementation matrix · workspace production graph audit |
| 缺口 | 无 nightly workflow/文档调度与 residual mutation DEFER 的交叉 AC 字面 |
| 判定 | **OPEN** |

### F7-3 — §14.4 contract-testkit 宏 expansion guard

| 状态 | **CLOSED** |
|------|------------|
| 证据 | **I-CTC-MACRO-CFG** |
| 证据 | **T-CTC-018**「薄宏 cfg(test)+fixture」AC=`I-CTC-MACRO-CFG` |
| 说明 | 覆盖 cfg(test) / compile fixture 方向；与 MACRO-001（testkit 导出）分工明确 |

## 新发现 FAIL（若有）

无（F7-4 branch 与 R10 F10-1 已由 T-GATE-008 双阈值闭合，不在本轮双计）。

## 本轮结论 FAIL

## fail_count: 2
