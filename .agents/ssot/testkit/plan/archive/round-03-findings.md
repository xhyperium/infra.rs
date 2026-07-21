# Round 3 — §7 ManualClock 完整合同（逐 API）

> Verifier: 只读计划完备性检查（非实现验收）  
> Source Spec: `.agent/SSOT/testkit/testkit-complete-spec.md`  
> Plan pack: `plan.md` · `tasks.md` · `gap-matrix.md` · `spec-inventory.md` · `residual-open.md` · `approval-packet.md` · `.worktree/testkit-todo.md`  
> 日期: 2026-07-14  
> 说明: 逐 API 对照；**不**宣称实现完成。

## 检查项

| 规范 API / 规则 | 要点 | plan §4 / I-CLK / T-CLK | 判定 |
|-----------------|------|-------------------------|------|
| §7.1 目标 | 独立墙钟/单调钟；advance；rewind wall；mono 只进；fault；snapshot；多线程；无真实时间；无静默溢出 | plan §4；I-CLK-*；T-CLK-002…018 | PASS |
| §7.2 模型 | `Mutex<State{wall, monotonic_elapsed, wall_fault}>` | I-CLK-MODEL；T-CLK-002 | PASS |
| §7.2 选型理由 | snapshot 一致；fault 与 wall 线性化；checked；非热路径；正确性优先 | plan 有 Mutex 理由摘要 | PASS |
| §7.2 禁多 atomic 伪一致 | 明确禁止 | Forbidden #7；plan §4 | PASS |
| §7.3 ManualClockFault | 三变体 + 映射 ClockError | I-CLK-FAULT；T-CLK-003；plan §0.7 | PASS |
| §7.3 属性 | `#[non_exhaustive]` + Debug/Clone/Copy/PartialEq/Eq | **inventory/tasks 未写 non_exhaustive 与 derive 集** | **FAIL** |
| §7.3 跟随 kernel | 不得自创平行错误语义 | plan §0.7；T-CLK-003 | PASS |
| §7.4 ManualClockError | 四变体；Display+Error；禁 anyhow | I-CLK-ERR；T-CLK-004 | PASS |
| §7.4 属性 | `#[non_exhaustive]` + Debug | **未映射 non_exhaustive** | **FAIL**（并入上条计数时合并为「枚举合同属性」1 条） |
| §7.5 Snapshot 类型 | 私有字段；const getter×3；derives | I-CLK-SNAP；T-CLK-005「私有字段；const getter」；**derives 未列** | PASS（弱） |
| §7.6 构造 | `new(Timestamp)`；`with_monotonic_elapsed`；**禁 Default**；理由 epoch 伪装 | I-CLK-NEW；T-CLK-006 | PASS |
| §7.7 set_wall | `Result<(), ManualClockError>` | I-CLK-WALL；T-CLK-007 | PASS |
| §7.7 advance_wall | `Result<Timestamp, _>`；checked_add；失败不改状态 | I-CLK-WALL「Result；checked；失败不改」；**返回 Timestamp 未写** | PASS（弱） |
| §7.7 rewind_wall | `Result<Timestamp, _>`；checked_sub；允许回退 | I-CLK-WALL；T-CLK-007 | PASS（弱） |
| §7.7 禁带符号纳秒 fetch_add | 明确禁止 | **plan/I-CLK/T-CLK 无** | **FAIL** |
| §7.7 禁 release 模式回绕 | 明确禁止 | Forbidden/静默回绕有；**「release 模式回绕」措辞未单独映射** | **FAIL** |
| §7.7 deprecated 迁移 API | `set_unix_nanos`/`advance_unix_nanos` + 删除版本 + 调用点清单 | I-CLK-DEP；T-CLK-013 | PASS |
| §7.8 set_monotonic_elapsed | `Result<(), _>`；小于当前 → MonotonicRegression；失败不改状态 | I-CLK-MONO「regression/overflow」；**失败不改状态未写在 MONO** | **FAIL** |
| §7.8 advance_monotonic | `Result<MonotonicInstant, _>`；溢出 MonotonicOverflow | I-CLK-MONO；T-CLK-008；**返回 MonotonicInstant 未写** | PASS（弱） |
| §7.8 禁 rewind / 禁 signed delta | 明确 | I-CLK-MONO「无 rewind」；**signed delta 无** | **FAIL** |
| §7.9 set/clear/wall_fault | 三者均 `Result`；语义：now 映射错误；不改 wall；clear 恢复；mono 不受影响 | I-CLK-FAULT-CTL；T-CLK-009 | PASS |
| §7.9 禁「仅失败一次」队列 | 首个稳定版不提供 | **完全无映射** | **FAIL** |
| §7.9 scripted fault sequence 准入 | 仅 ≥2 真实消费者才可新增 | **完全无映射** | **FAIL** |
| §7.10 snapshot() | `Result<ManualClockSnapshot,_>`；同锁临界区 | I-CLK-SNAP-API；T-CLK-010 | PASS |
| §7.11 Clock::now | 获锁 → 锁失败 Unavailable → fault → wall | I-CLK-TRAIT「now+fault」；T-CLK-011；**锁失败→Unavailable 未写** | **FAIL** |
| §7.11 Clock::monotonic | 可恢复锁中毒：不在持锁跑调用方代码；poison 恢复 inner；不伪造零；不 panic；文档化；不得擅自改 trait | T-CLK-011「mono poison 恢复文档」；I-CLK-TRAIT 压缩；**分项 bullet 未入库** | **FAIL** |
| §7.12 无 Clone / Arc 共享 | 明确 | I-CLK-CLONE；T-CLK-012 | PASS |
| §7.13 Send+Sync 编译断言 | 明确 | I-CLK-BOUNDS；T-CLK-012 | PASS |
| §7 签名表 | 规范给出的完整方法签名与返回类型矩阵 | inventory 为缩写表，**无签名级 SSOT** | **FAIL** |

## PASS

- 状态模型 Mutex + 三字段、Fault 三态映射、Error 四变体、Snapshot getters、构造禁 Default、墙钟 set/advance/rewind checked 方向、单调 set/advance 无 rewind、fault 控制、snapshot 同锁、无 Clone、Send+Sync、deprecated 旧 nanos API 均有 I-CLK 与 T-CLK 波次。
- DEF-002/003 与 plan §4 覆盖 V2 主路径。
- kernel Clock 对齐与禁自创错误语义在 plan §0.7。

## FAIL

1. **§7.3 / §7.4 枚举属性合同（non_exhaustive 等）**  
   - **规范引用**: §7.3 `#[non_exhaustive]` + derives；§7.4 `#[non_exhaustive]` + Debug  
   - **缺失内容**: I-CLK-FAULT / I-CLK-ERR 只列变体名与映射/Display，未要求 `non_exhaustive` 与 derive 集合；T-CLK-003/004 AC 未引用。  
   - **建议补丁位置**: `spec-inventory.md` 扩展 I-CLK-FAULT/ERR 行；`tasks.md` T-CLK-003/004 AC 增加 `non_exhaustive` + derive 检查（可用 compile assertion 或 API snapshot）。

2. **§7.7 禁带符号纳秒 fetch_add**  
   - **规范引用**: §7.7「不提供带符号纳秒 fetch_add」  
   - **缺失内容**: plan/I-CLK-WALL/T-CLK 无对应禁止项；仅有 checked/Result。  
   - **建议补丁位置**: I-CLK-WALL 增 bullet；T-CLK-007 AC；API snapshot 禁 `i64` 控制 API（除 deprecated 清单）。

3. **§7.7 禁 release 模式算术回绕**  
   - **规范引用**: §7.7「不允许 release 模式回绕」  
   - **缺失内容**: 通用「静默回绕」有，但未明确 debug/release 均不得 wrapping；无 mutation  survivorship 在本轮 inventory 绑定到该句（§13.6 有 wrapping 禁存活，属后轮，但 §7 合同句本身应在 I-CLK）。  
   - **建议补丁位置**: I-CLK-WALL 明确；交叉引用 I-TEST-MUT。

4. **§7.8 单调路径：失败不改状态 + 禁 signed delta**  
   - **规范引用**: §7.8「失败不修改状态」「不接受 signed delta」  
   - **缺失内容**: I-CLK-MONO 仅「无 rewind；regression/overflow」；失败不变写在 I-CLK-WALL 未复制到 mono；signed delta 全无。  
   - **建议补丁位置**: 扩展 I-CLK-MONO；T-CLK-008 AC；T-CLK-014 单测矩阵显式包含 mono 失败不变。

5. **§7.9 禁「仅失败一次」队列 + scripted fault 准入**  
   - **规范引用**: §7.9 末两段  
   - **缺失内容**: inventory/tasks/plan **完全无** 对应 I-* 或 Forbidden；实现者可能「顺手」加 one-shot fault 队列。  
   - **建议补丁位置**: 增 `I-CLK-FAULT-SCOPE`；plan Forbidden；T-CLK-009 AC「无 one-shot/scripted queue」；scripted 需求走 RFC + 两消费者证明（I-ADMIT）。

6. **§7.11 now() 锁失败 → Unavailable**  
   - **规范引用**: §7.11 `now()`：获锁 → 锁失败映射 Unavailable → wall_fault → wall  
   - **缺失内容**: I-CLK-TRAIT / T-CLK-011 只强调 fault 映射与 mono poison 文档，**未**写 now 路径的 Synchronization/Unavailable 映射。  
   - **建议补丁位置**: 扩展 I-CLK-TRAIT 为分步伪代码；T-CLK-011 AC 逐条。

7. **§7.11 monotonic() 锁中毒恢复分项**  
   - **规范引用**: §7.11 五条：不在持锁执行调用方代码；poison 恢复 inner；不伪造零；不 panic；文档明确；且不得为报错擅自改 `Clock::monotonic` 合同  
   - **缺失内容**: 压缩为「poison 恢复文档」一词；无 I-* 分项；「不伪造零 / 不 panic / 不改 trait」无 AC。  
   - **建议补丁位置**: `I-CLK-TRAIT-MONO-1…5`；T-CLK-011/014；plan §0.7 交叉引用「不得改 kernel trait」。

8. **§7 方法签名与返回类型矩阵缺失**  
   - **规范引用**: §7.6–7.11 完整 `fn` 签名（含 `Result<Timestamp,_>`、`Result<MonotonicInstant,_>`、fault 三 API 均 Result 等）  
   - **缺失内容**: inventory 为语义缩写，tasks AC 多「见 I-CLK-*」但 I-CLK 本身不完整；无「签名级」验收表，易在实现时改返回类型而不触发计划 FAIL。  
   - **建议补丁位置**: `spec-inventory.md` 增 `I-CLK-SIG` 表（逐方法签名，直接摘自规范）；T-CLK-005…011 AC 强制对照；可选 public-api snapshot 绑定。

## 本轮结论：FAIL

## fail_count: 8
