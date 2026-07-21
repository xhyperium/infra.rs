> **Post-ship (2026-07-14)**：战役 **COMPLETE**。API-002 已机控；mutants missed=0；crates.io **xhyper-kernel 0.1.1**。live SSOT：`residual-open.txt` + `gate.md` + `release.md`。

# Plan — SPEC-KERNEL-002 完整执行计划（v2）

| 字段            | 值                                                                                |
| --------------- | --------------------------------------------------------------------------------- |
| Plan ID         | `PLAN-KERNEL-002-v2-complete`                                                     |
| Source Spec     | `.agents/ssot/kernel/spec/xhyper-kernel-complete-spec.md`（≡ `spec/spec.md`）     |
| Spec ID         | `SPEC-KERNEL-002` · Status **Approved**                                           |
| Goal            | `GOAL-KERNEL-RUNTIME-SEMANTICS`                                                   |
| Package         | `kernel` @ `crates/kernel` · version **0.1.1**                                    |
| Branch          | `feat/kernel-002-e2-migrate-banned-apis`                                          |
| Ship PR         | [#235](https://github.com/xhyperium/infra.rs/pull/235)                           |
| Residual SSOT   | [evidence/2026-07-14/residual-open.txt](../evidence/2026-07-14/residual-open.txt) |
| Work Todo       | [`.worktrees/kernel-todo.md`](../../../../.worktrees/kernel-todo.md)                |
| Strategy        | **诚实台账 → 差距闭合 → 强化测试/门禁 → 十轮验收 → 人审 Approved → release**      |
| Campaign status | **COMPLETE** · Spec Approved · 0.1.1 · stable · API-002 · crates.io `xhyper-kernel` |
| Forbidden       | 假 §18 Done / registry `stable` / 无 Evidence 勾 PASS                             |

---

## 0. 深度分析结论（对照完整规范 §0–§19）

### 0.1 Kernel 是什么

`kernel` 是 workspace **语义信任根**，只承载三类全系统唯一语义：

1. **error** — 按反应分类，不按来源分类
2. **clock** — 墙钟/单调钟分离，失败显式，无隐式全局钟
3. **lifecycle** — 状态语言 + 一次触发多方观察的关停原语

任何不满足「准入四问」的能力 **永久禁止** 进入 kernel（§1.2 非目标清单）。

### 0.2 章节级现状矩阵（以源码 + 本仓 CI/tests + residual 为准）

| 章  | 主题          | 代码/机器        | 文档    | Residual / 备注                                                          |
| --- | ------------- | ---------------- | ------- | ------------------------------------------------------------------------ |
| §0  | 文档定位      | —                | PASS    | SSOT=002；Status **Approved**                                            |
| §1  | 设计原则      | PASS             | PASS    | README/AGENTS 准入四问                                                   |
| §2  | 目录结构      | PASS             | PASS    | `src/{error,clock,lifecycle}` + 5 tests                                  |
| §3  | 依赖合同      | **PASS**         | PASS    | 生产仅 thiserror；`default=[]`；loom 仅 `cfg(loom)`                      |
| §4  | Crate 属性    | **PASS**         | PASS    | forbid unsafe + deny missing_docs/unreachable_pub                        |
| §5  | error         | **PASS**         | PASS    | 无 `context_cow`（RES-ERR-010 CLOSED）                                   |
| §6  | clock         | **PASS**         | PASS    | origin+elapsed；unix nanos；`pub const fn from_clock_elapsed`            |
| §7  | lifecycle     | **PASS**         | PASS    | Mutex+Condvar；loom+CI；LC-005 补测 CLOSED                               |
| §8  | 公开 API 冻结 | **PASS**         | PASS    | 超面已删；API-001 快照对齐                                               |
| §9  | Serde 政策    | **PASS**         | PASS    | 无 serde                                                                 |
| §10 | Panic 策略    | **PASS**         | PASS    | poison→into_inner                                                        |
| §11 | 测试合同      | **PARTIAL**      | PARTIAL | unit/proptest/loom/LC-005 PASS；trybuild DEFER；mutants/miri/branch OPEN |
| §12 | CI / 机控     | **PASS**  | PASS  | 本仓：结构扫描 / tests / CI；archgate **OOS**；API-002 语义 **implemented**（RES-GATE-009） |
| §13 | 性能预算      | PARTIAL          | PASS    | `Cow::Owned`；RES-PERF-001 **CLOSED (DEFER accepted)**                   |
| §14 | 文档          | **PASS**         | PASS    | README/AGENTS/CHANGELOG 对齐 002                                         |
| §15 | 版本          | **PASS**         | PASS    | version **0.1.1**；registry **stable**                                   |
| §16 | 迁移          | **DONE**         | PASS    | E1–E3 / C1–C2 / L1–L2 已执行                                             |
| §17 | Evidence      | PARTIAL accepted | PASS\*  | campaign 包可追溯；\*RES-EVID-001 CLOSED（partial-accepted）             |
| §18 | 完成定义      | **PASS**         | PASS    | 全勾 CLOSED（18.3 末三项 waiver DEFER）                                  |
| §19 | 最终裁定      | 治理约束         | PASS    | 持续适用                                                                 |

### 0.3 residual 清单（与 residual-open 一致 · post W0–W5）

#### A. 已 CLOSED（不得重开，除非回归）

| ID                                                 | 含义                                                                                     |
| -------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| RES-ERR-001…007,009,**010**                        | opaque XError / 9 kinds / 禁 not_found·other / From\<ClockError\> / **context_cow 已删** |
| RES-CLK-001…009,**010**                            | no default mono / unix nanos / reverse None / origin / from_std 删除 / **const fn**      |
| RES-LC-001…004,**005**                             | Mutex 协议 / must_use / loom+CI / **poison+1000+guard drop+!Clone**                      |
| RES-API-001…009                            | 无 Component / features / docs attrs / publish=true as xhyper-kernel                                     |
| RES-TEST-001…004,006…013；**005 (DEFER accepted)** | 测试文件 + proptest + loom CI + 边界；trybuild 正式 defer                                |
| RES-GATE-001…008；**009 (DEFER accepted)**         | API 快照 + 13 KERNEL-\* + CI loom；API-002 正式 defer                                    |
| RES-DOWN-001…**006**                               | 下游 + sleep 审计（DOWN-006 CLOSED）                                                     |
| RES-DOC-001                                        | 台账与 residual-open 对齐                                                                |
| RES-PERF-001                                       | CLOSED (DEFER) · Cow::Borrowed                                                           |
| RES-EVID-001                                       | CLOSED (partial-accepted) · §17 inventory                                                |

#### B. 仍 OPEN（honest · residual-open SSOT）

**无。**

#### C. 本轮曾登记现已 CLOSED（勿再列 OPEN）

| ID                   | 终态                  | 关闭说明                                                         |
| -------------------- | --------------------- | ---------------------------------------------------------------- |
| RES-ERR-010          | CLOSED                | redisx → `context()`；删 `context_cow`；API 快照已去行           |
| RES-CLK-010          | CLOSED                | `pub const fn from_clock_elapsed`                                |
| RES-LC-005           | CLOSED                | poison_recovery + 1000 cycles + guard drop + assert_not_impl_any |
| RES-TEST-005         | CLOSED (DEFER)        | trybuild 正式 defer；static_assertions 覆盖核心禁止面            |
| RES-GATE-009         | CLOSED (implemented)  | KERNEL-API-002 baseline+RFC allowlist 机控                       |
| RES-DOC-001          | CLOSED                | Team-R10 / R10c doc-sync                                         |
| RES-DOWN-006         | CLOSED                | 全树 sleep 审计；Evidence DOWN-006                               |
| RES-PERF-001         | CLOSED (DEFER)        | Cow::Borrowed 正式 defer；Evidence PERF-001                      |
| RES-EVID-001         | CLOSED (partial)      | §17 inventory；campaign partial 可接受                           |
| RES-18-APPROVED      | CLOSED                | Spec Status Approved；人审授权 2026-07-14                        |
| RES-API-007          | CLOSED                | version 0.1.1                                                    |
| RES-TEST-014/015/016 | CLOSED (waiver DEFER) | §18 人审豁免                                                     |
| RES-18-FULL          | CLOSED                | §18 全勾 + registry stable                                       |

#### D. §18 勾选真相（不得粉饰）

| 节   | 项                                | 状态                                                        |
| ---- | --------------------------------- | ----------------------------------------------------------- |
| 18.1 | 本文件 Approved                   | **PASS**（RES-18-APPROVED CLOSED）                          |
| 18.1 | 旧 spec superseded                | PASS                                                        |
| 18.1 | README/AGENTS/CHANGELOG           | PASS                                                        |
| 18.1 | registry 一致（**stable**）       | PASS                                                        |
| 18.1 | 无未登记 Unknown                  | PASS                                                        |
| 18.2 | 代码主路径 9 项                   | **PASS**                                                    |
| 18.3 | unit / property / loom            | PASS                                                        |
| 18.3 | compile-fail（trybuild）          | **CLOSED (DEFER)** RES-TEST-005                             |
| 18.3 | line ≥95%                         | PASS（98.82%）                                              |
| 18.3 | branch ≥90% / mutants / miri      | **CLOSED (measured PASS)** RES-TEST-014/015/016             |
| 18.4 | API diff / 本仓 CI 机控 / lint-deps | PASS 机器轨（archgate **OOS**）                            |
| 18.4 | KERNEL-API-002                    | **CLOSED (implemented)** RES-GATE-009                       |
| 18.4 | cargo-deny / internal 棘轮 / 下游 | PASS / 棘轮=8                                               |
| 18.4 | Evidence → 当前 commit            | **CLOSED (partial-accepted)** RES-EVID-001；不声称 §17 全树 |

---

## 1. 执行策略与原则

```text
1. 证据优先：任何 PASS 必须绑定命令输出或 evidence 文件
2. 外科手术：只改 kernel 闭合所需路径；不顺手重构 workspace
3. 单 writer：同一文件路径不并行写
4. residual 纪律：OPEN/CLOSED only；禁止 Unknown
5. 禁止：registry stable、§18 全勾、手写 PASS 代替命令
6. 人审闸门：Spec Approved / version bump 策略 不由 AI 独断关闭
7. 十轮验收：每轮独立检查清单，fail_rounds 必须为 0 才可宣称「十轮通过」
```

---

## 2. 波次（Waves）与依赖 DAG

```text
W0  台账诚实（文档同步）          ──┐
W1  API 微缺口（ERR-010/CLK-010）  ──┼──→ W3 测试补强（LC-005/TEST-005）
W2  门禁补强（GATE-009 可选）     ──┤
W4  质量证据（014/015/016）       ──┤
W5  十轮验收 ×10                  ──┼──→ W6 人审包（Approved 决策包）
W7  version 0.1.1（策略允许时）   ──┘
W8  §18 闭合声明（仅人审后）
```

| Wave   | 名称                     | 可并行               | Owner         | 退出条件                                                           |
| ------ | ------------------------ | -------------------- | ------------- | ------------------------------------------------------------------ |
| **W0** | 文档/台账同步            | 是（只写 docs）      | Doc Agent     | gate/plan/tasks/matrix/review 与 residual 一致；RES-DOC-001 CLOSED |
| **W1** | 公开 API 微缺口          | 否（动 kernel+下游） | Code Agent    | RES-ERR-010 决策落地；RES-CLK-010 可选                             |
| **W2** | KERNEL-API-002 机控（历史 monorepo archgate） | 是 | Gate Agent | RES-GATE-009 CLOSED 或正式 DEFER；**infra.rs 不要求 archgate** |
| **W3** | 测试补强                 | 是（与 W2）          | Test Agent    | RES-LC-005 CLOSED；RES-TEST-005 决策                               |
| **W4** | branch/mutants/miri 证据 | 是（耗时长）         | Quality Agent | RES-TEST-014/015/016 有 PASS 或 DEFER+理由                         |
| **W5** | 十轮全量检查             | 串行 10 轮           | Verify Team   | fail_rounds=0；见 §4                                               |
| **W6** | 人审 Approved 包         | 人                   | Human         | Spec `Status: Approved`                                            |
| **W7** | version bump 0.1.1       | 策略后               | Release       | RES-API-007 CLOSED                                                 |
| **W8** | §18 全勾                 | 人+机                | Owner         | 全部 18.1–18.4；registry 仍须单独决策 stable                       |

### 2.1 本会话 AI 默认可执行范围

| 可立即执行                                    | 需人决策 / 外部                          |
| --------------------------------------------- | ---------------------------------------- |
| W0 文档同步                                   | W6 Spec Approved                         |
| W1a 下游 `context_cow`→`context()` + 删除超面 | W7 是否现在 bump 版本                    |
| W1b `const fn from_clock_elapsed`             | W8 stable                                |
| W3 lifecycle 补测                             | RES-PERF-001 已 DEFER（勿半破坏 API）    |
| W2 最小 KERNEL-API-002 或 defer 文档          | RES-TEST-005 trybuild 若与解析冲突则 ADR |
| W4 miri/mutants/branch（环境允许时）          |                                          |
| W5 十轮检查                                   |                                          |

---

## 3. 任务拆解（可指派 agent team）

### Team 角色

| Role                 | 职责                            | 写入路径                                             |
| -------------------- | ------------------------------- | ---------------------------------------------------- |
| **Planner**          | 本 plan + residual 登记         | `.agents/ssot/kernel/plan/*`、`residual-open.txt`    |
| **Executor-API**     | ERR-010 / CLK-010               | `crates/kernel`、`crates/adapters/storage/redis`     |
| **Executor-Test**    | LC-005 / 可选 trybuild          | `crates/kernel/tests/*`、`src/*` tests               |
| **Executor-Gate**    | GATE-009 / 文档 defer           | 历史 `tools/archgate`（**OOS**）或 evidence           |
| **Executor-Quality** | 014/015/016 命令+证据           | `evidence/2026-07-14/*`                              |
| **Doc-Sync**         | gate/tasks/matrix/review/goal   | `.agents/ssot/kernel/**`、`.worktrees/kernel-todo.md` |
| **Verifier**         | 十轮清单 + 汇总 verdict         | `EVID-KERNEL-002-R10b-*.md`                          |
| **Skeptic**          | 反证：找假 PASS / 遗漏 residual | 只读 + 写 review 批注                                |

### 3.1 原子任务表

| Task ID       | Wave | 内容                                                           | AC                              | Status                                  |
| ------------- | ---- | -------------------------------------------------------------- | ------------------------------- | --------------------------------------- |
| T-PLAN-001    | W0   | 本 plan v2 落盘                                                | 文件存在且含 §0–§6              | **DONE**                                |
| T-TODO-001    | W0   | `.worktrees/kernel-todo.md`                                     | 全 OPEN 项可追踪                | **DONE**                                |
| T-DOC-001     | W0   | 刷新 gate.md / tasks / matrix / review / goal 与 residual 一致 | 无矛盾 FAIL 陈述                | **DONE**（Team-R10 再同步）             |
| T-RES-001     | W0   | residual-open 登记 C 类 ID                                     | 全部 OPEN/CLOSED                | **DONE**                                |
| T-API-001     | W1   | 删除或冻结 `context_cow`                                       | redisx 迁 `context()`；快照更新 | **DONE** · RES-ERR-010 CLOSED           |
| T-API-002     | W1   | `from_clock_elapsed` → `const fn`                              | 编译+测试绿                     | **DONE** · RES-CLK-010 CLOSED           |
| T-TEST-001    | W3   | poison recovery 单测                                           | assert 中毒后状态可恢复         | **DONE** · RES-LC-005                   |
| T-TEST-002    | W3   | 1000 次并发回归                                                | 无 hang / 全观察 trigger        | **DONE** · RES-LC-005                   |
| T-TEST-003    | W3   | guard drop 不触发                                              | drop 后 is_triggered=false      | **DONE** · RES-LC-005                   |
| T-TEST-004    | W3   | `assert_not_impl_any!(ShutdownGuard: Clone)` 等                | api_compile 扩展                | **DONE** · RES-LC-005                   |
| T-TEST-005    | W3   | trybuild 或 DEFER 记录                                         | RES-TEST-005 有终态             | **DONE** · CLOSED (DEFER)               |
| T-GATE-001    | W2   | KERNEL-API-002 实现或 DEFER                                    | residual 终态                   | **DONE** · CLOSED (implemented)         |
| T-Q-001       | W4   | branch cov 报告                                                | RES-TEST-014 终态               | **DONE**（证据；residual **OPEN**）     |
| T-Q-002       | W4   | mutants                                                        | RES-TEST-015 终态               | **DONE**（证据；residual **OPEN**）     |
| T-Q-003       | W4   | miri                                                           | RES-TEST-016 终态               | **DONE**（证据；residual **OPEN**）     |
| T-VER-001…010 | W5   | 十轮检查                                                       | fail=0                          | **DONE** · R10b fail_rounds=0 · L1 PASS |
| T-HUM-001     | W6   | Spec Approved 决策包 + 人审授权落盘                            | Status → Approved               | **DONE** · RES-18-APPROVED **CLOSED**   |
| T-REL-001     | W7   | version 0.1.1                                                  | **默认 DEFER** 至人审策略       | **BLOCKED** · RES-API-007 OPEN          |

---

## 4. 十轮验收协议（W5 · 不可省略）

每轮 **独立** 执行下列检查；任一项 FAIL → 记入 round-log 并修复后重跑该轮。

| #   | Check ID | 命令 / 动作                                                                     | 期望              |
| --- | -------- | ------------------------------------------------------------------------------- | ----------------- |
| 1   | C-FMT    | `cargo fmt -p kernel -- --check`                                                | exit 0            |
| 2   | C-CLIPPY | `cargo clippy -p kernel --all-targets -- -D warnings`                           | exit 0            |
| 3   | C-TEST   | `cargo test -p kernel`                                                          | 全绿              |
| 4   | C-LOOM   | `RUSTFLAGS='--cfg loom' cargo test -p kernel --test lifecycle_concurrency_loom` | ≥1 passed         |
| 5   | C-ARCH   | ~~`cargo run -p archgate -- --json`~~ → **infra.rs 不适用（OOS）**；改用结构扫描 / unit tests / CI（coverage, loom, miri, public-api） | 本仓机控绿 |
| 6   | C-DEPS   | `cargo xtl lint-deps`                                                           | exit 0            |
| 7   | C-API    | 快照与 `cargo public-api -p kernel` 无意外 diff                                 | 一致或已提交      |
| 8   | C-SSOT   | residual-open 无 Unknown；gate/plan/todo 无矛盾                                 | 一致              |
| 9   | C-BAN    | 无 `not_found`/`other` 构造；无 Component 导出；无 default mono                 | rg 空             |
| 10  | C-§18    | 未宣称 stable/§18 全闭合                                                        | 文档诚实          |

**轮次规则**

```text
for r in 1..10:
  run all C-*
  if any FAIL: fix → re-run same r until PASS
  append EVID round-log
assert fail_rounds == 0
write R10b verdict (honest §18 still OPEN if Approved missing)
```

---

## 5. 验收与交付物

### 5.1 本战役交付

| 产物          | 路径                                                     |
| ------------- | -------------------------------------------------------- |
| 完整计划 v2   | `.agents/ssot/kernel/plan/plan.md`（本文件）             |
| 章节差距附录  | `.agents/ssot/kernel/plan/gap-matrix-v2.md`              |
| 工作台账      | `.worktrees/kernel-todo.md`                               |
| residual 更新 | `evidence/2026-07-14/residual-open.txt`                  |
| 十轮日志      | `evidence/2026-07-14/EVID-KERNEL-002-R10b-round-log.txt` |
| 十轮裁定      | `evidence/2026-07-14/EVID-KERNEL-002-R10b-verdict.md`    |
| 人审包        | `.agents/ssot/kernel/plan/approval-packet.md`            |

### 5.2 成功定义（分层）

| 层级            | 含义                                                      |
| --------------- | --------------------------------------------------------- |
| **L1 战役成功** | 可执行 residual 关闭或正式 DEFER；十轮 fail=0；文档无漂移 |
| **L2 机器闭合** | §18.2 + 可机器化的 18.3/18.4 全绿                         |
| **L3 目标闭合** | L2 + Spec Approved + version 策略 + 可选 stable           |

**本 plan 默认冲 L1；L2 尽力；L3 禁止 AI 独自宣布。**

---

## 6. 风险与回滚

| 风险                           | 缓解                                 |
| ------------------------------ | ------------------------------------ |
| 删 `context_cow` 破下游        | 先 rg 全树迁移再删；更新 API 快照    |
| trybuild 与 workspace 解析冲突 | 保留 static_assertions；写 DEFER ADR |
| mutants/miri 环境缺失          | 记录 SKIP≠PASS；OPEN 保留            |
| 误标 §18/stable                | Skeptic 轮次 + 禁止词检查            |
| 分支脏文件（非 kernel）        | 不纳入本战役 commit                  |

回滚：`git checkout -- crates/kernel .agents/ssot/kernel`（仅本战役路径；历史 monorepo 含 `tools/archgate`，infra.rs 不维护）。

---

## 7. 与 v1 plan 关系

| v1                        | v2                              |
| ------------------------- | ------------------------------- |
| Phases D/E/C/L/G **DONE** | 继承为基线，不重做              |
| §18 OPEN                  | 仍 OPEN；拆解为 W6–W8           |
| residual 5 OPEN           | 扩展 C 类新 residual + 关闭路径 |
| 无十轮协议细节            | §4 完整十轮清单                 |

---

## 8. 战役进度与下一步

| 层级         | 状态                                         |
| ------------ | -------------------------------------------- |
| **L1 战役**  | **PASS**                                     |
| **L2 / §18** | **CLOSED**（014/015/016 human waiver DEFER） |
| **L3**       | **PASS**（Approved + 0.1.1 + stable）        |

**下一步（可选）：**

1. ~~Approved / 0.1.1 / §18 / stable~~ → **DONE**（EVID-KERNEL-002-18-RELEASE）
2. 可选：补跑 nightly branch / mutants / miri（豁免→实测）
3. 可选：`git tag kernel-v0.1.1`

**OPEN 仅余：无**（API-002 + crates.io 已闭合）

---

_Generated from deep analysis of `xhyper-kernel-complete-spec.md` + live tree @ branch `feat/kernel-002-e2-migrate-banned-apis`. Residual SSOT: residual-open.txt（Team-R10c）。_
