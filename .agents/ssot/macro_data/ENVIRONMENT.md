# ENVIRONMENT.md — macro_data SSOT 环境信息

## 来源

| 项目 | 值 |
|------|----|
| **外部仓库** | `/home/workspace/macro_data.rs` |
| **本仓路径** | `.agents/ssot/macro_data/` |
| **导入方式** | `cp -rf` 全文复制 |
| **导入日期** | 2026-07-24 |

## 域分类

| 层级 | 域 | 类型 | spec_status | impl_status |
|------|----|------|-------------|-------------|
| L0 内核 | `domain_macro` | 宏观共享模型 | draft | partial (`crates/macrox`) |
| L0 内核 | `yield_curve` | 收益率曲线统一契约 | draft | not_started |
| provider | `bea` | 美国经济分析局 | draft | not_started |
| provider | `eastmoney` | 东方财富 | draft | not_started |
| provider | `ecb` | 欧洲中央银行 | draft | not_started |
| provider | `fred` | FRED 美联储 | draft | not_started |
| provider | `japan_cb` | 日本央行 | draft | not_started |
| provider | `jin10` | 金十数据 | draft | not_started |
| provider | `treasury` | 美国财政部 | draft | not_started |
| provider | `uk_cb` | 英国央行 | draft | not_started |
| provider | `yahoo` | Yahoo 财经 | draft | not_started |

## 核心概念

| 概念 | 说明 |
|------|------|
| `SourceSeriesId` | 来源/数据集/原始系列键的稳定不透明标识 |
| `IndicatorId` | 本仓规范指标标识 |
| `CountryCode` | ISO 3166-1 alpha-2 |
| `CurrencyCode` | ISO 4217 |
| `Period` | Date/Month/Quarter/Year 枚举 |
| `Vintage` | 数据在某个 as_of 时点可见的版本 |
| `ObservationIdentity` | source + series + indicator + subject + period + vintage |
| `ScaledUnit` | 维度 + 币种 + scale10 + 基期 + 变化口径 |
| `NumericValue` | Decimal 或有原因的空缺值 |
| `RevisionChain` | 版本链，append-only |
| `IndicatorCategory` | 收入/物价/就业/货币/贸易/财政 六大类 |

## domain_macro 设计决策

| ADR | 主题 | 要点 |
|-----|------|------|
| DM-001 | 验证值对象 | 私有字段 + TryFrom |
| DM-002 | 来源身份与期间分离 | SourceSeriesId != IndicatorId |
| DM-003 | 十进制与显式单位 | 生产 wire 使用定点值 |
| DM-004 | 聚合根维护修订与快照 | RevisionChain::append 原子性 |
| DM-005 | 低基数错误观测 | 错误携带稳定错误码 |

## provider 通用要求

- 只允许离线 fixture 解析，禁止实现网络 I/O、认证、缓存、重试
- 来源身份保留原始键，不得直接充当 IndicatorId
- 缺失值必须带缺失原因，不转零或删除
- 同一 fixture 重复解析必须产生相同结果

## 验证证据

- 10 轮质量门禁全量通过 (round-attestation.json)
- 130 文件 SHA-256 基线 (接管基线提交 a3cb50c12)
- evidence/rounds/ 下有 round-01 至 round-10 的完整记录

## 关键文件

| 文件 | 用途 |
|------|------|
| `manifest.json` | 机器可读的域结构与状态清单 |
| `AGENTS.md` | Agent 操作说明 |
| `evidence/round-attestation.json` | 10 轮验证证明 |

## 与本仓的关系

- macro_data SSOT 是规格层镜像，不包含 Rust 实现代码
- 实现路径在 macro_data.rs 仓库的 crates/ 下
- domain_macro 声称 crates/macrox 有 partial 实现
- 所有 9 个 provider 适配器实现均为 not_started
