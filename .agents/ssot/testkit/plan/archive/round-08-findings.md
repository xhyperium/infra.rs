# Round 8 — §17–§20 文档要求 / 版本稳定性 / 迁移 Phase0–6 / PR-1…6

| 字段 | 值 |
|------|-----|
| Scope | 计划完备性（非实现验收） |
| Spec | `SPEC-TESTKIT-002` §17–§20 |
| 日期 | 2026-07-14 |

## 检查项

| # | 规范点 | 计划应覆盖内容 |
|---|--------|----------------|
| 8.1 | §17.1 README | 8 bullet 全点 → Task |
| 8.2 | §17.2 AGENTS | 7 bullet 全点 → Task |
| 8.3 | §17.3 CHANGELOG | 宏退役/Fixture/provider/V2/layer |
| 8.4 | §18.1 status/layer | incubating 保持；layer→test-support；禁 premature stable |
| 8.5 | §18.2 版本与删除流程 | 0.1.0→0.1.1；RFC+清单+迁移+CHANGELOG+API diff+兼容决策 |
| 8.6 | §18.3 publish | `publish=false` + API snapshot |
| 8.7 | §19 Phase 0–6 | 每阶段有 Wave/Task；冻结 rg；终态 active path |
| 8.8 | §20 PR-1…6 | 切分内容与退出条件；worktree/禁止 main 等纪律 |

## PASS

1. **§17.1/17.2/17.3 有 Task 挂钩**：
   - `T-ARCH-008` README「§17.1 全点」+ **I-DOC-README**
   - `T-ARCH-009` AGENTS「§17.2 全点」+ **I-DOC-AGENTS**
   - `T-ARCH-010` CHANGELOG 宏退役/V2/layer + **I-DOC-CL**
   - `T-DEL-006` 删除后更新 README/AGENTS 去宏职责
2. **§18.1**：I-VER-STATUS incubating until 闭合；I-VER-LAYER test-support；`T-ARCH-001` layer；plan Forbidden「registry stable 而 §24 未全勾」；A8 DEFER stable。
3. **§18.2 bump**：目标 0.1.0→0.1.1 贯穿 plan 页眉 / todo / `T-HUM-002`；删除路径依赖 PR-3/4 + CHANGELOG + API snapshot（`T-GATE-004`）+ 人审 A3/A4。
4. **§18.3**：`T-ARCH-011` publish=false 显式；I-VER-PUB。
5. **§19 Phase ↔ Wave 映射清晰**（plan §1.4 + **I-PHASE**）：

   | Phase | Wave | 代表 Task |
   |-------|------|-----------|
   | 0 冻结 | W0 | T-FREEZE-001 · T-INV-001 · T-DOC-* |
   | 1 Clock V2 | W1 | T-CLK-* |
   | 2 迁移 | W2 | T-MIG-* |
   | 3 删无价值 API | W3 | T-DEL-001…003 |
   | 4 contract suites | W4 | T-CTC-* · T-DEL-004 |
   | 5 架构对齐 | W5 | T-ARCH-* |
   | 6 防回流 | W6 | T-GATE-* |

6. **Phase 0 冻结动作**：T-FREEZE-001 禁新增宏/placeholder/normal dep；T-INV-001 消费者扫描（对齐 §19.1 rg + metadata 意图）。
7. **Phase 1–3 API 迁移故事完整**：V2 先增后 deprecated（T-CLK-013）→ 迁移（T-MIG）→ 删宏（T-DEL）；与 §19.2–19.4 一致。
8. **Phase 4 Binance/OKX**：T-CTC-013/014；dev-dep contract-testkit 方向正确。
9. **§20 PR-1…6**：plan §1.2 六段 PR 内容/退出与规范 §20 对齐；todo §4 同表；独立 worktree / 禁 main / 不混 evidence 在 plan §1.1/§1.2。
10. **PR 纪律**：plan Forbidden #10；T-BRANCH-001。

## FAIL

### F8-1 — §19.6 终态权威路径未强制为 `spec/spec.md`

- **规范引用**：§19.6「删除双重 spec…**保留一个活动权威：`.agent/SSOT/testkit/spec/spec.md`**」；历史 Superseded 或 archive。
- **缺失**：
  - `T-ARCH-007` AC 写成「**complete-spec 或** `spec/spec.md`」——**放宽了规范强制路径**；
  - 无 Task：创建/迁移 `spec/spec.md`、将 complete-spec 降为 historical 或 move；
  - `T-DOC-002/003` 仅 Superseded 页眉，**未要求删除或 archive 双重 active-looking 文件**（规范用词「删除双重 spec」）；
  - inventory **无 I-DOC-ACTIVE-PATH** 固定终态路径字符串。
- **建议补丁**：
  1. 修正 `T-ARCH-007`：终态 active = `.agent/SSOT/testkit/spec/spec.md`（或显式 RFC 修改规范路径后再改 Task）。
  2. 增加 `T-DOC-004`：complete-spec → 迁入 `spec/spec.md` 或声明 redirect；旧 testkit-spec/testkitx → archive。
  3. I-METRICS `active_testkit_spec_count=1` 的判定脚本路径写死到终态文件。

### F8-2 — §18.2 宏/placeholder 删除「Approved RFC」步骤未任务化

- **规范引用**：§18.2 宏和 placeholder 删除必须：Approved RFC · 消费者清单 · 同批迁移 · CHANGELOG · public API diff · compatibility decision。
- **缺失**：
  - 有消费者清单（T-INV-001）、迁移/删除 Task、CHANGELOG、API snapshot、A3 兼容决策；
  - **无「撰写/归档 RFC」Task**；仅 `T-ARCH-006` 修订 ADR-010 备注 + `T-HUM-001` Spec Approved；
  - 若将 complete-spec 视为 RFC，计划未写明「Spec Approved ≡ §18.2 RFC」的等价关系 → 实现者可能跳过书面 RFC。
- **建议补丁**：
  1. 在 approval-packet 增加 **A11**：§18.2 删除 RFC 等价物 = SPEC-TESTKIT-002 Approved（或独立 RFC 路径）。
  2. 或新增 `T-DOC-RFC-DEL`：删除宏/placeholder 的 RFC/ADR 条目在 PR-4 前 Approved。

### F8-3（次要）— §17 bullet 未在 inventory 逐条展开

- **规范引用**：§17.1 八条、§17.2 七条。
- **缺失**：I-DOC-README/AGENTS 写「全部 bullet」而非 I-DOC-README-1…；与 plan §1.3 严格解读略松。
- **建议补丁**：可选展开；`T-ARCH-008/009` 已写「全点」可接受为 **PASS 倾向**，本条不计入主 fail。

### F8-4（次要）— Phase 5 STRUCTURE/TECH「删除」vs「标滞后」

- **规范引用**：§19.6 修改 STRUCTURE.md 等。
- **现状**：`T-ARCH-005`「对齐或标滞后」——允许滞后标注，与架构修复计划诚实原则一致；**不**算 FAIL，记观察项。

## 本轮结论：FAIL

## fail_count: 2

> 主 FAIL：F8-1、F8-2。PR 切分与 Phase0–6 Wave 映射整体扎实。
