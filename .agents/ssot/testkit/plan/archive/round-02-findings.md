# Round 2 — §4–§6 目录 / 依赖合同 / crate 规则

> Verifier: 只读计划完备性检查（非实现验收）  
> Source Spec: `.agent/SSOT/testkit/testkit-complete-spec.md`  
> Plan pack: `plan.md` · `tasks.md` · `gap-matrix.md` · `spec-inventory.md` · `residual-open.md` · `approval-packet.md` · `.worktree/testkit-todo.md`  
> 日期: 2026-07-14

## 检查项

| 规范 | 要点 | plan | tasks | inventory / residual / gap | 判定 |
|------|------|------|-------|----------------------------|------|
| §4.1 核心目录正例 | `Cargo.toml/README/AGENTS/CHANGELOG`；`src/{lib,clock}.rs`；`tests/{manual_clock_contract,manual_clock_concurrency,compile_fail,production_graph_guard}.rs` | plan §4 仅 clock 模型；未列完整树 | T-CLK-001 clock.rs；T-CLK-014…017 测试未钉文件名；T-GATE-005 有 production_graph_guard；T-ARCH-008…010 文档 | **I-DIR 仅禁模块，无正例树** | **FAIL** |
| §4.1 禁止新增模块 | util/common/prelude/mock/fixture/provider/integration/docker.rs | Forbidden 相关 | 无独立 guard 任务（可并 archgate PLACEHOLDER/API） | I-DIR 列表完整 | PASS（inventory 有；机控任务弱） |
| §4.1 新模块准入 | 满足准入八问 + RFC | plan §0.2 八问 | **无**「新模块必须 RFC」任务 | I-DONE-24.6 有 RFC·ADR 总勾；**无 §4.1 绑定** | **FAIL** |
| §4.2 contract 目录正例 | `crates/test-support/contracts/{Cargo,README,AGENTS,src/*.rs,tests/suite_self_tests.rs,compile_fail.rs}` | plan §5 路径+deps | T-CTC-001 建包；suite 模块 T-CTC-004…009；README T-CTC-017；**无 suite_self_tests.rs / compile_fail.rs 文件级 AC** | I-CTC 按 suite 名；**无 I-DIR-CTC 正例树** | **FAIL** |
| §5.1 生产 dep 仅 kernel | 白名单到此为止 | plan §0.3 | T-ARCH / 维持；T-GATE-003 DEP | I-2.1 | PASS |
| §5.1 禁止 dep 列表 | contracts/canonical/decimalx/evidence/observex/configx/tokio/futures/serde/serde_json/rand/proptest/mockall/rstest/tracing/anyhow | — | archgate DEP | I-2.2（含 serde_json? 写 serde；**缺 serde_json 显式**） | PASS（弱：serde_json 可归入 serde 族，建议显式） |
| §5.1 测试 dep 允许 | proptest / static_assertions / trybuild / loom | plan CI 有 mutants/miri | T-CLK-015 proptest；T-CLK-017 static_assertions/trybuild；**loom 无任务**（规范为「可使用」非必须） | I-2.3 | PASS（loom 可选） |
| §5.2 features | `default=[]`；core 无 feature；禁 mock/async/tokio/snapshot/serde/real/integration | — | T-GATE-003 FEATURE-001 | I-2.4 | PASS |
| §5.3 消费者规则 | 仅 `[dev-dependencies]`；禁 `[dependencies]`/`[build-dependencies]`；同适用于 contract-testkit / fixture support / integration libs | Forbidden #4；plan §0.6 | T-GATE-001…006；T-FREEZE-001 | I-2.5；I-GRAPH-1…5 | PASS |
| §5.4 生产图隔离 | testkit/contract-testkit ∉ production graph；不靠 feature resolver 偶然性；示例 cargo tree | plan §6.1 | T-GATE-001…005 | I-GRAPH；I-CI-PROD | PASS |
| §6 crate 属性 | `forbid(unsafe_code)`；`deny(missing_docs)`；`deny(unreachable_pub)` | — | T-CLK-019 | I-3.1–3.3 | PASS |
| §6 禁止项全表 | unsafe/todo!/unimplemented!/占位 public API/生产 panic!/静默回绕/真实 sleep/SystemTime·Instant/env/网络文件 IO/全局 mut/隐式 runtime/隐式 tracing | Forbidden #8 等 | T-CLK-018 时间/sleep；其余靠 I-3.4 + archgate | I-3.4 基本齐全 | PASS |
| §6 测试 panic 允许 | 测试断言可用 panic | **未写** | 无 | 无 | PASS（解释性条款，不阻塞合同） |

## PASS

- 依赖白名单/黑名单（§5.1–5.2）在 `I-2.*` 与门禁任务有映射。
- 消费者仅 dev-dep、生产图隔离（§5.3–5.4）有 GRAPH/GATE 波次与 Forbidden。
- Crate 级 `forbid`/`deny` 与禁止行为清单在 `I-3.*` + `T-CLK-019`/`T-CLK-018`。
- 禁止新增杂项模块名在 `I-DIR` 列出。
- `clock.rs` 拆分、production_graph_guard、README/AGENTS/CHANGELOG 均有任务钩子。

## FAIL

1. **§4.1 Core 目录正例树未入库**  
   - **规范引用**: §4.1 要求 `tests/manual_clock_contract.rs`、`manual_clock_concurrency.rs`、`compile_fail.rs`、`production_graph_guard.rs` 及 `src/clock.rs` 等固定布局  
   - **缺失内容**: inventory 只有「禁止模块」`I-DIR`，**没有**正例目录/测试文件名 ID；tasks 中并发/单测/compile_fail 未绑定规范文件名（仅 T-GATE-005 点名 production_graph_guard）。实现可把测试全堆 `lib.rs` cfg(test) 仍勾 DONE。  
   - **建议补丁位置**: `spec-inventory.md` 增 `I-DIR-CORE` 正例树（逐文件）；`tasks.md` `T-CLK-001/014/016/017` AC 写明目标路径；`gap-matrix.md` §4.1 关闭条件引用 `I-DIR-CORE`。

2. **§4.1 新模块准入八问 + RFC 无任务**  
   - **规范引用**: §4.1「新增模块必须满足准入八问并通过 RFC」  
   - **缺失内容**: plan 有八问正文，但无 I-*、无 Task、无门禁「禁止无 RFC 的新 src 模块」；与 §12 新增公开项 RFC 未在本轮目录合同中交叉引用。  
   - **建议补丁位置**: `spec-inventory.md` 增 `I-DIR-RFC`；`tasks.md` W6 或 W0 增 `T-FREEZE-00x`「archgate/文档：新模块无 RFC 则 fail」；`plan.md` Forbidden 可写「无 RFC 新增 testkit 模块」。

3. **§4.2 contract-testkit 测试目录与自测文件未映射**  
   - **规范引用**: §4.2 `tests/suite_self_tests.rs`、`tests/compile_fail.rs`；src 下按 trait 分文件  
   - **缺失内容**: T-CTC-011/012 有 reference/broken 自测语义，但**未**要求 `suite_self_tests.rs` / `compile_fail.rs` 布局；inventory 无 contract 包目录正例。薄宏/隐藏依赖有任务，目录合同不完整。  
   - **建议补丁位置**: `spec-inventory.md` 增 `I-DIR-CTC` 正例树；`tasks.md` `T-CTC-001` AC 列目录；新增 `T-CTC-018` compile_fail（contract-testkit 不错误依赖 adapter 等）；`T-CTC-011` 绑定 `suite_self_tests.rs`。

## 本轮结论：FAIL

## fail_count: 3
