# Round 1 — §0–§3 定位 / 问题本质 / 当前裁定 / 组件划分

> Verifier: 只读计划完备性检查（非实现验收）  
> Source Spec: `.agent/SSOT/testkit/testkit-complete-spec.md`  
> Plan pack: `plan.md` · `tasks.md` · `gap-matrix.md` · `spec-inventory.md` · `residual-open.md` · `approval-packet.md` · `.worktree/testkit-todo.md`  
> 日期: 2026-07-14

## 检查项

| 规范 | 要点 | plan | tasks | inventory / residual / gap | 判定 |
|------|------|------|-------|----------------------------|------|
| §0 身份 | T0 / test-support；非 L0/L1/mock/集成系统 | plan §0.1–0.3；Campaign plane | T-ARCH-001；T-ARCH-004 | I-1.3；DEF-001 | PASS |
| §0 依赖图 | Production vs Test graph（testkit→kernel；contract-testkit；harness） | plan §0.1 | T-CTC-001；residual harness DEFER | I-1.1–1.6 | PASS |
| §0 获批后·layer | `workspace.toml` kernel→test-support | plan §0.5 DEF-001；§1.2 PR-6 | T-ARCH-001 | I-VER-LAYER；gap §0 | PASS |
| §0 获批后·架构图 | Test Support 画成正交测试平面 | plan 仅「测试平面叙述」 | T-ARCH-004/005 文档叙述，无「架构图」交付物 | I-DONE-24.1 有 README/architecture 对齐，无 diagram | **FAIL** |
| §0 获批后·dev-dep only | 业务 crate 仅 dev-dependency | plan §0.6；Forbidden #4 | T-GATE-001…006；T-FREEZE-001 | I-2.5；I-GRAPH | PASS |
| §0 获批后·宏退役 | xlib_test!/mock!/FixtureBuilder 退役 | plan DEF-004 | T-DEL-001…003 | I-DEL-1…3 | PASS |
| §0 获批后·provider 迁出 | suite 迁独立测试支持 crate | plan DEF-005/006 | T-DEL-004；T-CTC-* | I-DEL-4；I-CTC | PASS |
| §0 获批后·core 仅证明过的原语 | 准入后只留确定性原语 | plan §0.1–0.2 | 准入叙述在 plan；无独立 I-ADMIT | 无 I-ADMIT 枚举，但 plan §0.2 有八问 | PASS（弱） |
| §1 核心价值 | 隐式输入→显式可注入/推进/检查 | plan §0.1 | — | I-DET 部分重叠 | PASS |
| §1 不稳定来源清单 | 墙钟/单调钟/sleep/随机/调度/全局态/env/外部服务/网络/磁盘/未版本化 fixture/默认 mock/吞错宏 | plan 未逐条枚举 | 无 | I-DET 覆盖时间/sleep/顺序/全局/seed/时区/env/端口/吞错/retry；**缺**磁盘、未版本化 fixture、默认值 mock、吞错宏、线程调度 等 §1 原文 bullet | **FAIL** |
| §1 准入八问 | 8 条全满足才进 core | plan §0.2 | gap §1 → W0/W6；无强制 RFC 任务 | 无 I-ADMIT-* | PASS（plan 有全文；inventory 缺 ID） |
| §1 否决去向 | 消费方 / contract-testkit / harness | plan §0.2 | — | — | PASS |
| §2.1 保留 ManualClock 方向 | 独立 wall/mono + Clock trait | plan §0.3–0.4；§4 | T-CLK-* | I-CLK-* | PASS |
| §2.1 收紧清单 | 禁静默回绕；fault；snapshot；并发线性化；禁 SeqCst 顶替；控制 API Result；禁真实 sleep 验证 | plan Forbidden + §4 | T-CLK-007…018；T-CLK-016 | I-CLK；Forbidden #7/#8 | PASS（线性化语义见 Mutex/同锁；显式「线性化点」定义在 §13.4，本轮以 §2.1 映射为够） |
| §2.2 xlib_test! | 无隔离/超时/确定性…；终态删 | plan DEF-004 | T-DEL-001 | I-DEL-1 | PASS |
| §2.2 mock! | 空壳非 mock；必删 | plan DEF-004 | T-DEL-002 | I-DEL-2 | PASS |
| §2.2 FixtureBuilder | PhantomData 占位；必删 | plan DEF-004 | T-DEL-003 | I-DEL-3 | PASS |
| §2.2 provider 宏 | 隐藏依赖 + 硬编码行为列表 | plan DEF-005；硬编码摘要 | T-DEL-004；T-CTC-005…；I-CTC-8 仅「server_time==0 等」 | **硬编码全表未入库** | **FAIL** |
| §3.1 testkit | path/package/plane/role/deps/公开面/无宏 | plan §0.1；§5 | T-CLK；T-DEL；T-ARCH | I-1.*；I-API | PASS |
| §3.2 contract-testkit | path/package/deps/按 trait 分 suite/禁一锅宏 | plan §5 | T-CTC-001…017 | I-1.4–1.5；I-CTC-1…7 | PASS |
| §3.3 Integration Harness | path 候选 + 职责清单（Docker/Compose、Redis/Kafka/PG/TDengine、testnet、网络故障、进程 kill、真实端口/凭据、Evidence）+ testkit 不承担 | residual DEFER「完整实现」；gap PARTIAL | 无 OUT-OF-SCOPE 职责枚举任务 | I-1.6 仅路径三选一，**无职责 bullet** | **FAIL** |
| §3.4 Fixture 所有权 | schema 归属 crate；示例路径；≥2 crate 复用才可 `crates/test-support/fixtures/<schema>`；禁回流 FixtureBuilder | gap 写 OK（碰巧）；禁 builder 有 | 仅 T-DEL-003 删 builder | I-DEL-3；**无 shared fixture 路径/准入** | **FAIL** |

## PASS

- T0 / test-support 身份与「非生产分层」在 plan、gap、DEF-001、I-1.3、T-ARCH-* 有完整映射。
- Test graph 三层（testkit / contract-testkit / integration harness）在 plan §0.1 与 I-1 有路径级映射。
- 获批后 layer、dev-dep only、宏退役、provider 迁出、ManualClock 收紧方向均有 Wave/Task/DEF。
- §1 核心价值句与准入八问全文在 plan §0.1–0.2。
- §2.1–2.2 删除对象（四项）与保留方向均有 DEF + T-DEL / T-CLK。
- §3.1–3.2 组件路径、依赖、公开面、suite 拆分在 inventory 与 W4 任务齐全。

## FAIL

1. **§0 获批后·架构图正交平面**  
   - **规范引用**: §0「架构图必须将 Test Support 画成正交测试平面」  
   - **缺失内容**: plan/tasks 仅有 `docs/architecture/spec.md`「测试平面叙述」（T-ARCH-004）与 STRUCTURE/TECH 对齐（T-ARCH-005），无「架构图/图示」交付物或验收 AC。  
   - **建议补丁位置**: `spec-inventory.md` 增 `I-1.7`（架构图正交平面）；`tasks.md` W5 增 `T-ARCH-0xx`「更新架构图（STRUCTURE/ARCHITECTURE 图或 spec 内 diagram）将 Test Support 画为正交平面」；`plan.md` §0.5/W5 退出条件引用该 I-*。

2. **§1 不稳定来源完整清单**  
   - **规范引用**: §1 列表（真实墙钟、真实单调钟、sleep、随机数、线程调度、全局状态、环境变量、外部服务、网络、磁盘、未版本化 fixture、默认值 mock、吞错宏）  
   - **缺失内容**: inventory `I-DET` 仅覆盖 §11 确定性子集；plan 未把 §1 完整列表登记为问题域/反目标 checklist，导致「磁盘 / 未版本化 fixture / 默认值 mock / 吞错宏 / 线程调度」等无防遗漏 ID。  
   - **建议补丁位置**: `spec-inventory.md` 增 `I-1-IMPLICIT` 13 bullet；`gap-matrix.md` §1 行注明映射；可选 `plan.md` §0.1 附完整列表引用 I-*。

3. **§2.2 provider 宏硬编码行为全表**  
   - **规范引用**: §2.2 `provider_capability_contract_tests!` 硬编码：`stream 必须为空`；`server_time == 0`；`position/balance 必须为空`；`query_order == Pending`；`invalid venue cancel 必须失败`；隐藏 dep `canonical/contracts/futures_util/tokio`  
   - **缺失内容**: `I-CTC-8` 仅写「禁硬编码 server_time==0 等」；tasks 无逐条「硬编码断言移除/改 Profile」AC；隐藏依赖虽在 DEF-005/T-CTC-002，硬编码行为迁移清单未入库。  
   - **建议补丁位置**: `spec-inventory.md` 增 `I-CTC-HARDCODE-1…N`；`tasks.md` `T-CTC-005…009` AC 引用并要求每条改为 profile 或删除；`gap-matrix.md` DEF-005 明细表。

4. **§3.3 Integration Harness 职责边界未枚举**  
   - **规范引用**: §3.3 role 列表（Docker/Compose；Redis/Kafka/PostgreSQL/TDengine；交易所 testnet；网络故障；进程 kill；真实端口；真实凭据注入；Evidence artifact）+「testkit 不承担」  
   - **缺失内容**: residual 仅 `DEFER(pending accept) integration harness 完整实现`；I-1.6 只有路径，无「本战役 OUT-OF-SCOPE 职责」枚举；tasks 无边界声明任务。实现者可能把 harness 能力误塞进 testkit 或 contract-testkit。  
   - **建议补丁位置**: `spec-inventory.md` 增 `I-1.6.a…h` OUT-OF-SCOPE；`residual-open.md` DEFER 项附职责列表并标 `accepted` 条件；`plan.md` §0.1 明确「职责 bullet 禁止进入 testkit core AC」。

5. **§3.4 Fixture 共享路径与两消费者准入**  
   - **规范引用**: §3.4 领域 fixture 归属 schema 拥有 crate；示例路径；**仅当 ≥2 独立 crate 复用**才允许 `crates/test-support/fixtures/<schema>`；禁回流无行为 `FixtureBuilder<T>`  
   - **缺失内容**: 仅有删除 FixtureBuilder（I-DEL-3 / T-DEL-003）；**完全无** shared fixture 路径、两消费者准入、示例路径归属策略的 I-* 或 Task。gap 写「OK（碰巧）」不能代替合同映射。  
   - **建议补丁位置**: `spec-inventory.md` 增 `I-FIXTURE-1`（归属）、`I-FIXTURE-2`（两消费者门闩）、`I-FIXTURE-3`（允许路径）；`tasks.md` W0/W5 增冻结任务「禁新建 test-support/fixtures 除非准入证明」；`plan.md` Forbidden 可增一条。

## 本轮结论：FAIL

## fail_count: 5
