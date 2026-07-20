> **状态：Superseded（已被取代）**  
> 权威规范：[`xhyper-testkit-complete-spec.md`](./xhyper-testkit-complete-spec.md)（SPEC-TESTKIT-002）  
> 执行计划：[`plan/plan.md`](../plan/plan.md)（PLAN-TESTKIT-002-v1-complete v1.2.0）  
> 本文仅作历史草案保留；**不得**作为实现或门禁 SSOT。`xlib_test!` / `mock!` 等职责以 002 退役合同为准。

---

# xlib_harness 规范

- **状态**：草案；实现受 [§14](#14-已知未知项与必需决策) 的开放决策阻塞
- **目标 crate**：`crates/testkit`
- **初始版本**：`0.1.0`
- **依据**：`CONSTITUTION.md`、XLib spec v0.2、Approved ADR

## 1. 目的

`xlib_harness` 是 XLib 的 L0 测试支持 crate。它提供 XLib spec §4.1 唯一点名的两个公开入口
`xlib_test!`、`mock!`（均为函数式宏，ADR-010 §2 已裁定，取代 spec.md 早期骨架写法
`` `#[xlib_test]` `` 暗示的属性宏形态），以及可复用的契约测试脚手架，使调用方无需生产适配器或真实基础设施即可测试契约与领域行为。

本文用以下标记区分主张来源：

- **Evidence**：宪法、XLib spec 或 Approved ADR 明文要求；
- **Inference**：为使明文要求自洽所需的最小推论；
- **Unknown**：现有权威未裁定，不得在实现中猜测。

## 2. 依据与优先级

1. `CONSTITUTION.md` 最高（序言、Article VI）。
2. XLib spec v0.2 是 XLib 事实来源，重点为 §§2–6、§8、§9。
3. Approved ADR 可细化 spec；ADR-009、ADR-010 均为 Proposed，尚不是本 crate 的强制决策依据——
   但 ADR-010 §2 对宏形态（函数式宏 vs 属性宏）的裁定已被本文档采纳为现状描述，获批后即生效。
4. `implementation-plan.md` 仅是实现输入；其 §4 只确定 L0 链在 week 1 的顺序。

冲突必须走 XLib spec §9 RFC；本 crate 不得用代码自行裁定。

## 3. 职责

| 职责 | 分类 | 依据 |
| --- | --- | --- |
| 导出 `xlib_test!`（函数式宏） | Evidence + ADR-010 §2（Proposed，裁定形态） | XLib spec §4.1 |
| 导出 `mock!`（函数式宏） | Evidence | XLib spec §4.1 |
| 提供契约测试脚手架 | Evidence | XLib spec §§4.1、6 |
| 保持测试设施边界，不成为业务 crate 的生产依赖 | Evidence | XLib spec §2 R1、§4.4；ADR-007 |
| 独立三段式版本，从 `0.1.0` 起步且仅递增修订位 | Evidence | Constitution §7.3；XLib spec §§3、5 |

### 3.1 非目标

本 crate 不定义生产领域/跨层契约，不实现存储、交易所、传输、调度、可观测、bootstrap 或 evidence；
不连接外部服务，不拥有生产录制数据；不替代负责集成测试工具的 `testkitx`（XLib spec §§4.4、6）；
不在 `mock!` 已批准边界外建设通用 mocking 框架；不预先选择 runtime、executor、断言库、fixture/snapshot
格式或代码生成依赖；不把 observability 移入 L0（ADR-005）。

## 4. 依赖与位置契约

### 4.1 生产依赖

唯一获批的 workspace 常规依赖 是：

```text
xlib_harness -> xlib_standard
```

依据 XLib spec §§3、4.1 与 [`xtask/xtask-spec.md`](../../tools/xtask/xtask-spec.md) §4.2/§4.8，
禁止 normal 依赖 `xlib_evidence`、`xlibgate`、`/types/`、contracts、L1、适配器或 domain crate；
公开宏当前也没有获批的第三方依赖。

### 4.2 消费方依赖种类

- L2.5 Domain crate 只能以 `dev-dependency` 使用本 crate（XLib spec §2 R1；
  [`xtask/xtask-spec.md`](../../tools/xtask/xtask-spec.md) §4.1）。
- 其他层是否可依赖本 crate，按各层已批准的依赖矩阵逐项判断；不得把 R1 扩写成所有业务 crate 的通则。
- `testkitx` 可依赖本 crate 组装更高层集成测试设施（XLib spec §4.4）。
- Cargo feature 不能使被禁的生产依赖边合法；扩大使用范围须先经 spec/ADR 裁定。

### 4.3 Crate 形态（ADR-010 §2 已裁定）

**已裁定（ADR-010 §2，Proposed）**：`xlib_test!`、`mock!` 均实现为 `macro_rules!` **函数式宏**，
不是属性宏；因此本 crate **不需要**成为 `proc-macro` crate，也不需要配套 `proc-macro` crate
——Rust 属性宏才必须由 `proc-macro` crate 导出，函数式宏没有这一限制。§14.1 中"是否需要
proc-macro crate"这一子问题已随本裁定解决；该子问题下的其余问题（是否允许配套 crate 扩展
现有形态、`syn`/`quote`/`proc-macro2` 依赖等）仅在未来确有需要引入属性宏或派生宏时才重新开放，
当前实现不涉及。

## 5. 公开 API 契约

获批表面仅限以下两个名字，不推断其完整语法或展开行为（形态已由 ADR-010 §2 裁定为函数式宏）。

### 5.1 `xlib_test!`

**Evidence**：XLib spec §4.1 要求名为 `xlib_test` 的测试标注入口；**已裁定（ADR-010 §2）**其
形态为函数式宏 `xlib_test!(fn ... { ... })`，透传 `#[test]`。

在 §14.2 裁定前只保证：下游测试可通过文档路径使用；非法参数或不支持的函数形态产生编译错误；
展开不吞掉测试原有成败；执行单元/契约测试不隐式要求真实基础设施。当前最小实现（透传
`#[test]`）是 w1 起点，被 ADR-010 §2 批准为不违反规范的最小占位，不批准为最终能力范围。

参数、sync/async 支持、runtime、timeout、retry、panic、fixture 注入、日志初始化与生成名称均是 **Unknown**。

### 5.2 `mock!`

**Evidence**：XLib spec §4.1 要求名为 `mock!` 的宏。

在 §14.3 裁定前只固定公开名称。非法输入可行动失败、生成项作用域、无隐式外部副作用与确定性
是候选验收目标，须随 grammar 和展开合同一并批准后才成为规范要求。

声明 grammar、支持的 trait 形态、expectation 模型、调用记录、async、associated type、generic、
错误注入与线程安全保证均是 **Unknown**。

### 5.3 契约测试脚手架

XLib spec §§4.1、6 要求契约测试脚手架，并要求 `domain_exchange` 用本 crate 加录制数据做契约测试。
除上述两个宏名外，尚无获批的公开函数、trait、type、module 或 fixture schema。新增表面必须先走 RFC。

**Inference**：同一契约 suite 应能应用到多个实现，且契约定义不能导入具体生产适配器；实现机制未裁定。

### 5.4 Prelude 与重导出

没有获批的 prelude 或公开重导出策略。禁止重导出 L1/适配器具体类型（XLib spec §2 R6）。

## 6. 候选不变量（待批准）

以下目标用于约束后续 API 决策，不是现有权威已经批准的展开或运行时语义；§14 裁定后方可转为强制验收项：

1. L2.5 Domain crate 不获得本 crate 的生产依赖；其他层按已批准矩阵判断。
2. 生成的测试支持默认无外部副作用。
3. 相同显式输入/配置产生相同 harness 控制结果；真实时间、随机数、环境、调度与全局进程状态不得成为隐式输入。
4. 非法宏输入编译失败；契约违反或未满足 expectation 必须使测试失败，不得吞错。
5. 常规 workspace 依赖上限为 `xlib_standard`。
6. dev-dependency 身份不能借生成代码规避生产依赖或重导出规则。
7. 若本 crate 新增 trait，须按 Constitution Article IX 提供 `mock` feature；现有权威未要求新增 trait。

## 7. 候选错误行为（待批准）

### 7.1 编译期

宏 grammar 获批后，非法语法、不支持的 item、展开导致的名称冲突、无效配置应给出定位到输入的可行动诊断；
不得接受后静默忽略选项。诊断文字只有在明确文档化并测试后才是稳定 API；首个公开发布后错误类别稳定，具体措辞可变。

### 7.2 测试执行期

契约违反与未满足 expectation 必须使测试失败。使用 panic、`XError`/`XResult` 还是生成的返回转换，
取决于未决 宏签名。不得只靠 destructor 报错，以免 unwind 隐藏错误或 double panic。

### 7.3 基础设施失败

真实服务失败归 `testkitx` 或被测适配器；无获批契约时，本 crate 不分类、不重试。

## 8. 候选并发与隔离要求（待批准）

当前权威未批准线程模型；以下仅在对应语义获批后作为验收目标：

- 不依赖未同步的可变全局状态。
- 未显式共享调用方状态的测试可由 Rust test runner 并行执行。
- 仅当生成字段与配置行为满足边界时才声明 `Send`/`Sync`；未经批准和性能证据禁止 unsafe（Constitution §4.3）。
- 不保证不同测试、线程或 mock instance 间的顺序。
- async runtime 所有权、虚拟时间、调度确定性、跨线程 expectation 记录均待 §14 裁定。

## 9. Feature 契约

当前未批准本 crate 的任何 feature。`mock!` 是必需公开入口，不等于批准名为 `mock` 的 feature。
不得预加 runtime、snapshot、fixture、serde、tracing 或 adapter feature。若新增公开 trait，
Constitution Article IX 要求对应 `mock` feature，但内容仍须先经 API 决策。

## 10. 测试策略

下列检查随 §14 中对应语义批准后生效；fixture 的权威合规位置也是 §14.4 的阻塞项。

| 范围 | 最小可重复检查 |
| --- | --- |
| `#[xlib_test]` 合法/非法形态 | 获批 grammar 的 pass fixture 可编译运行；compile-fail fixture 被拒绝 |
| `mock!` 合法/非法 grammar | 配置行为可观察；非法 grammar 编译失败 |
| 确定性 | 相同配置重复执行结果一致 |
| 隔离 | 两个独立 instance/test 不泄漏状态或 expectation |
| 并发边界 | compile-pass/fail fixture 证明声明的 `Send`/`Sync` |
| 契约脚手架 | 同一 suite 不修改即可运行于至少两个测试实现 |
| 依赖边界 | `xtask lint-deps` 接受合法 dev 边、拒绝非法 normal 边 |
| 文档 | 所有公开示例作为 doctest 通过，不用 `ignore` |

未批准 `trybuild` 等依赖；默认用 Cargo fixture crate，除非 RFC 批准辅助库。

## 11. CI 与质量门

Constitution §4.5 的仓库门禁为：

```text
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt -- --check
node scripts/check.mjs
```

XLib §8 追加：

```text
cargo run -p xtask -- lint-deps
cargo deny check
```

宏通过/失败 fixture 与 doctest 必须进入 `cargo test`。spec §2/§8 仍要求 `lint-deps` 覆盖 R1–R6；
`lint-deps` 已通过 `check_r6` 提供源码级 `pub use` 静态扫描（见
[`xtask/xtask-spec.md`](../../tools/xtask/xtask-spec.md) §4.8），但该扫描是逐行文本匹配的
最小实现，已知不处理 glob/别名/分组导入等场景，不得解读为"已穷尽证明 R6 合规"。已知局限的
记录与是否拆分独立命令见
[ADR-009](../../../../docs/architecture/adr/009-r6-enforcement-boundary.md)（Proposed）。

## 12. 版本与兼容性

- 独立从 `0.1.0` 发布（Constitution §7.3；XLib spec §5）。
- 每次更新仅递增修订位：`x.y.z → x.y.(z+1)`。
- 首个公开发布后的破坏性变更须记录 CHANGELOG、提供迁移说明并走 RFC；版本号仍仅递增修订位。
- 改变 spec 规则、分层、依赖上限或必需公开名，无论版本均走 XLib spec §9。
- 宏输入语法与可观察 expansion 是公开 API，发布后服从同一兼容性评审规则。
- `cargo-semver-checks` 对所有已有前一版本 tag 的 crate 生效；宏展开行为仍须靠 fixture 证明。

## 13. 验收标准

1. §14 中影响 crate 布局和 宏语法 的决策已批准并回写本文。
2. crate 版本为 `0.1.0`，唯一获批 常规 workspace 依赖是 `xlib_standard`。
3. 下游 fixture 能从文档路径使用 `xlib_test!`、`mock!`。
4. 每类合法形态有 pass fixture，每类拒绝情形有 compile-fail fixture。
5. 非法输入不被静默接受；harness 失败能使测试失败。
6. 并发/隔离语义获批后，两个独立测试按批准模型运行且不泄漏状态。
7. 同一契约 suite 可不修改地运行于至少两个测试实现。
8. Domain dev-dependency fixture 通过 `lint-deps`，等价 normal 边被拒绝。
9. 所有公开 rustdoc 示例通过且无 `ignore`。
10. §11 命令通过，CI 不夸大 R6 覆盖。
11. 无缺少 authority anchor 或 Approved RFC 的依赖、feature、公开项、重导出。

## 14. 已知未知项与必需决策

以下是实现阻塞项，不是选择方便默认值的授权。

### 14.1 打包与依赖

- ~~`xlib_harness` 自身是否为 `proc-macro` crate，或是否允许配套 `proc-macro` crate？~~
  **已裁定（ADR-010 §2）**：否——两个宏均为 `macro_rules!` 函数式宏，不需要 `proc-macro` crate。
- 以下子问题因上一条已裁定而暂不适用，仅在未来确有需要引入属性/派生宏时重新开放：
  companion crate 的新 workspace 路径是否修订 XLib spec §3；companion crate 的依赖分类、
  workspace 成员身份及 `lint-deps` 层级映射；可否使用 `syn`、`quote`、`proc-macro2`；如何同时
  暴露 attribute、function-like macro 与普通契约脚手架。

### 14.2 `xlib_test!` 语义

- 参数与可标注函数签名；sync/async 支持范围。
- async runtime/executor、所有权、feature gate、虚拟时间策略。
- timeout、retry、panic、Result、fixture、环境、日志与生成名称/冲突诊断。

### 14.3 `mock!` 语义

- 输入是声明 mock、指向既有 trait，还是配置既有 mock？
- 方法、泛型、生命周期、关联项、默认/异步方法 支持范围。
- expectation、默认行为、次数/顺序、参数匹配、返回/错误/panic 注入、诊断。
- 调用历史所有权、poison、`Send`/`Sync`、unwind 行为及与 per-trait `mock` feature 的关系。
- 一份跨 L0 兼容矩阵必须同时覆盖 `Clock`、`EvidenceSink`、`Capability`：trait 所属 crate 的
  `mock` feature、harness 集成、合法依赖方向、生成类型和原 trait bound；本文不预判实现归属。

### 14.4 契约脚手架与 fixture

- runner/suite 公开 API 使用 function、trait、macro 还是 module？
- XLib spec §6 录制交易所报文的所有权、规范格式、脱敏、来源、许可、版本、损坏处理、确定性 replay。
- 由哪一层完成 replay 数据到 contracts/domain 类型的转换且不违反 R1/R2/R6？
- suite 是否同时覆盖 storage adapter，共享 suite 放在哪里？
- compile-pass/fail fixture 放置在哪里、是否作为 workspace 成员，以及如何遵守 Constitution Article VIII
  的测试布局限制？权威裁定前不得默认创建独立 `tests/` 目录。

### 14.5 Clock 与确定性

- harness 时间如何关联 `xlib_standard::Clock`，fake clock 由哪一方拥有、另一方如何消费？
- fake clock API、初始 epoch、单调性、手动推进、溢出、倒退、并发观察语义。
- 随机 seed 与环境/进程隔离是否在范围内？

### 14.6 失败与可观测策略

- 运行期失败使用 panic、`XError`/`XResult` 还是生成 adapter？
- expectation 显式还是自动检查，如何避免 double panic？
- 是否接入 tracing？ADR-005 未授权 L0 observability 依赖。
- 是否记录 `xlib_evidence`？当前依赖上限不允许。

### 14.7 未采纳的 Proposed 项

`implementation-plan.md` §3.6 的 `HealthCheck` 不属于本契约；它在 contracts、xlibgate 或未来 ADR
中的归属未决。ADR-009 为 Proposed，不为本 crate 增加 API 或 CI 命令。

## 15. 可追溯性摘要

| 契约范围 | 稳定依据锚点 |
| --- | --- |
| L0 职责、公开名、依赖 | XLib spec §§3、4.1 |
| 测试依赖边界 | XLib spec §2 R1；ADR-007；[`xtask/xtask-spec.md`](../../tools/xtask/xtask-spec.md) §4.1/§4.2 |
| 宏形态（函数式宏，非属性宏） | ADR-010（Proposed）§2 |
| `testkitx` 关系 | XLib spec §§4.4、6 |
| trait 的 mock feature | Constitution Article IX |
| 并发安全 | Constitution §4.3 |
| 测试、doctest、仓库质量门 | Constitution §§4.4–4.5 |
| 独立版本 | Constitution §7.3；XLib spec §5 |
| CI | XLib spec §8；[`xtask/xtask-spec.md`](../../tools/xtask/xtask-spec.md) §4.8 |
| 架构变更 | Constitution Article VI；XLib spec §9 |
| 可观测组装边界 | ADR-005 |
