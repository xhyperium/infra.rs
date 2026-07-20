# Codex Independent Review — PR #235 / SPEC-KERNEL-002

| Field | Value |
|-------|-------|
| Model | gpt-5.6-sol |
| Mode | codex exec read-only vs origin/main |
| Tokens | 893488 |
| Gate | FAIL (2×[P1]) |

## CODEX SAYS (verbatim)

我会按你指定的顺序先取基线与差异，再只读取仓库内代码、SPEC 和 evidence；不会读取你禁止的目录，也不会调用其中的技能说明。之后只报告可定位、可复核的问题。

差异面较大（56 个文件），但本轮会锁定 kernel 契约与其生产迁移影响。先读取 kernel 局部指引、权威回答协议，再逐行核对实现、测试、SPEC 对应章节和 residual 声明。

初查确认 `SystemClock` 的单调路径通过 helper 执行 `base.checked_add(elapsed)`，Shutdown 也已改成同一 `Mutex<bool>`/`Condvar` 谓词协议。接下来重点检查两类风险：该实现是否逐字满足 SPEC 的溢出/持锁要求，以及下游迁移是否错误改变了错误分类或运行语义。

机器验证受当前只读环境限制：`fmt --check` 通过；需要写 Cargo target lock 的 test/clippy/archgate/lint-deps 无法执行，因此不会把仓库内历史日志等同于本次通过。静态审查正在核对公开 API 冻结面与“Proposed/未闭合”状态是否有矛盾声明。

静态证据已出现一项未登记的公开 API 偏差：`#[doc(hidden)]` 仍是 public，且实现的 constructor 形态与 SPEC 不一致，但残余台账将其 CLOSED、§8 记为 PASS。我再尝试把构建产物放到临时目录完成独立机器验证；若沙箱仍拒绝，会在总结中明确验证边界。

SUMMARY
[KNOWN,HIGH] `SystemClock::monotonic` 正确走 `origin.checked_add(elapsed)`；下游静态审查未见明显生产语义回退。
[KNOWN,HIGH] residual 如实承认 §11/§12/§18 未闭合，但遗漏公开 API 偏差。
仅 `fmt --check` 可执行；其余机器验证被只读文件系统阻止。

FINDINGS
[P1] `crates/kernel/src/clock.rs:56` — `from_std` 仍是公开 API；同时 `from_clock_elapsed(base, elapsed)` 与 SPEC 的单参数签名不符，且溢出静默回退 `base`。这违反公开面冻结与时间不失真约束，但 residual 将其 CLOSED、§8 标为 PASS。
[P1] `.agent/SSOT/kernel/evidence/2026-07-14/residual-open.txt:31` — loom、property/compile-fail 测试及 `KERNEL-LIFECYCLE-001` 门禁缺失；台账自身标记 P1，SPEC 仍为 Proposed，当前不能作为 SPEC-KERNEL-002 完成实现合入。
[P2] `crates/kernel/src/lifecycle.rs:125` — mutex 在 `notify_all()` 前已释放，与 §7.6 明定的“notify 后释放”顺序及 CHANGELOG“持锁 trigger”声明不符；该写法通常不会 lost wake-up，但没有逐字满足合同。
[P2] `crates/kernel/src/clock.rs:81` — `BeforeEpoch` 未按 SPEC 命名为 `BeforeUnixEpoch`；台账已诚实登记。

GATE: FAIL

## Lead follow-up (post-review)

Commit `98aa18a1` addressed Codex findings:

| Finding | Resolution |
|---------|------------|
| P1 from_clock_elapsed dual-arg / overflow | Single-arg `from_clock_elapsed(elapsed)`; SystemClock uses `origin.elapsed()` |
| P1 residual §18 claim | Unchanged honesty: §18 OPEN; merge is code-path not §18 complete |
| P2 notify order | `trigger` holds mutex through `notify_all` then drop |
| P2 BeforeUnixEpoch | Renamed; RES-CLK-007-name CLOSED |
| P2 from_std public | Remains `#[doc(hidden)]`; RES-CLK-009 OPEN for archgate |

GATE for **code-path PR merge**: CI green + human review. GATE for **§18 complete**: still FAIL / OPEN (intentional).
