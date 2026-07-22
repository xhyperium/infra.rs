# GATE-TESTKIT-002 · 交付门禁

| 字段 | 值 |
|---|---|
| Spec | [spec/spec.md](../spec/spec.md) |
| Design | [design/design.md](../design/design.md) |
| Test | [test/test.md](../test/test.md) |
| 当前 package 版本 | `0.1.3` |
| 当前裁定 | **NO-GO / R3 GATES PASS / INDEPENDENT REVIEW PENDING** |

当前 active 合同与实现已收敛，固定内容候选的机器门禁已闭合；独立终审尚未闭合。历史 Stable/COMPLETE/PASS 不自动继承到本轮候选。

## 1. 必须闭合的门禁

| Gate | 当前状态 | GO 条件 |
|---|---|---|
| G-01 SSOT 一致 | `387a1dc` PASS / REVIEW PENDING | spec/design/test/gate/matrix/AGENTS 使用 `testkit`、`0.1.3` 当前事实与同一边界；旧战役文档显式标为历史 |
| G-02 ManualClock 合同 | `387a1dc` PASS / REVIEW PENDING | 单 Mutex、checked、fault、snapshot、poison、独立 domain 全部有新鲜测试 |
| G-03 Runner typed API | `387a1dc` PASS / REVIEW PENDING | 导出 `HarnessReport`、`HarnessRunError`、四态 `StepOutcome`；`StepRecord` 字段私有且 getter 完整 |
| G-04 Runner fail-closed | `387a1dc` PASS / REVIEW PENDING | `step(self)->Self`、`run(self)` 编译期排除重跑/运行后追加；clock fault/panic 返回 terminal error；首错停止 |
| G-05 无 sentinel | `387a1dc` PASS / REVIEW PENDING | 所有 runner/clock 错误路径均不使用 epoch 0、空串或布尔成功掩盖失败 |
| G-06 图隔离 | `387a1dc` PASS | 仅 dev-dependency 消费；normal production dependents 为零 |
| G-07 外部边界 | `387a1dc` PASS | crate 无网络/进程/I/O/真实时间；external harness 仍在 tools/CI OOS |
| G-08 质量门禁 | `387a1dc` PASS / REVIEW PENDING | fmt/clippy/test/API surface/相关质量门禁在固定候选上新鲜通过 |
| G-09 版本同步 | `387a1dc` PASS / REVIEW PENDING | 行为变化交付执行 PATCH bump，并同步 Cargo/lock/消费者/CHANGELOG/对齐文/SSOT |

## 2. Residual

| ID | 状态 | 边界 |
|---|---|---|
| `R-CLK-DOMAIN-EXHAUSTION` | OPEN | 唯一性只覆盖单进程生命周期内 allocator 未耗尽；未闭合 typed exhaustion/防回绕前不得扩张声明 |
| `R-EXTERNAL-HARNESS` | OOS | 真实服务、网络、进程、容器、端口、凭据、故障注入和 CI evidence 归 tools/CI；不是 crate runner 的缺失实现 |

## 3. GO / NO-GO 规则

只有 G-01 至 G-09 全部满足，且固定候选的新鲜 evidence 可追溯时，才可将本轮裁定改为 GO。`R-CLK-DOMAIN-EXHAUSTION` 若未闭合，必须由 reviewer 明确判定其是否阻塞本次版本；不得自行改写为 PASS。

以下说法始终禁止：

- `testkit` 是 production runtime；
- package 已发布或 package stable；
- 历史 evidence 等于本轮 fresh PASS；
- external harness OOS 等于 crate 内禁止任何 harness；
- “ManualClock 族四类型”等于 crate 只能导出四个类型。
