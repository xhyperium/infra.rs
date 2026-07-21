# 生产签核草稿 — PLAN-CORE-PROD-002 后（**未签核**）

> **状态：DRAFT · Agent 仅填充证据指针 · 禁止当作已签核**  
> Maintainer 请复制本文件字段到正式 `docs/plans/releases/<version>-signoff.md` 后手签。  
> 模板 SSOT：[`docs/governance/prod-signoff-TEMPLATE.md`](../../governance/prod-signoff-TEMPLATE.md)

---

## 元信息

| 字段 | 值 |
|------|----|
| 版本 / Tag | _待 Maintainer 指定（当前 workspace `0.3.0`，`publish = false`）_ |
| 关联 | PLAN-CORE-PROD-002 · beads `infra-asa` · PR #120 #121 #124 #125 #127 #128 |
| 支持矩阵 | [`support-matrix.md`](../../governance/support-matrix.md)：Linux x86_64 + MSRV 1.85 |
| 草稿日期 | 2026-07-21 |
| 签核人 | **空白 — 仅 Maintainer 填写** |
| 签核结论 | **空白 — GO / NO-GO 仅人类** |

---

## 红线（再声明）

```text
Maintainer only — Agent must not sign.
本文件是草稿，不是 Production Ready 批准。
```

---

## 已合入实现证据（Agent 整理）

| 波次 | PR | 合入摘要 |
|------|-----|----------|
| W0 | [#120](https://github.com/xhyperium/infra.rs/pull/120) | 计划 + artifacts 冻结 |
| W1 | [#121](https://github.com/xhyperium/infra.rs/pull/121) | decimalx oracle / 边界 / panicking 门禁 / scheduled miri·mutants |
| W2 | [#124](https://github.com/xhyperium/infra.rs/pull/124) | canonical committed wire v1.1–v1.3 |
| W3 | [#128](https://github.com/xhyperium/infra.rs/pull/128) | contracts 语义文档 + fakes + venue override 门禁 |
| W4 | [#125](https://github.com/xhyperium/infra.rs/pull/125) | adapters 离线 mock 验证入口 |
| W5 | [#127](https://github.com/xhyperium/infra.rs/pull/127) | support-matrix / public-api baselines / 签核模板 |

DEFER 表：[`../artifacts/defer-disposition.md`](../artifacts/defer-disposition.md)

---

## L1 — 正确性与不变量（建议勾选 · 待人确认）

- [ ] 核心 crate 测试在官方矩阵通过  
  **建议证据命令：**

  ```bash
  cargo test -p contracts -p decimalx -p canonical -p kernel -p testkit --all-targets
  cargo clippy -p contracts -p decimalx -p canonical -p kernel -p testkit --all-targets -- -D warnings
  ```

- [ ] decimalx：非法 scale 不可表示；checked 路径；oracle 差分见 `crates/types/decimal/tests/oracle_diff.rs`
- [ ] 无「整体 Production Ready」错误对外表述

**Maintainer 证据：**

```text
（粘贴 CI run URL / 本地输出）
```

---

## L2 — API / SemVer

- [ ] `node scripts/quality-gates/check-public-api.mjs`（需 nightly + cargo-public-api）
- [ ] baselines：`docs/api-baselines/{kernel,testkit,decimalx,canonical,contracts}.txt`
- [ ] 0.x / `publish = false` 策略已知

**Maintainer 证据：**

```text
```

---

## L3 — 平台与工具链

- [ ] 官方支持 = Linux x86_64 + MSRV 1.85（Accept 非 Linux）
- [ ] MSRV CI job 绿（合入 PR 历史可查）

**Maintainer 证据：**

```text
```

---

## L4 — 安全与供应链

- [ ] `cargo deny check`
- [ ] 无密钥进入版本库
- [ ] `node scripts/quality-gates/check-decimal-no-panicking-ops.mjs`

**Maintainer 证据：**

```text
```

---

## L5 — 运维与回滚（**人签焦点**）

- [ ] 回滚路径：squash PR 可 `git revert`；分支已删但 PR 可还原
- [ ] DEFER Accept 清单已读：真实后端、二期 trait、非 Linux
- [ ] 发布/不发布 crates.io 决策明确（当前 `publish = false`）
- [ ] **Signed-off-by / GO|NO-GO** — **仅人类**

---

## Accept 风险清单（签核时必须知悉）

| 项 | 说明 |
|----|------|
| 真实后端 | adapters mock ≠ 生产 DB/MQ/交易所客户端 |
| 二期 trait | ObjectStore / TimeSeries / Analytics / PubSub 深度合同未全量 |
| 非 Linux | 未官方支持 |
| Venue override | 运行时门禁为主，非 compile-fail lint |
| package stable | 未宣称 crates.io / Spec Goal Achieved |

---

## Maintainer 签核区（Agent 不得填写）

| 字段 | 值 |
|------|----|
| 结论 | GO / NO-GO / GO-with-Accepts |
| 姓名 / `@handle` | |
| 日期 | |
| 备注 | |
