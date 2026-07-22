# Evidence — testkit

> 模块战役证据索引。testkit 是 Stable 已 ship 模块，单波次证据；证据实体在仓库根 `evidence/`（非本 spec 目录）。

## 权威（本 Goal 执行波）

ship 波次证据：仓库根 [`evidence/testkit/2026-07-14-stable-gates/`](../../../../evidence/testkit/2026-07-14-stable-gates/)

| 文件 / 类别 | 用途 |
|-------------|------|
| Stable gates 实测 | unit/contract/concurrency · property · mutants(missed=0) · Miri · line cov≥95% · test-graph-check |
| `plan/residual-open.md` | residual ledger SSOT（DEF-001…010 全 CLOSED · 1 OPTIONAL） |
| `plan/testkit-plan-10x-verdict.md` | 计划完备性终裁（fail_rounds=0 · PASS） |
| `plan/approval-packet.md` | 人审决策 A1–A10 |
| `plan/spec-inventory.md` | I-1…I-26 防遗漏枚举（反漂移索引） |

> spec §21.1 记录了 ship 时点的门禁实测结果表（权威镜像）。residual SSOT 永远以 `plan/residual-open.md` 为准。

## 历史

- testkit 单波次 ship（2026-07-14），无多波次继承问题。
- `plan/archive/`：十轮验收过程证据（round-findings / pass summaries），pass3 CLOSED 后归档，**非 live SSOT**，不得作为 §24 闭合证据。

## 2026-07-23 contract-testkit 维护候选

临时叠加原始摘要：[2026-07-23-contract-testkit-stack.md](./2026-07-23-contract-testkit-stack.md)。该文件不是最终主干 evidence；重放后必须替换/补充最终结果。

候选证据位于源码与可重放命令，不伪造静态 PASS 日志：

```bash
cargo test -p contract-testkit --all-targets
cargo test -p contract-testkit --test negative_implementations
cargo clippy -p contract-testkit -p contracts --all-targets -- -D warnings
node --test scripts/quality-gates/check-test-support-graph.test.mjs
node scripts/quality-gates/check-test-support-graph.mjs --json
node scripts/quality-gates/check-public-api.mjs -p contract-testkit --require-tool
```

最终 evidence 必须来自重放到最新 main 后的执行结果与 CI；当前候选不得写成已发布或人工批准。

## 禁止

- 把 `plan/archive/` 下的过程证据当成当前 live SSOT。
- 用归档的 round-findings 宣称 §24 全闭合——以 `plan/residual-open.md` + spec §21.1 实测为准。
