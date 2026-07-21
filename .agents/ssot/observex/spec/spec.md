# observex 实现规范

状态：当前 `0.1.0` 最小实现与 ADR-005 差距的 active 验收合同

- Package / lib：`xhyper-observex` / `observex`
- Implementation snapshot：`b0934baa`（2026-07-15）
- Document commit：`e0b98df4`
- Verified at：`e0b98df4`（相关实现路径未变化）
- Candidate：[SPEC-INFRA-OBSERVEX-002](../../../../draft/xhyper-observex-complete-spec.md)（Draft，非权威，不覆盖本文）

## 0. 文档定位与裁定边界

本文使用 **证据（Evidence）**、**推论（Inference）**、**未知（Unknown）** 区分当前事实、最低验收
解释和待评审事项；服从 XLib spec v0.2 与 ADR-005。代码与 Approved ADR 不一致时，必须记录差距，
不能把现状静默升级为架构批准。

## 1. 定位、职责与非目标

- **证据**：`observex` 位于 `crates/observex`，是 L1 tracing/metrics 统一封装，依赖
  `kernel`、`contracts`，目标包括 OpenTelemetry 导出并实现
  `contracts::Instrumentation`（XLib spec §4.4）。
- **证据**：当前实现只把三类事件写入 `tracing::info!`；没有 OpenTelemetry exporter、缓冲、flush
  或 shutdown。
- **证据**：ADR-005 指定具体实现名 `ObservexInstrumentation`，当前代码公开的名称是
  `TracingInstrumentation`。行为接口已实现，但命名尚未对齐 Approved ADR。

非目标：业务审计（属于 `xhyper-evidence`）、重试/熔断策略、全局组装，以及让其他 L1 直接依赖本 crate。

## 2. 位置、依赖与版本

| 项目 | 当前事实 | 合同 |
| --- | --- | --- |
| 路径 | `crates/observex` | L1 Infra |
| 版本 | `0.1.0` | 独立维护；每次只允许 `x.y.z → x.y.(z+1)` |
| 普通依赖 | `xhyper-kernel`, `xhyper-contracts`, `tracing` | 前两者符合 spec；`tracing` 是当前后端 |
| feature | 无 | exporter/runtime feature 尚未裁定 |

`kernel` 当前仅以保留导入存在，没有直接参与公开行为。OpenTelemetry SDK、导出协议、runtime、
资源属性和配置入口仍为 **未知**。

## 3. 当前公开 API（代码事实）

| API | 当前语义 |
| --- | --- |
| `TracingInstrumentation` | 零字段公开类型；实现 `Debug + Default + Clone + Copy` |
| `TracingInstrumentation::new() -> Self` | 创建该零字段实现 |
| `Instrumentation::record_retry` | `tracing::info!` 记录 `op`、`attempt` 和 `"retry"` |
| `Instrumentation::record_circuit_open` | `tracing::info!` 记录 `op` 和 `"circuit_open"` |
| `Instrumentation::record_circuit_close` | `tracing::info!` 记录 `op` 和 `"circuit_close"` |

当前没有 `ObservexInstrumentation`、构造配置、subscriber 安装、metric handle、exporter、flush 或 shutdown
API。ADR-005 的命名差距须通过代码重命名/兼容别名修复，或先修订 ADR；本文不替代该决定。

## 4. 行为、不变量与差距

1. **证据——解耦**：实现 contracts trait；resiliencx 无需依赖 observex。
2. **证据——事件映射**：三个 trait 方法各产生对应的 info event；没有 subscriber 时调用仍不 panic。
3. **证据——同步热路径**：方法为同步调用，当前没有显式 I/O、队列或锁。
4. **推论——失败隔离**：观测调用不得改变被观测业务结果；未来 exporter 失败也须保持此边界。
5. **推论——基数与敏感性**：`op` 必须来自受控集合，不得直接使用订单号、用户输入、secret、PII
   或完整 URL；当前代码未强制这一点。
6. **未知——导出合同**：指标名、单位、标签、span 事件、采样、缓冲满策略和 OpenTelemetry 映射。

当前 `tracing::info!` 实现满足 trait 行为的最小演示，但不等于已经完成 spec §4.4 的 OpenTelemetry
导出目标，也不证明生产级有界性或丢弃策略。

## 5. 错误、并发与生命周期

- `TracingInstrumentation` 为零字段 Copy 类型，可在线程间共享；trait 自身要求 `Send + Sync`。
- 当前记录 API 不返回错误，subscriber/exporter 状态对调用方不可见；没有 flush/shutdown 生命周期。
- **未知**：初始化失败、缓冲满、exporter 重试、flush timeout、关闭错误及递归诊断策略。
- 未来实现不得在同步热路径无界阻塞，也不得因观测失败 panic 或覆盖业务错误。

## 6. 测试合同

当前测试只证明：三个方法可调用且不 panic、Default/new 可构造、trait object 可用、Clone/Copy 行为可用。
当前版本至少运行：

```text
cargo test -p xhyper-observex
cargo check -p xhyper-observex --all-targets
cargo clippy -p xhyper-observex --all-targets -- -D warnings
cargo fmt -- --check
cargo xtl lint-deps
```

尚缺：捕获并断言 tracing event 字段、受控基数、敏感值处理、并发记录，以及 exporter/flush/shutdown
测试（相关 API 获批后）。还须证明 resiliencx 的 Cargo 图中没有 observex。

## 7. 验收标准与开放决策

- [ ] 当前依赖、`TracingInstrumentation` API 和测试与源码一致。
- [ ] ADR-005 的 `ObservexInstrumentation` 命名通过代码兼容或 ADR 修订解决。
- [ ] 文档不把 tracing info 事件宣称为已完成的 OpenTelemetry 导出。
- [ ] bootstrap 注入链落地后证明 observex 与 resiliencx 无直接依赖。
- [ ] 每次版本更新仅执行 `x.y.z → x.y.(z+1)`，兼容性治理独立执行。

仍需裁定：类型命名迁移；SDK/exporter；metric 名称、单位和标签；span API；采样；缓冲/丢弃；
初始化、flush/shutdown；测试 exporter。

## 8. 可追溯性

| 合同 | 来源 |
| --- | --- |
| 职责、依赖、OpenTelemetry 目标 | XLib spec §4.4 |
| Instrumentation 签名 | XLib spec §4.3；`crates/contracts/src/lib.rs` |
| 具体实现命名与注入 | ADR-005 |
| 当前 tracing 实现与测试 | `crates/observex/src/lib.rs` |
| 当前依赖与版本 | `crates/observex/Cargo.toml` |
| 版本更新规则 | Constitution §7.3；XLib spec §5 |
