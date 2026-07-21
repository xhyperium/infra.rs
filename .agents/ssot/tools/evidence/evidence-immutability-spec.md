# evidence-immutability-spec.md — 指针

> **正文**：[`spec/evidence-immutability-spec.md`](spec/evidence-immutability-spec.md)
>
> 根路径保留文件名以免历史链接失效。布局镜像 [`xhyper-evidence-complete-spec.md`](xhyper-evidence-complete-spec.md)。

---

| 项 | 值 |
|----|-----|
| Spec ID | SPEC-EVIDENCE-IMMUTABILITY-DRAFT-001 |
| Title | infra.rs Evidence Immutability Spec |
| Status | **Draft**（未 Approved） |
| Created | 2026-07-19 |
| Owner | platform / security |
| 关联审计 | §7.P2 — Evidence Immutability Spec + Gate |
| 关联 PR | 本 worktree（`fix-audit-2026-07-19-followup`，隔离 PR #721） |
| 文档定位 | 指针；权威正文见上方链接 |

## 修复目标

响应审计 §7.P2：现有 [`main-first-contract`](../../../../docs/policies/MAIN_FIRST_POLICY.md)
仅保护 `evidence/**/*.contract.json` 的 append-only 语义，**不**覆盖历史 Evidence 真值
（`.log`、`.json`、`.md`）。PR #721（commit `acbd06ca4`）改写 138 份 `evidence/**/*.log`
后 `Harness Check` 仅报 `log hash mismatch`，仓库历史层缺正式不可改写不变量与门禁。

## 与 SPEC-EVIDENCE-002 的关系

**补充而非替代**。

- [`SPEC-EVIDENCE-002`](xhyper-evidence-complete-spec.md)（Approved 2026-07-14）规定
  Evidence 如何**生产**：canonical 编码、域分离 digest、chain 不变量、append durability、
  signed checkpoint、tail-truncation 检测等。
- 本 Spec（Draft）规定 Evidence **生产后不可改写**：已合入 `main` 的 Evidence 文件
  默认 immutable；`git diff --name-status <base>..HEAD` 中的 `M`/`D` 触发 FAIL；
  `A` 允许；路径迁移走 correction evidence + manifest，不替换原始内容。

两者协同：SPEC-EVIDENCE-002 在运行时形成 tamper-evident 链；本 Spec 在仓库历史层
形成 append-only 锁。运行时篡改会在 chain verify 阶段失败；仓库历史改写会在本 gate
阶段失败。

## 与 Main First Policy 的关系

Main First Policy §2 已规定 Change Contract 的 append-only；本 Spec 把同样的不可改写
约束延伸到 Evidence 文件本身（宪法第 25 条「main 是唯一正式状态」的反面：已进入 main
的 Evidence 不可被候选改写）。Policy 草案引用见
[`docs/policies/MAIN_FIRST_POLICY.md`](../../../../docs/policies/MAIN_FIRST_POLICY.md) §12。

## 路径说明

本 Spec 位于 `.agents/ssot/tools/evidence/`。PR #721（commit `629420b3f`，2026-07-19 合并）已把 `.agent/specs/` 重命名为 `.agents/ssot/`，本 Spec 随之迁移。路径迁移本身需正式 RFC + Maintainer approval + Risk Owner review（见下文 §路径迁移）；本 Draft 不假设路径迁移的合宪性，仅在事实层面记录当前位置。
