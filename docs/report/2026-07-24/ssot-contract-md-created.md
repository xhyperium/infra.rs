# 报告：创建 `.agents/ssot/CONTRACT.md`

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-24 |
| 分支 | `docs/add-ssot-contract-md` |
| 性质 | 文档补齐；无 Rust 源码变更 |
| 动机 | `CONTRACT_SPEC.md`（#379）引用 `./CONTRACT.md` 为数据模型 SSOT，文件此前缺失 |

## 1. 背景

PR #379 落地了合同合规基础设施：

- `.agents/ssot/CONTRACT_SPEC.md` — L1–L4 验证规则
- `scripts/quality-gates/check-contract-compliance.mjs`
- `.github/workflows/contract-compliance.yml`
- `.claude/hooks/contract-compliance-guard.mjs`

`CONTRACT_SPEC.md` 开篇写明：

> 数据模型以 [`CONTRACT.md`](./CONTRACT.md) 为准；验证规则以本文件为准。

根目录 `TODO.md` 亦列「创建 .agents/ssot/CONTRACT.md」。本变更补齐该断链。

## 2. 产出

| 路径 | 动作 |
|------|------|
| `.agents/ssot/CONTRACT.md` | **新建** v1.0.0：定义 / schema / Spec ID / layer / 兼容策略 / 状态词汇 / 合规 catalog 十域 |
| `.agents/ssot/SSOT.md` | 清单登记 `CONTRACT.md` + `CONTRACT_SPEC.md`；版本 v2.3.1 |
| `.agents/ssot/AGENTS.md` | 路由表 + 验证命令指向合同文件 |
| `docs/report/2026-07-24/ssot-contract-md-created.md` | 本报告 |

## 3. 设计要点

1. **职责分离**：`CONTRACT.md` = 数据模型；`CONTRACT_SPEC.md` = 验证规则；域 `spec/spec.md` = 行为合同。
2. **合规 catalog** 与脚本 `srcDirs` 对齐：kernel / testkit / contracts / 七个 L1 infra 叶域。
3. **诚实边界**：明确 complete 文件名、STATUS 分、`cargo check` 均不能单独推出生产就绪。
4. **子平面不冲突**：`market_data/CONTRACT.md` 等保持局部横向规则，不覆盖全局模型。

## 4. 验证

```bash
test -f .agents/ssot/CONTRACT.md
test -f .agents/ssot/CONTRACT_SPEC.md
# 链接可读
grep -q 'CONTRACT.md' .agents/ssot/CONTRACT_SPEC.md
grep -q 'CONTRACT.md' .agents/ssot/SSOT.md
node scripts/quality-gates/check-contract-compliance.mjs --level L1 --fail-level L1
```

（L1 结果以 PR 会话实测为准；本报告不伪造 exit 码。）

## 5. 非目标 / Follow-up

- 未改合规脚本字段解析（现有宽松正则仍适用）
- 未将 adapters / types / tools 纳入默认 `srcDirs`（见 `CONTRACT.md` §8.1）
- 未宣称任何 package stable

可选 follow-up：

1. 统一各域 `spec.md` front-matter 为 §3.3 形态  
2. 扩展 catalog 至 types / adapters（分 PR）  
3. 从 `TODO.md` 勾除「创建 .agents/ssot/CONTRACT.md」
