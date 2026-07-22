# GATE-TESTKIT-002 · 交付门禁

| 字段 | 值 |
|---|---|
| Spec | [spec/spec.md](../spec/spec.md) |
| Design | [design/design.md](../design/design.md) |
| Test | [test/test.md](../test/test.md) |
| 当前 package 版本 | `0.1.3` |
| 当前裁定 | **REVIEW PENDING — PR #258 coverage 修复待固定候选** |

前一证据候选 `c27b7ce` 的终门禁发现 `thiserror`、panic 首错停止、公开方法测试与并发重叠缺口；
该候选已经失效。最终状态校正候选 `c4604ce` 的全量机器门禁与 `fff07ea` 独立终审曾通过；
PR #258 随后发现 testkit 100% 行覆盖门禁缺口，旧 GO 按 failure condition 失效，新候选完成机器门禁与独立重审前保持 REVIEW PENDING。

## 1. 必须闭合的门禁

| Gate | 当前状态 | GO 条件 |
|---|---|---|
| G-01 SSOT 一致 | `c4604ce` PASS / REVIEW GO | spec/design/test/gate/matrix/AGENTS 使用 `testkit`、`0.1.3` 当前事实与同一边界；旧战役文档显式标为历史 |
| G-02 ManualClock 合同 | `c4604ce` PASS / REVIEW GO | 单 Mutex、checked、fault、snapshot、poison、独立 domain 全部有新鲜测试 |
| G-03 Runner typed API | `c4604ce` PASS / REVIEW GO | 导出 `HarnessReport`、`HarnessRunError`、四态 `StepOutcome`；`StepRecord` 字段私有且 getter 完整 |
| G-04 Runner fail-closed | `c4604ce` PASS / REVIEW GO | `step(self)->Self`、`run(self)` 编译期排除重跑/运行后追加；clock fault/panic 返回 terminal error；首错停止 |
| G-05 无 sentinel | `c4604ce` PASS / REVIEW GO | 所有 runner/clock 错误路径均不使用 epoch 0、空串或布尔成功掩盖失败 |
| G-06 图隔离 | `c4604ce` PASS / REVIEW GO | 仅 dev-dependency 消费；normal production dependents 为零 |
| G-07 外部边界 | `c4604ce` PASS / REVIEW GO | crate 无网络/进程/I/O/真实时间；external harness 仍在 tools/CI OOS |
| G-08 质量门禁 | `c4604ce` PASS / REVIEW GO | fmt/clippy/test/API surface/相关质量门禁在固定候选上新鲜通过 |
| G-09 版本同步 | `c4604ce` PASS / REVIEW GO | 行为变化交付执行 PATCH bump，并同步 Cargo/lock/消费者/CHANGELOG/对齐文/SSOT |

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
