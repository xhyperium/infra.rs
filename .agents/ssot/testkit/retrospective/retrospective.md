> **Post-ship**：`publish = false`（internal only，crates.io package 名 **`xhyper-testkit`**）；战役 **COMPLETE** · 0.1.1 Stable CLAIMED。

# Retrospective — testkit 002 执行

| 字段 | 值 |
|------|-----|
| Ship PR | [#247](https://github.com/xhyperium/xhyper.rs/pull/247) · [#254](https://github.com/xhyperium/xhyper.rs/pull/254) · [#255](https://github.com/xhyperium/xhyper.rs/pull/255) |
| Spec Stable | 2026-07-14 |
| Date | 2026-07-17（post-ship 复盘） |

## 学到的

1. **ManualClock 必须 Mutex + poison 显式恢复**：checked wall/mono API + `ManualClockFault` 三态映射 `ClockError` + `ManualClockSnapshot`；不提供 `Clone`/`Default`，避免共享状态被无声复制（spec §7）。
2. **退役宏是合同义务不是清理**：`xlib_test!`/`mock!`/`FixtureBuilder`/`provider_capability_contract_tests!` 的删除写进 spec §8 退役合同 + 防回流门禁，而非一次性 git rm。
3. **test-support plane 与生产图正交**：testkit → kernel 是测试图依赖，绝不进生产 normal graph；`cargo xtl test-graph-check` 把这个不变量机控化（spec §14）。
4. **contract-testkit 独立 crate**：trait-level suites 放 `contract-testkit`（→ testkit + contracts + canonical），不污染 testkit core 的极简公开面。
5. **Approved/Stable ≠ 全闭合 ≠ production ready**：薄壳 spec.md 与 plan 多处加 Note 区分三层语义；`branch cov ≥90%` OPTIONAL 诚实标注，不粉饰为实测 PASS。
6. **薄壳 → complete-spec → cmp 双镜像**：spec/spec.md 从「68 行薄壳指针」收敛为「与 complete-spec 字节镜像」（AGENTS.md §2.4），消除薄壳与正文的状态漂移；薄壳原增量（已实现范围/实测门禁）并入 complete-spec §21.1/§24.0/§26。
7. **十轮机械复检 + residual ledger**：沿用 kernel 002 经验；plan 十轮 pass3 判 fail_rounds=0，round-findings 作为过程证据归档（plan/archive/），不污染长期 SSOT。

## 本波结果

| 项 | 结果 |
|----|------|
| Spec Stable | **PASS** |
| W0–W6 ship | **PASS**（PR #247 #254 #255 · tag testkit-v0.1.1） |
| Plan 十轮验收 | **PASS**（fail_rounds=0） |
| §24 验收主体 + Stable | **PASS**（branch cov OPTIONAL） |
| residual 清零 | **PASS**（DEF-001…010 全 CLOSED · 1 OPTIONAL） |
| 下一门禁 | 保持 Stable · 破坏性改动走新 spec 版本 |

**战役 COMPLETE。0.1.1 Stable CLAIMED。`publish = false` 保持。**
