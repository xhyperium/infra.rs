> **历史 monorepo 记录**（infra.rs）：文中 archgate / `.architecture` 不构成本仓验收条件；本仓不移植 archgate。

# Round 2 Pass2

> Verifier: 计划完备性重检（非实现验收）  
> 对照: pass1 `round-02-findings.md` · v1.1 I-PATCH · tasks 补丁表  
> 日期: 2026-07-14  
> 纪律: 部分覆盖 = OPEN

## 原 FAIL 关闭状态（逐条 CLOSED/OPEN + 证据文件引用）

### F2-1 — §4.1 Core 目录正例树未入库

| 状态 | **OPEN**（部分覆盖） |
|------|----------------------|
| 已有 | `I-DIR-CORE / I-DIR-CTC` 段；写「含 tests/manual_clock_*.rs、compile_fail、production_graph_guard」 |
| 已有 | `T-CLK-001` AC「目录符合 §4.1」 |
| 缺口 | inventory 仍以 **「见规范 §4.1」** 为主，**未**逐文件正例树（Cargo.toml/README/AGENTS/CHANGELOG、`src/{lib,clock}.rs`、四个 tests 文件名全表）；与 plan §1.3「不得只写见 §xx」冲突 |
| 缺口 | `T-CLK-001` **未**绑定 `I-DIR-CORE`；`T-CLK-014/016/017` 仍未钉规范文件名 |
| 判定 | 摘要引用 ≠ 可勾 SSOT 树 → **OPEN** |

### F2-2 — §4.1 新模块准入八问 + RFC 无任务

| 状态 | **CLOSED** |
|------|------------|
| 证据 | **I-DIR-RFC**：新模块 = 准入八问 + RFC |
| 证据 | **T-GATE-015** AC=`I-DIR-RFC`（依赖 T-FREEZE-001） |

### F2-3 — §4.2 contract-testkit 测试目录与自测文件

| 状态 | **OPEN**（部分覆盖） |
|------|----------------------|
| 已有 | I-DIR-CTC 提及 suite_self_tests、compile_fail |
| 已有 | **T-CTC-023** AC=`I-DIR-CTC`（suite_self_tests + compile_fail） |
| 缺口 | 正例树仍「见规范 §4.2」，**未**入库 src 下按 trait 分文件全表 + tests 布局为可勾行 |
| 缺口 | 无 contract-testkit 包级 compile_fail 与 adapter 禁依赖的独立 I-DIR 行（与 T-CTC-018 宏守卫不同） |
| 判定 | 关键测试文件有 Task，**目录 SSOT 仍部分** → **OPEN** |

## 新发现 FAIL（若有）

无。

## 本轮结论 FAIL

## fail_count: 2
