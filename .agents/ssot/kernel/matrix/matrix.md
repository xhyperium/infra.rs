# Matrix — kernel

| 字段 | 当前值 |
|------|--------|
| Active Spec | `SPEC-KERNEL-002` · Approved / Active |
| Active Design | `DESIGN-KERNEL-002` |
| Package / lib | `kernel` / `kernel` |
| Current version | `0.3.1` |
| Distribution | `publish = false` |
| Maturity | L1 Internal Ready；L4 仅限已证支持面 |
| Delivery | 候选实现与一次 PATCH bump 已落地；R1/R2 focused PASS；R3 声明收敛候选待固定 SHA 重验 |
| Production certification | 未声明 |

## 合同矩阵

| 面 | 当前合同 | 必要证据 |
|----|----------|----------|
| Error | 9 类反应分类；opaque `XError` | unit / property / doctest / API |
| Timestamp | `i64` 纳秒；checked 全域算术 | unit / property |
| Clock domain | process domain + 跨 domain `None` | unit / integration |
| SystemClock | 所有实例共享进程 origin | 多实例集成测试 |
| 构造 seam | 两个 `#[doc(hidden)]` 公开 seam | API 基线 / 调用边界审查 |
| Shutdown | Mutex + Condvar；一次触发 | unit / integration / loom |
| Timeout | `Result<bool, WaitTimeoutError>` | std unit / integration |
| Overflow / completion precedence | 未触发时 `Duration::MAX` 为 typed Err；已触发时立即 `Ok(true)` | 两种状态的精确断言 |
| Negative API | 无 Default/serde/Component | doctest / static assertion |
| Public API | 根导出含 `WaitTimeoutError` | public API gate |

## 状态解释

`Approved` 是合同状态；L1 Internal Ready 是内部使用准备度；L4 只覆盖当前新鲜证据明确证明的支持面。

这些状态不等于 crates.io 发布、全量 Platform Ready 或 production-certified。历史 `xhyper-kernel 0.1.1` 不是当前 package/version/distribution 事实。

## Evidence 规则

2026-07-14 历史 evidence 仅作背景。当前矩阵的 PASS 必须绑定本轮被验收 commit；未运行、SKIP 或旧日志不得填写为 PASS。
