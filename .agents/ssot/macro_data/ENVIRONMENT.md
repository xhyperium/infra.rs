# ENVIRONMENT.md — macro_data SSOT 环境信息

## 来源

| 项目 | 值 |
|------|----|
| **本仓 SSOT** | `.agents/ssot/macro_data/`（域规格单一事实源；`SSOT.md` R6） |
| **领域模型** | 规格在 `.agents/ssot/core/domain_macro/`（实现 crate `macrox` 计划中，尚未落地） |
| **建立日期** | 2026-07-24 |

> macro_data 规格是**本仓 SSOT**（非外部仓库镜像）；provider 实现尚未落地（见下方实现状态）。外仓名字面量（`xhyper` / `macro_data.rs`）不得进入本树（`SSOT.md` §5.4）。

## 域分类

| 层级 | 域 | 类型 | 位置 |
|------|----|------|------|
| L0 内核 | `domain_macro` | 宏观共享模型 | `→ core/domain_macro/` |
| L0 内核 | `yield_curve` | 收益率曲线统一契约 | 本目录 |
| provider | `bea` | 美国经济分析局 | 本目录 |
| provider | `eastmoney` | 东方财富 | 本目录 |
| provider | `ecb` | 欧洲中央银行 | 本目录 |
| provider | `fred` | FRED 美联储 | 本目录 |
| provider | `japan_cb` | 日本央行 | 本目录 |
| provider | `jin10` | 金十数据 | 本目录 |
| provider | `treasury` | 美国财政部 | 本目录 |
| provider | `uk_cb` | 英国央行 | 本目录 |
| provider | `yahoo` | Yahoo 财经 | 本目录 |

> `domain_macro` 已移至 `.agents/ssot/core/`，详见 `core/AGENTS.md`。

## 实现状态

| 域 | spec_status | impl_status |
|----|-------------|-------------|
| domain_macro | draft | not_started（计划 `crates/macrox`，尚未落地） |
| yield_curve | draft | not_started |
| 所有 provider | draft | not_started |

## 与本仓的关系

- macro_data SSOT 是**本仓规格 SSOT**（R6），不包含 Rust 实现代码
- 领域模型 (`domain_macro`) 规格统一在 `core/` 维护
