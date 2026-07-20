# Round 7 — §14–§16 图隔离 / Archgate 全规则 ID / CI 命令

| 字段 | 值 |
|------|-----|
| Scope | 计划完备性（非实现验收） |
| Spec | `SPEC-TESTKIT-002` §14–§16 |
| 日期 | 2026-07-14 |

## 检查项

| # | 规范点 | 计划应覆盖内容 |
|---|--------|----------------|
| 7.1 | §14.1 GRAPH-001…005 | 规则 ID + 机控命令 + 失败 exit≠0 |
| 7.2 | §14.2 Release tree | release binary normal graph 无 test-support |
| 7.3 | §14.3 Source guard | 生产 src 禁 `use testkit`；允许 cfg(test)/tests |
| 7.4 | §14.4 Macro expansion guard | contract-testkit 宏仅 `#[cfg(test)]`；compile fixture；不导出 production symbols |
| 7.5 | §15 Archgate | 11 条 TESTKIT-* 规则 ID 全表 + 实现 Task |
| 7.6 | §16.1 Core CI | 9 条命令全收录 |
| 7.7 | §16.2 contract-testkit CI | clippy / test / negative_implementations |
| 7.8 | §16.3 test-graph-check | 输出列齐全 |
| 7.9 | §16.4 Nightly | 5 项矩阵有调度/Task 映射 |

## PASS

1. **§14.1 规则 ID 全覆盖**：plan §6.1 + **I-GRAPH-1…5** = `TESTKIT-GRAPH-001…005`；`T-GATE-001/002` 实现 + exit≠0。
2. **§14.2 / 生产图**：`T-GATE-005` production_graph_guard；指标 `testkit_production_dependents=0`；plan §6.3 含 `test-graph-check`。
3. **§14.3 Source guard**：`T-GATE-006`「生产 src 禁 use testkit；允许 cfg(test)/tests」。
4. **§15 Archgate 规则 ID 全集在 inventory + plan**：

   | 规则 ID | Inventory |
   |---------|-----------|
   | TESTKIT-LAYER-001 | I-AG-LAYER |
   | TESTKIT-DEP-001 | I-AG-DEP |
   | TESTKIT-FEATURE-001 | I-AG-FEAT |
   | TESTKIT-API-001 | I-AG-API |
   | TESTKIT-MACRO-001 | I-AG-MACRO |
   | TESTKIT-PLACEHOLDER-001 | I-AG-PLACE |
   | TESTKIT-TIME-001 | I-AG-TIME |
   | TESTKIT-IO-001 | I-AG-IO |
   | TESTKIT-GRAPH-001（archgate 侧） | I-AG-GRAPH |
   | TESTKIT-HIDDEN-DEP-001 | I-AG-HIDDEN |
   | TESTKIT-NAMING-001 | I-AG-NAME |

   `T-GATE-003` AC「§15 列表」；NAMING 首期 warning 与 residual/A9 一致。

5. **§16.1 Core 命令在 plan §6.3 齐全**：
   - `cargo fmt -- --check`
   - `cargo clippy -p testkit --all-targets -- -D warnings`
   - `cargo test -p testkit`
   - `cargo llvm-cov -p testkit --fail-under-lines 95`
   - `cargo mutants -p testkit`
   - `cargo miri test -p testkit`
   - `cargo run -p archgate -- --json`（入口缺失可 residual）
   - `cargo run -p xtask -- lint-deps`
   - `cargo run -p xtask -- crate-standard --check`
6. **§16.3 输出列**：plan §6.1 / `T-GATE-001`「§14 输出列齐全」对齐 package/consumer/kind/target/feature path/verdict。
7. **§16.1 与 lint-deps 分工**：规范 §14.1 写 lint-deps 检查 GRAPH；§16.3 建议专用 `test-graph-check`——计划采用专用命令，**不视为遗漏**。
8. **I-CI-CORE / I-CI-CTC / I-CI-PROD / I-CI-NIGHTLY** inventory 入口齐全。

## FAIL

### F7-1 — §16.2 contract-testkit CI 命令无接线 Task

- **规范引用**：§16.2 必须执行：
  ```bash
  cargo clippy -p contract-testkit --all-targets -- -D warnings
  cargo test -p contract-testkit
  cargo test -p contract-testkit --test negative_implementations
  ```
- **缺失**：
  - **I-CI-CTC** 仅「§16.2」指针，无命令展开；
  - `T-GATE-007` AC **明确写「CI 接线 core 命令 §16.1」**，不含 §16.2；
  - `T-CTC-011/012` 覆盖自测语义，**不是** CI workflow/文档接线；
  - plan §6.3 命令块也未列出 §16.2 三条。
- **建议补丁**：
  1. 扩展 `T-GATE-007` 或新增 `T-GATE-013`：CI/文档接线 §16.2 三条；AC 引用展开后的 **I-CI-CTC-1…3**。
  2. plan §6.3 增加 Contract-testkit 子节与规范对齐。
  3. `negative_implementations` 测试目标名写入 tasks（与 §16.2 字面一致）。

### F7-2 — §16.4 Nightly 五项无 Task / 时间表映射

- **规范引用**：§16.4 Nightly：
  - full mutation
  - Miri
  - property test extended cases
  - contract suite broken-implementation matrix
  - workspace production graph audit
- **缺失**：
  - **I-CI-NIGHTLY** 仅标签，无 5 项展开；
  - 无 `T-GATE-*` / `T-CI-*` 绑定 nightly workflow 或文档化调度；
  - residual 仅 DEFER「mutation 进 required PR CI」；**A10** 问 required vs nightly，但未把 §16.4 矩阵写成验收包；
  - gap-matrix 未单列 §16.4。
- **建议补丁**：
  1. inventory 展开 **I-CI-NIGHTLY-1…5**。
  2. 新增 `T-GATE-014`（或并入 A10 关闭条件）：nightly job/文档列出 5 项 + Evidence 路径。
  3. 与 residual mutation DEFER 交叉引用：required 可 DEFER，**nightly full mutation 不可静默消失**。

### F7-3 — §14.4 contract-testkit 宏 expansion guard 无显式 Task

- **规范引用**：§14.4 若存在 contract-testkit 宏，必须：
  - 只能生成 `#[cfg(test)]` 项；
  - 有 compile fixture 证明；
  - 不导出 production symbols。
- **缺失**：
  - `T-CTC-010` 仅「薄声明宏；无隐藏依赖；deps 在 Cargo.toml」——**未写 cfg(test)/production symbols/compile fixture**；
  - 无 compile_fail/trybuild Task 针对 contract-testkit 宏展开边界；
  - archgate **MACRO-001** 针对 **testkit** 导出宏，不自动覆盖 contract-testkit 宏语义。
- **建议补丁**：
  1. 扩展 `T-CTC-010` AC：宏展开仅 `#[cfg(test)]` + compile fixture + 无 production symbols（I-CTC 新增子项）。
  2. 或新增 `T-CTC-018` compile_fail：生产路径引用宏生成符号失败。

### F7-4（次要）— branch coverage 与 §16.1 命令字面

- **规范引用**：§13.7 `branch coverage >= 90%`；§16.1 仅 `--fail-under-lines 95`。
- **缺失**：plan §6.3 复制 §16.1 字面，**未补 branch 强制方式**（llvm-cov 额外 flag 或分步检查）；与 Round 10 T-GATE-008 相关。
- **建议补丁**：在 I-CI-CORE 注明「line 命令 + branch 验收步骤」；不要求改规范 §16.1 字面。

## 本轮结论：FAIL

## fail_count: 3

> 主 FAIL：F7-1、F7-2、F7-3。F7-4 次要。  
> GRAPH/Archgate 规则 ID 表本身 **PASS**；缺口在 CI 接线与 §14.4 宏守卫任务化。
