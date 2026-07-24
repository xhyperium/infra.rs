# Round 08 Findings — Completion Skeptic

| 字段 | 值 |
|------|-----|
| Round | **8** |
| Role | Verifier / Completion Skeptic |
| Scope | §33.1–33.6 勾选 ↔ `tasks.md` Task ID；§32 Evidence 模板；Forbidden 一致性；诚实状态（禁 Approved/stable）；假完成扫描 |
| Baseline | `xhyper-evidence-complete-spec.md` §32–§33 · `plan.md` §1/§8/§9 · `tasks.md` 映射表 · `approval-packet.md` |
| **result** | **FAIL** |

---

## failed_checks

### R-SPEC-003 — §33.1–33.6 每勾选映射到 Task ID（FAIL）

`tasks.md` 有「§33 勾选 → Task 映射」节，**表面覆盖**，逐条对规范 checkbox 后发现 **幽灵 ID / 过宽代理 / 无 ID**：

#### 33.1 规格闭合

| Checkbox（规范） | 映射 | 判定 |
|------------------|------|------|
| SPEC-EVIDENCE-002 Approved | `T-HUM-001` | OK（人审） |
| 旧 spec superseded | `T-DOC-002` + `T-HUM-002` | OK |
| ADR 冲突已修订 | `T-DOC-004` | **弱**：AC 是「修订**提案**」，不是「ADR Accepted/合并」；且只提 ADR-010，**ADR-012 未映射** |
| 路径和 package 对齐 | `T-CUT-002` `T-CUT-003` | 弱：无 package rename/双包策略 Task |
| architecture registry 对齐 | `T-REG-001` `T-REG-002` | OK 形 |
| evidence-policy.toml 已建立 | `T-POL-001` `T-POL-002` | OK |
| 无未登记安全 Unknown | `T-RES-001` `T-SKEP-001` | **断链**：`T-RES-001` TODO 且 **residual-open 文件不存在** |

#### 33.2 Core 闭合

| Checkbox | 映射 | 判定 |
|----------|------|------|
| crates/infra/evidence 已落地 | `T-CORE-001` | OK |
| canonical V1 冻结 | `T-CORE-014..027` | OK 范围 |
| 无字段拼接歧义 | `T-CORE-029` | OK |
| 无全零 genesis | `T-CORE-017` | OK |
| ChainId / sequence / EventId | `T-CORE-005/006` + `T-MEM-002` | OK（sequence 靠 mem） |
| recorded_at / event_time | `T-CORE-011` | OK |
| actor / **subject** 完整 | `T-CORE-008` `T-CORE-010` | **缺口**：仅 Actor + Draft；**无** §7.2 Subject 规范化/校验独立 Task |
| typed outcome | `T-CORE-009` | OK |
| 无 generic hash_bytes | `T-CORE-018` `T-CUT-004` | OK |
| 无 Debug/JSON hash | `T-DOM-001` `T-ARCH-004` | OK |
| 无 anyhow | `T-CORE-002` `T-ARCH-003` | OK |
| record 字段私有 | `T-CORE-012` `T-CORE-035` | OK |

#### 33.3 Adapter 闭合

| Checkbox | 映射 | 判定 |
|----------|------|------|
| memory 仅测试 | `T-MEM-004/007` `T-ARCH-005` | OK |
| file durable | `T-FILE-004` | OK |
| postgres/outbox atomic | `T-PG-003/004` | OK |
| 并发 conformance | `T-MEM-005` `T-FILE-009` `T-PG-007` | OK |
| idempotency | `T-MEM-002` `T-PG-005` | OK |
| crash recovery | `T-FILE-005/008` `T-PG-006` | OK |
| disk full / short write / fsync failure | `T-FILE-008` | **弱**：chaos 任务笼统，未列 disk-full/short-write 分项 AC |
| production 不降级 volatile | `T-MEM-004` `T-ARCH-005` | OK 形；缺 bootstrap 强制（见 R7） |

#### 33.4 Checkpoint 闭合

| Checkbox | 映射 | 判定 |
|----------|------|------|
| signed checkpoint | `T-CP-002` | OK |
| key rotation | `T-CP-006` | OK |
| independent anchor | `T-CP-005` | OK 合同接口 |
| tail truncation 可检测 | `T-CP-004` | OK |
| full chain replacement 可检测 | `T-CP-005 + verify` | **非 Task ID**；「+ verify」含糊，无 `T-CP-00x` 专测 |
| startup verify | `T-FILE-005` `T-CP-004` | **弱拼凑**：恢复 ≠ 启动强制 verify 合同 |

#### 33.5 测试闭合 — **高假完成区**

| Checkbox | 映射 | 判定 |
|----------|------|------|
| golden | `T-CORE-026/027` `T-CLI-004` | OK |
| property | `T-CORE-028/030` | OK |
| fuzz | **`T-CI-002`** | **FAIL 映射**：`T-CI-002` AC = 「nightly mutants/miri/fuzz **任务规划**；文档+workflow **草案**」≠ fuzz **通过** |
| line ≥95% | `T-CORE-033` | 弱：AC「门禁脚本**或记录**」可降级为只写文档 |
| branch ≥90% | **`T-CI-002`** | 同上，**仅规划** |
| mutation ≥90% | **`T-CI-002`** | 同上 |
| Miri | **`T-CI-002`** | 同上 |
| adapter chaos | `T-FILE-008` `T-PG-006` | OK 形 |
| historical schema verification | `T-LEG-002` `T-CI-002` | 弱 + 规划桶 |

**结论**：33.5 中 **fuzz / branch / mutants / Miri** 被塞进同一个「草案」Task，勾选表制造「已有 Task」假象，**不能**支撑 §33 完成定义。

#### 33.6 系统闭合 — **幽灵 Task**

| Checkbox | 映射 | 判定 |
|----------|------|------|
| required operations 登记 | `T-POL-002` | OK |
| required fail-closed | `T-DOM-005` | OK（域局部；非全系统 required 枚举） |
| Tier-A 事务/outbox/SoT | **`T-PG-004 T-ATOM via design`** | **FAIL**：**`T-ATOM` 不存在**于 tasks 表；「via design」= 无 AC 无 Owner |
| external side effects Attempted+terminal | **「订单域后续；policy 预留」** | **FAIL**：无 Task ID、无正式 `DEFER(accepted)` 记录 |
| source artifacts retention | `T-POL-002 + docs` | 弱；policy 任务未写 retention 天数 AC |
| verifier/schema/keys 保留期 | `T-CP-006 + docs` | **错配**：`T-CP-006` = key **rotation 测试**，非 retention 期 |
| CI Evidence 可追溯当前 commit | `T-EVID-SYS` `T-CI-001` | OK 形（未做） |

---

### R-EVID-001 — §32 Evidence 系统模板（PARTIAL → 记缺口）

plan §8 目录模板与规范 §32 **字段级一致**（manifest / logs / coverage / mutants / fuzz / golden-diff / conformance / recovery / public-api / schema-compat / threat-model / verdict）。

| 检查 | 结果 |
|------|------|
| 模板写在 plan | YES |
| 禁止项（自证/SKIP=PASS/旧 commit/手写 digest） | plan §8 有；与 §32 对齐 |
| 独立 golden verifier | 规范要求；`T-CLI-004` 部分覆盖 |
| 实例目录 `evidence/system/<date>-<change-id>/` | **不存在**（`evidence/` 下无 `system/`） |
| `T-EVID-SYS` | **TODO** |
| 每波强制产包 | gap 写「每波」；tasks 仅战役末 `T-EVID-SYS` — **粒度漂移** |

计划完备性层面：模板存在 = 部分满足。  
完成 skeptic：**无实例 + 无 per-wave Task = 不可勾 §32/33.6 CI Evidence**。

---

### R-FORBID-001 — Forbidden 清单一致性（FAIL）

三处 Forbidden / 禁令 **不等价**：

| 来源 | 条目摘要 |
|------|----------|
| **plan 页眉 Forbidden** | 假 §33 Done；registry stable；无 Evidence 勾 PASS；旧链静默重编码；mock 进生产 |
| **plan §1 原则** | 上表 + residual Unknown 禁；core 禁 anyhow/serde/tokio/uuid/chrono；人审闸；fail_rounds；分支纪律 |
| **approval §2 不可豁免** | 伪造/手写 PASS；链损坏→Invalid；生产 evidence_memory；无 domain hash_bytes；跳过 fail-closed；私钥进 core/仓 |

不一致点：

1. 页眉 **无**「链损坏→Invalid」「fail-closed」「私钥」「anyhow 进 core」。
2. approval **无**「旧链静默重编码」（在 A7 建议里，不在不可豁免表）。
3. `T-PLAN-001` AC 写「Forbidden 清单」— 若只验页眉，会 **漏** approval 不可豁免项。
4. 执行步骤无「违反 Forbidden → 自动 FAIL Wave」的机控 Task（依赖人读）。

**不矛盾执行步骤**方面：未发现「先允许 mock 进 core 再禁 mock」的直接反指令；但 **Article IX mock feature（ADR-010）vs 取消 core mock（A5/T-CORE-034）** 在治理未修订前，执行 W1 会与 Accepted ADR 冲突——属治理序，不是字面自相矛盾步骤。

---

### R-HONEST-001 — 诚实状态（PASS 主叙事 / 局部假完成）

**主叙事诚实（保留 PASS）**：

- Spec Status 写 **Proposed**，目标 Approved/Stable，**未**写成已 Approved。
- Campaign：**PLANNING / READY W0–W1** · **≠ Stable** · **≠ §33 闭合**。
- todo：§33 / registry stable / Core V1 = NO。
- 实现任务几乎全 **TODO**；未谎称 adapters/checkpoint 已落地。

**局部假完成 / 过早 DONE**：

| Task | Status | 问题 |
|------|--------|------|
| `T-PLAN-001`…`004` | **DONE** | 文件存在 → 文档 DONE 可接受 |
| `T-TODO-001` | **DONE** | `.worktrees/evidence-todo.md` 存在，但 **被 gitignore**，非仓内持久证据 |
| `T-PLAN-003` AC | 「覆盖 §33 **全部**勾选项」 | 映射表 **未真正全部**（T-ATOM、external、33.5 规划桶）→ **AC 过称** |
| `T-PLAN-002` | DONE | gap 有 §0–§34/DEF — OK |
| 任何实现/Approved/stable | — | **未错误声称** |

**禁止升级**：本轮 **不得** 因 plan 包 DONE 暗示战役前进到 W1 完成或 §33。

---

### residual-open 与路径假完整（FAIL 附属）

| 项 | 事实 |
|----|------|
| residual-open 文件 | **缺失**（`T-RES-001` TODO） |
| DEF 列表 | gap-matrix + gitignored todo only |
| `crates/adapters/evidence` | **不存在**；现树为 `crates/adapters/storage/*` + `exchange/*` |
| `crates/infra/evidence` | **不存在** |
| `evidence/system/` | **不存在** |
| 若把「tasks 写出了路径」当「路径已对齐」 | **false completeness** |

---

## omissions

1. **创建真实 Task**：`T-ATOM-*`（或删映射中的 `T-ATOM`）；`T-EXT-ATTEMPTED-*` 或正式 `DEFER(accepted)` 行。
2. **拆分 `T-CI-002`**：fuzz / branch-cov / mutants / miri 各 Task + AC=门槛数值，而非「草案」。
3. **Subject 完整**：`T-CORE-*` Subject 模型/策略版本任务。
4. **full replacement / startup verify** 独立 AC 与测试 Task。
5. **§32 per-wave** 最小 Evidence 包 Task（或明确仅战役末一次并改 gap「每波」措辞）。
6. **统一 Forbidden SSOT**（页眉 = approval 不可豁免并集）。
7. **residual-open.md**（或 plan 内 § residual）非 worktree 文件。
8. **`T-PLAN-003` AC 降级**：改为「映射表已建；未映射项 OPEN 列表」直至幽灵 ID 清零。
9. **ADR 修订闭合 Task**：`T-DOC-004` 提案 vs `T-HUM-ADR` Accepted 分离（33.1「已修订」）。

---

## false_pass_risks

| 模式 | 触发条件 | 后果 |
|------|----------|------|
| 映射表存在 ⇒ R-SPEC-003 PASS | 审查只数行不核 ID | 幽灵 `T-ATOM`、规划桶 33.5 漏网 |
| `T-CI-002` DONE（写了 workflow 草案） | 把草案当 mutants/miri 绿 | 假 §33.5 |
| `T-CORE-033`「或记录」 | 只提交 coverage 说明无阈值 | 假 line≥95% |
| `T-EVID-SYS` 手写 verdict.md | 违反 §32 禁手写 | 假战役闭合 |
| residual 只在 `.worktree` | clone 无 OPEN | 假「无 Unknown」 |
| plan 包 5×DONE | 对外说「evidence 计划已完成」= 实现完成 | 范围偷换 |
| 路径写在 tasks | `adapters/evidence` 未建勾 33.1 路径对齐 | 假路径 SSOT |
| registry `T-REG-002` DEFER | DEFER 被记 PASS | 假 stable 决策 |

---

## notes

### DONE 任务抽查（防空 TODO 标 DONE）

| ID | 声称 | 证据 | 裁定 |
|----|------|------|------|
| T-PLAN-001 | plan.md | 文件存在且含 Forbidden/Waves | DONE 可接受 |
| T-PLAN-002 | gap-matrix | §0–§34 + DEF + T1–T18 | DONE 可接受 |
| T-PLAN-003 | 覆盖 §33 全部 | 映射表有空洞 | **AC 不满足严格释义** → 建议改回 TODO 或改 AC |
| T-PLAN-004 | approval-packet | 人审闸清晰 | DONE 可接受 |
| T-TODO-001 | evidence-todo | 仅 worktree | **仓外**；战役证据价值低 |

实现类 Task：**无**错误标 DONE（好）。

### Forbidden 推荐并集（文档补丁用，本轮不改）

```text
- 假 §33 Done / 无命令或 Evidence 包勾 PASS
- registry stable / Spec Approved 由 AI 独断
- SKIP/DEFER 记 PASS
- 旧链静默重编码为 V1 并声称历史连续
- mock / evidence_memory 进生产装配
- 无 domain 的 hash_bytes / Debug|JSON hash
- 链损坏映射为普通 Invalid
- 跳过 required fail-closed
- 私钥进入 core 或仓库
- core 引入 anyhow/serde/tokio/uuid/chrono
```

### 诚实结论（强制）

```text
SPEC-EVIDENCE-002: Proposed
PLAN-EVIDENCE-002: planning package partially landed
§33: NOT closable
stable / 3/3 / 5/5: FORBIDDEN to claim
fail_rounds (plan 10x): not yet 0 (this round FAIL contributes)
```

### 证据指针

- 映射表：`/home/workspace/.agents/ssot/tools/evidence/plan/tasks.md`（§33 勾选 → Task）
- 规范勾选：`.../xhyper-evidence-complete-spec.md` §33.1–33.6
- §32 模板：spec #32；plan §8
- Forbidden：plan 页眉 + §1；approval §2
- 缺失 residual-open：`plan/` 目录仅有 approval-packet · gap-matrix · plan · tasks · 本 findings
- 适配器实树：`/home/workspace/infra.rs/crates/adapters/storage/`（无 `evidence/`）
- `evidence/system/`：不存在

---

## verdict_summary

```text
round: 8
result: FAIL
failed_checks:
  - R-SPEC-003 (§33 映射：T-ATOM 幽灵；external 无 Task；33.5 塞进 T-CI-002 草案；subject/startup/full-replacement 弱)
  - R-EVID-001 (模板有；evidence/system 实例无；每波 vs 战役末粒度漂移)
  - R-FORBID-001 (页眉 / §1 / approval 不可豁免 三套不等价)
  - residual-open 缺失 + adapters/evidence 路径仅纸面
passed_subchecks:
  - 主状态未伪称 Approved/stable
  - 实现 Task 未集体假 DONE
  - plan §8 与 §32 模板结构对齐
omissions: [见上节]
false_pass_risks: [见上节]
honest_status: Proposed / PLANNING / §33 open / residual-open ABSENT
```
