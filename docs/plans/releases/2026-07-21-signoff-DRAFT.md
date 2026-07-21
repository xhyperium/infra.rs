# 生产签核草稿 — PLAN-CORE-PROD-002 后（**未签核**）

> **状态：DRAFT · Agent 已填充证据 · 禁止当作已签核**  
> Maintainer 请审阅证据后：**复制**本文件到正式  
> `docs/plans/releases/<version>-signoff.md`（例：`0.3.0-signoff.md`），再在**正式文件**中手签。  
> **禁止**在仍命名为 `*-DRAFT.md` 的文件上完成最终签核。  
> 模板 SSOT：[`docs/governance/prod-signoff-TEMPLATE.md`](../../governance/prod-signoff-TEMPLATE.md)  
> 交接摘要：[`2026-07-21-MAINTAINER-HANDOFF.md`](./2026-07-21-MAINTAINER-HANDOFF.md)

---

## 元信息

| 字段 | 值 |
|------|----|
| 版本 / Tag | _待 Maintainer 在正式 signoff 文件中指定（当前 workspace `0.3.0`，`publish = false`）_ |
| 关联 | PLAN-CORE-PROD-002 · beads `infra-asa` · PR #98 #120 #121 #124 #125 #127 #128 #138 #141 |
| 支持矩阵 | [`support-matrix.md`](../../governance/support-matrix.md)：Linux x86_64 + MSRV 1.85 |
| 证据基线 commit | `b0154db`（`main` 含 #141 验收勾选回写） |
| 草稿日期 | 2026-07-21 |
| Agent 证据会话 | 2026-07-21T09:31:06Z UTC · worktree `docs/signoff-evidence-fill` |
| 签核人 | **空白 — 仅 Maintainer 在正式 signoff 文件填写** |
| 签核结论 | **空白 — GO / NO-GO / GO-with-Accepts 仅人类在正式文件选择** |

---

## 红线（再声明）

```text
Maintainer only — Agent must not sign.
本文件是草稿，不是 Production Ready 批准。
Agent 可填证据与「证据是否齐备」说明；不得选择最终 GO/NO-GO；
不得填写签核人；不得代写 Signed-off-by。
最终签核必须落在 docs/plans/releases/<version>-signoff.md（非 *DRAFT* 文件名）。
```

---

## 已合入实现证据（Agent 整理）

| 波次 | PR | squash | 合入摘要 |
|------|-----|--------|----------|
| 基线 | [#98](https://github.com/xhyperium/infra.rs/pull/98) | `76c56d7` | 五核心 P0/P1 可机器验证子集 |
| W0 | [#120](https://github.com/xhyperium/infra.rs/pull/120) | `3b82fe7` | 计划 + artifacts 冻结 |
| W1 | [#121](https://github.com/xhyperium/infra.rs/pull/121) | `0e01f97` | decimalx oracle / 边界 / panicking 门禁 / scheduled miri·mutants |
| W2 | [#124](https://github.com/xhyperium/infra.rs/pull/124) | `ee45d97` | canonical committed wire v1.1–v1.3 |
| W3 | [#128](https://github.com/xhyperium/infra.rs/pull/128) | `d72dcc4` | contracts 语义文档 + fakes + venue override 门禁 |
| W4 | [#125](https://github.com/xhyperium/infra.rs/pull/125) | `10954c3` | adapters **离线 mock** 验证入口 |
| W5 | [#127](https://github.com/xhyperium/infra.rs/pull/127) | `f214eeb` | support-matrix / public-api baselines / 签核模板 |
| 收尾 | [#138](https://github.com/xhyperium/infra.rs/pull/138) | `bbdb083` | 计划状态 + 本 DRAFT 初版 |
| 勾选回写 | [#141](https://github.com/xhyperium/infra.rs/pull/141) | `b0154db` | 验收勾选 + 审计报告 §12 |

DEFER 表：[`../artifacts/defer-disposition.md`](../artifacts/defer-disposition.md)  
计划 DONE 状态：[`../2026-07-21-core-crates-production-readiness.md`](../2026-07-21-core-crates-production-readiness.md) §15（**人签前未关闭**）  
审计 post-W5：[`../../report/2026-07-21/core-crates-production-readiness.md`](../../report/2026-07-21/core-crates-production-readiness.md) §12

---

## L1 — 正确性与不变量

| 项 | Maintainer 勾选 | Agent 证据状态 |
|----|-----------------|----------------|
| 核心 crate 测试在官方矩阵通过 | [ ] | 已采集（见下） |
| decimalx：非法 scale 不可表示；checked；oracle | [ ] | 已采集 — `tests/oracle_diff.rs` + #98/#121 |
| 无「整体 Production Ready」错误对外表述 | [ ] | 已核对 — 计划/报告/DRAFT 均写 **否** |

**Agent 证据（本地 · `b0154db` · 2026-07-21T09:31Z UTC）：**

```text
cargo test -p contracts -p decimalx -p canonical -p kernel -p testkit --all-targets
→ 通过（exit 0）

cargo clippy -p contracts -p decimalx -p canonical -p kernel -p testkit --all-targets -- -D warnings
→ Finished dev profile（exit 0）

node scripts/quality-gates/check-canonical-align.mjs
→ ALL CHECKS PASSED

kernel-loom（CI）success:
https://github.com/xhyperium/infra.rs/actions/runs/29818624366
https://github.com/xhyperium/infra.rs/actions/runs/29816437481
```

**Maintainer 证据（可追加）：**

```text
（粘贴你认可的 CI run / 复跑输出）
```

---

## L2 — API / SemVer

| 项 | Maintainer 勾选 | Agent 证据状态 |
|----|-----------------|----------------|
| `check-public-api.mjs` 通过 | [ ] | 已采集 — 5 package baselines match |
| baselines 落盘 | [ ] | 已核对 — `docs/api-baselines/{kernel,testkit,decimalx,canonical,contracts}.txt` |
| 0.x / `publish = false` 已知 | [ ] | 已核对 — workspace `0.3.0`，未宣称 crates.io |

**Agent 证据：**

```text
node scripts/quality-gates/check-public-api.mjs
public-api: kernel… OK
public-api: testkit… OK
public-api: decimalx… OK
public-api: canonical… OK
public-api: contracts… OK
public-api gate: OK (5 package(s), baselines match)
```

**Maintainer 证据：**

```text
```

---

## L3 — 平台与工具链

| 项 | Maintainer 勾选 | Agent 证据状态 |
|----|-----------------|----------------|
| 官方支持 = Linux x86_64 + MSRV 1.85 | [ ] | 已核对 — `docs/governance/support-matrix.md` |
| MSRV 1.85 在**当前基线**可构建 | [ ] | **待 Maintainer 确认** — 见下；勿仅凭 #98 历史 |
| Accept 非 Linux | [ ] | 已写入风险清单 — DEFER-6 Accept |

**Agent 说明（MSRV）：**

- 支持矩阵声明 MSRV **1.85**；`Cargo.toml` `rust-version = "1.85"`。  
- 本会话**未**用 `rustup run 1.85 cargo test` 复跑当前 `b0154db` 基线。  
- PR #98 合入时含 MSRV job 绿，但 **W1–W5 之后代码已变**，不能单独作为当前 MSRV 证据。  
- 请 Maintainer 粘贴**当前** `ci-rust` / MSRV job URL，或本地 `rust-version` 复跑摘要。

**Maintainer 证据：**

```text
（当前基线 MSRV 1.85 CI URL 或本地输出）
```

---

## L4 — 安全与供应链

| 项 | Maintainer 勾选 | Agent 证据状态 |
|----|-----------------|----------------|
| `cargo deny check` | [ ] | 已采集 — advisories/bans/licenses/sources ok；Zlib allowance 未命中警告 |
| 无密钥入仓 | [ ] | 本会话未发现；仍建议抽查 ignore 规则 |
| panicking ops 门禁 | [ ] | 已采集 — OK (0 hits) |

**Agent 证据：**

```text
cargo deny check
→ advisories ok, bans ok, licenses ok, sources ok
→ warning: Zlib license allowance unmatched（非阻断）

node scripts/quality-gates/check-decimal-no-panicking-ops.mjs
→ decimal panicking-ops gate: OK (33 files scanned, 0 hits)
```

**Maintainer 证据：**

```text
```

---

## L5 — 运维与回滚（**人签焦点 · 正式文件填写**）

| 项 | Maintainer 勾选 |
|----|-----------------|
| 回滚路径：squash PR 可 `git revert` | [ ] |
| DEFER Accept 清单已读 | [ ] |
| crates.io 不发布决策 | [ ] |
| **Signed-off-by / 结论** | [ ] **仅正式 `<version>-signoff.md`** |

---

## Accept 风险清单（签核时必须知悉）

| 项 | 说明 |
|----|------|
| 真实后端 | adapters **mock ≠** 生产 DB/MQ/交易所客户端（DEFER-1 Accept） |
| 二期 trait | ObjectStore / TimeSeries / Analytics / PubSub 深度合同未全量（DEFER-2 Accept） |
| 非 Linux | 未官方支持（DEFER-6 Accept） |
| Venue override | **运行时**门禁为主，非 compile-fail lint（DEFER-8 Close 口径） |
| fuzz | proptest/对抗 serde 轻量入口；**无**完整 `cargo fuzz` 靶场 |
| package stable | 未宣称 crates.io / Spec Goal Achieved |
| 部分 workflow 噪声 | 签核前 Maintainer 应扫一眼 Actions；本地五 crate 门禁 + public-api + loom 为 Agent 主证据 |

---

## Maintainer 签核区（**仅复制到正式文件后填写**）

> 下列表格保留在 DRAFT 中作提醒；**实际签名写在** `docs/plans/releases/<version>-signoff.md`。

| 字段 | 值（正式文件填写） |
|------|--------------------|
| 结论 | GO / NO-GO / GO-with-Accepts（**中性列举，无默认推荐**） |
| 姓名 / `@handle` | |
| 日期 | `YYYY-MM-DD` |
| 备注 | |

模板签名格式（与 [`prod-signoff-TEMPLATE.md`](../../governance/prod-signoff-TEMPLATE.md) 一致）：

```text
Signed-off-by: @________  YYYY-MM-DD
```

---

## Agent 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-21 | 初版 DRAFT（#138） |
| 2026-07-21 | 填充本地验证证据 + PR squash 表 |
| 2026-07-21 | 按 review 修正：去掉 GO 默认偏向；签名格式对齐模板；禁止 DRAFT 终签；MSRV 改待确认 |
