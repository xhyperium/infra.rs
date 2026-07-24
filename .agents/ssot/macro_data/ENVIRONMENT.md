# ENVIRONMENT.md — macro_data SSOT 环境信息

## 来源

| 项目 | 值 |
|------|----|
| **外部仓库** | `/home/workspace/macro_data.rs` |
| **本仓路径** | `.agents/ssot/macro_data/` |
| **导入方式** | `cp -rf` 全文复制 |
| **导入日期** | 2026-07-24 |

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
| domain_macro | draft | partial (`crates/macrox`) |
| yield_curve | draft | not_started |
| 所有 provider | draft | not_started |

## 与本仓的关系

- macro_data SSOT 是规格层镜像，不包含 Rust 实现代码
- 领域模型 (`domain_macro`) 规格统一在 `core/` 维护
