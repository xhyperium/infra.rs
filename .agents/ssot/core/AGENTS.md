# `.agents/ssot/core/` — 共享领域规格 (Shared Domain Core)

本目录汇聚跨子系统共享的领域模型规格。适配器和 provider 规格不在本层，分别在 `market_data/` 和 `macro_data/` 下。

## 1. 本层域树

| 路径 | 来源 | 角色 |
|------|------|------|
| `domainx/` | market_data | 共享交易值对象：Order、Position、Trade、Portfolio |
| `domain_market/` | market_data | 市场数据域模型：Tick、Quote、Bar、OrderBook |
| `domain_exchange/` | market_data | 交易所抽象：VenueAdapter trait、StreamType、AdapterError |
| `domain_macro/` | macro_data | 宏观经济共享模型：Period、Vintage、RevisionChain |

## 2. 标准文档结构

```text
goal/goal.md       # 目标：为什么需要、解决什么问题
design/design.md   # 设计：架构决策（ADR）、权衡
spec/spec.md       # 规格：API 契约、类型约束
review/            # 复审记录：逐轮评审结论
evidence/          # 验证证据
matrix/            # 追溯矩阵（门禁 → 实现 → 测试）
```

**Code 不在本树**：实现路径在对应仓库的 `crates/` 下，禁止在 SSOT 目录写实现代码。

## 3. 门禁总览

| 域 | 总门禁 | verified | pending | blocked | 关键阻塞 |
|----|--------|----------|---------|---------|---------|
| `domainx` | 5 | 4 | 0 | 1 | xhyper-canonical 未引入 |
| `domain_market` | 6 | 5 | 0 | 1 | 同上 |
| `domain_exchange` | 6 | 6 | 0 | 0 | adapter HTTP 映射待实现 |
| `domain_macro` | - | - | draft | - | 规格 draft，待落地 |

| **合计** | **17+** | **15** | **0** | **2** |

## 4. 跨域依赖

```
core/domainx ←── core/domain_market
             ←── core/domain_exchange
             
core/domain_macro ←── macro_data/bea, ecb, fred, ...
                 ←── macro_data/yield_curve
```

- `domainx` 是交易领域共享基础，domain_market 和 domain_exchange 依赖其类型
- `domain_macro` 是宏观领域共享基础，所有 macro_data provider 依赖其模型
- 两层之间**无直接依赖**

## 5. 变更规则

- 修改域规级必须走 worktree + PR
- 新增域需先创建 goal → design → spec → matrix 四层
- 门禁状态变更需同步更新本文件统计
