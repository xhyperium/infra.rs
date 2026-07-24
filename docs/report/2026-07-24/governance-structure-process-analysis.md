# infra.rs 治理结构、规则与流程分析报告

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-24 |
| 类型 | 只读审计 / 治理诊断 |
| 范围 | 治理规则、治理结构、项目管理、项目流程、开发流程、业务流程 |
| 依据 | `docs/constitution/`、`.agents/rules/`、`AGENTS.md`、`CLAUDE.md`、`docs/ssot/`、`docs/plans/`、`docs/decisions/`、CI workflows |
| 结论摘要 | 规则体系完整且可强制执行能力强；层级过多、文档漂移、流程偏重，导致「治理成本 > 业务推进效率」风险明显 |

---

## 1. 治理结构（现状地图）

```text
组织层（xhyperium/.github rulesets）
  language / rust / agent-teams-constitution / agent-workflow / agent-quality-gates …
        ↓ 可加严，不可削弱
工程宪章（docs/constitution/ v1.8.0）
  使命 → 价值观 → 架构 → 代码标准 → 门禁 → 治理 → AI → 修订
        ↓ 落地细则
项目规则 SSOT（.agents/rules/）
  worktree / VERSIONING / 开发规则 / 语言 / 签核模板 / quant …
        ↓ 协作入口
AGENTS.md + CLAUDE.md + 多份 AGENTS.md（crate / tools / .claude / .codex）
        ↓ 域规格
.agents/ssot/**（规格）  ≠  crates/**（实现）
        ↓ 任务与证据
Beads(bd) + GitHub Issue/PR + docs/plans + docs/report + docs/decisions(DDR)
```

| 层级 | 权威路径 | 职责 |
|------|----------|------|
| 组织 P0 | `~/.claude/rules/`（外链 `xhyperium/.github`） | 语言、Rust、Agent 宪法 |
| 工程宪章 | `docs/constitution/` | 不可轻易改的主干原则 |
| 项目规则 | `.agents/rules/` | 实施细则 |
| 兼容 stub | `docs/governance/` | 仅重定向，非正文 |
| 域规格 | `.agents/ssot/` | 规格 SSOT，**≠ 已 ship** |
| 对齐审计 | `docs/ssot/*-alignment.md` | 镜像 vs 落地 |
| 决策 | `docs/decisions/DDR-*` | 架构/流程决策 |
| 发布签核 | `docs/plans/releases/*` + `prod-signoff-TEMPLATE` | L1–L5 人工签核 |

**优点：**

- 分层意图清楚
- `docs/governance/` 迁到 `.agents/rules/` 后，规则与 Agent 资产对齐
- worktree / 宪章脚本 / CI 有机器强制点

---

## 2. 治理规则（内容摘要）

### 2.1 硬规则（真正“管住”的）

| 域 | 要点 | 强制手段 |
|----|------|----------|
| Git Main First | 禁止 main 直开；PR + squash；30 天收敛 | 分支保护 + Agent 自律 |
| Worktree | 开发必须在 `.worktrees/<branch>` | `pre-tool-check` **BLOCK** |
| 质量门禁 | fmt / clippy -D / test / deny / 宪章脚本 | CI + `make ci` |
| 依赖 | workspace 集中 `workspace = true` | `check-workspace-deps.mjs` |
| 版本 | crate **独立 version**，交付默认 PATCH +1 | `check-crate-versions.mjs` |
| 语言 | 人类可读文本强制中文 + UTF-8 | 宪章 + 组织 `language.md` |
| AI 边界 | 不可 approve/merge/push main；不可改 CODEOWNERS | 宪章 §7 + Ruleset |
| 生产签核 | L1–L5 仅 Maintainer 手签 | 模板红线 + 技能约束 |

### 2.2 项目开发规则（`.agents/rules/项目开发规则.md`）

- 文档：`docs/report/{YYYY-MM-DD}/`
- 复用：DEV-001 / 002 / 003（先检索内部库、重复抽象、复用模块零业务依赖）
- 提交：Conventional Commits + 中文简述
- 门禁：四条本地命令清单

### 2.3 组织 Agent 规则（会话上下文额外叠加）

组织级 `agent-teams-constitution`、双审、Solo/Codex 路由、证据三关卡、Kill Switch 等与本仓宪章**并行生效**，但并非全部在本仓 CI 中落地。

---

## 3. 项目管理规则

| 机制 | 角色 | 问题信号 |
|------|------|----------|
| **Beads (`bd`)** | 跨 Agent 任务板（DDR-006） | 与 GitHub Issue/PR 双轨；local Dolt + remote sync 复杂度高 |
| **GitHub PR** | 变更唯一合入路径 | `dismiss_stale_reviews_on_push` 导致 rebase 后重审成本高 |
| **CODEOWNERS** | 所有权 | 与 `pr-auto-approve`（第二账号代批）并存，边界需纪律 |
| **plans/** | 多波次可执行计划 | 状态文案密集（DONE / SIGNED / SUPERSEDED） |
| **report/** | 审计与多轮 review | 证据丰富但体积膨胀 |
| **STATUS.md** | 自动进度看板 | 与 SSOT 对齐文、多份 review 结论可能不一致 |

任务生命周期（`AGENTS.md`）：

```text
接收 → 分析 → 分解 → 执行 → 验证 → 交付
         bd claim          worktree+PR     fmt/clippy/test     bd close
```

---

## 4. 项目 / 开发 / 业务流程

### 4.1 标准开发流程（规范路径）

```text
fetch main
  → worktree create feat|fix|...
  →（复杂任务）plan / beads claim
  → TDD / 实现
  → 本地 make ci + 相关 crate 测试
  → commit（Conventional + 中文）
  → PR → 审查 → CI 全绿 → squash merge
  → 清理 worktree/分支
  → 文档/SSOT 对齐更新（触及时）
```

### 4.2 生产就绪 / 发布流程（本仓的“业务主流程”）

本仓是**基础设施库工作区**，业务不是交易闭环，而是：

```text
规格(.agents/ssot)
  → 实现(crates)
  → 对齐审计(docs/ssot)
  → 多轮 review / gap 矩阵
  → 生产就绪 plan
  → L1–L5 签核(仅 Maintainer)
  → 内部 tag / 可选发布
```

明确红线（文档反复强调）：

- **规格 COMPLETE ≠ 本仓 ship**
- **exchange 交易 = NO-GO**
- **storage 多项 Cluster/HA/EOS 仍 NO-GO**
- **禁止宣称 workspace Production Ready / package stable（未签核时）**

### 4.3 验证 / 标准文档流程

- 每 crate 一份 `标准.md`（17 项标准模板）
- `docs/standards/验证流程.md`：markdownlint / cspell / fence 检查
- **缺口**：文档自承 `check.mjs` **尚未**集成 fence 检查 → 规范写了、门禁未闭环

---

## 5. 存在的问题

### 5.1 P0 — 权威源冲突 / 事实漂移（会直接误导 Agent 与人类）

| # | 问题 | 证据 |
|---|------|------|
| **P0-1** | **架构真相过时** | 宪章 `03-architecture` 只画 kernel/testkit/types；`ARCHITECTURE.md` 仍写 configx 旧路径、「workflows 6 个」、scripts 根路径；实际已有 `crates/infra/*`、adapters×9、tools、约 35 个 workflow |
| **P0-2** | **包名叙事不一致** | `AGENTS`/`CLAUDE` 仍写 `xhyper-evidence` / `xhyper-contracts` / `xhyper-transportx`；`Cargo.toml` 实际为 `evidence` / `contracts` / `transportx`；宪章 §4.3 写 **禁止 `xhyper-` 前缀** |
| **P0-3** | **SSOT 自描述滞后** | `.agents/ssot/SSOT.md` 仍把 `CONSTITUTION.md` 标为“源层正文”；宪章正文已在 `docs/constitution/`；R6 条目有重复粘贴 |
| **P0-4** | **docs/README 内部矛盾** | 前文已声明规则 SSOT 在 `.agents/rules/`，后文 status 小节仍写「规则类文档请放在 `governance/`」 |

**影响：** Agent 上下文同时注入过期与正确描述时，容易“按错地图施工”，属于治理最高风险。

### 5.2 P1 — 治理过载 / 流程过重

| # | 问题 | 说明 |
|---|------|------|
| **P1-1** | **规则叠床架屋** | 组织 agent 宪法 + 本仓宪章 + AGENTS + CLAUDE + 多 crate AGENTS + skills + beads 块；会话启动成本极高 |
| **P1-2** | **10 轮 review 文化常态化** | `docs/report/**/round-0x` 等大量轮次产物；审查深度可贵，但边际收益递减、文档维护成本爆炸 |
| **P1-3** | **每 crate 17 项标准文档** | 24 份 `标准.md` 强模板；adapter 多为 NO-GO/部分能力时，易变成填空合规 |
| **P1-4** | **门禁叙事分散** | 宪章 §5、`项目开发规则`、AGENTS、CI per-crate job 多套清单；新人不知「最小合入集」 |
| **P1-5** | **审查作废成本高** | §6.0.7 `dismiss_stale_reviews_on_push` + land 默认 rebase force-push → 合法同步常触发重审 |
| **P1-6** | **任务系统双轨** | Beads 强制 vs GitHub Issue/PR 合入主载体；同步与 profile 认知负担大 |

### 5.3 P2 — 边界模糊 / 纪律与工具打架

| # | 问题 | 说明 |
|---|------|------|
| **P2-1** | **AI 不可 approve vs pr-auto-approve 技能** | 第二维护者 token 代批，非 self-approve，但可被 Agent 批量调用，削弱“人类审查”语义 |
| **P2-2** | **enforce_admins 生产=false** | 管理员应急绕过存在，事后审计协议偏弱 |
| **P2-3** | **语言政策历史包袱** | 组织与 §4.5 强制中文；§4.6 STE 可选；`CLAUDE.md` 仍易读成“英文技术文档用 STE” |
| **P2-4** | **报告路径规则已破例** | 约定仅 `docs/report/{YYYY-MM-DD}/`，仍存在 `YYYY-MM-DD-slug` 历史目录（规范自我例外） |
| **P2-5** | **验证流程未完全进门禁** | 标准文档 fence 检查未进 `check.mjs` |
| **P2-6** | **specs 与实现长期半对齐** | gate/testkitx/xtask 等仅镜像；COMPLETE 误读为已交付的风险持续存在 |

### 5.4 P3 — 可维护性 / 产品化缺口

| # | 问题 | 说明 |
|---|------|------|
| **P3-1** | **命名规则未完全统一** | `*x` 后缀“推荐” vs kernel/testkit/contracts 等无 `x`；缺冻结对照表 |
| **P3-2** | **业务价值路径不清晰** | 对外「何时可依赖哪个 crate 哪一级能力」依赖读多份 alignment + GO/NO-GO |
| **P3-3** | **多工具链并行** | Claude / Codex / Copilot / OMC / gstack / beads / make… 日常选型仍模糊 |
| **P3-4** | **文档生成物与手写混放** | `STATUS.md`、CI 矩阵 generated 与人工叙事并存，过期风险高 |

---

## 6. 问题根因

```text
1. 快速演进（crates 布局、SSOT 展平/再归组、规则迁移）
   → 多份入口文档未同步更新

2. Agent 优先治理（钩子、双审、证据、多轮 review）
   → 对人类与小改动过重；文档成为主产物之一

3. 组织规则 + 本仓规则双 SSOT
   → 正确但认知负担高；缺少精简的「本仓生效摘要」

4. 规格镜像仓基因残留
   → SSOT 大树 + COMPLETE 叙事；落地裁定靠人工纪律
```

---

## 7. 流程健康度速评

| 维度 | 评分（主观） | 说明 |
|------|--------------|------|
| 主干保护 / 合入路径 | ★★★★★ | Main First + 分支保护清晰 |
| 机器强制 worktree | ★★★★★ | 硬门禁是本仓亮点 |
| 规则 SSOT 清晰度 | ★★★☆☆ | 结构对了，入口文案仍漂移 |
| 架构文档新鲜度 | ★★☆☆☆ | 宪章第三章 / ARCHITECTURE 明显滞后 |
| 开发摩擦（日常小改） | ★★☆☆☆ | worktree + 门禁 + 重审 + beads 叠加重 |
| 生产签核严肃性 | ★★★★☆ | 模板与红线好；执行依赖 Maintainer 带宽 |
| 规格→实现闭环 | ★★★☆☆ | 对齐文完善，OPEN/NO-GO 面仍大 |
| 任务系统统一性 | ★★☆☆☆ | bd + GH + plans 三套语义 |

---

## 8. 改进建议

### 8.1 立刻（止损漂移）

1. **刷新单一架构真相**：更新 `docs/constitution/03-architecture.md` + `ARCHITECTURE.md`（members、依赖图、路径、workflow 数量）。
2. **统一包名表述**：以 `cargo metadata` 为 SSOT，改掉 `AGENTS`/`CLAUDE` 中 `xhyper-*` 残留。
3. **修 docs/README 与 SSOT.md**：去掉“规则在 governance/”、CONSTITUTION 源层误述；删 R6 重复段。

### 8.2 短期（降载）

4. **写一页「本仓生效规则摘要」**（≤2 屏）：合入最小门禁、worktree 开工、禁止事项、签核红线；其余外链。
5. **定义审查档位**：trivial / standard / production-ready；禁止默认 10 轮；仅 L5/交易相关升档。
6. **明确「合入前必跑」命令集**为唯一清单（与 CI required checks 对齐表）。
7. **约束 pr-auto-approve**：仅用户显式要求 + 审计日志；禁止 Agent 默认调用。

### 8.3 中期（产品化治理）

8. **Consumer 支持矩阵一页纸**：每个 package 的支持级（L1 内部 / NO-GO 面 / live 条件）。
9. **任务系统选型固化**：要么 “bd 为执行、GH 为公开”，要么合并叙事，避免双写义务。
10. **规格树瘦身策略**：未 member 域折叠为 OOS index，降低 COMPLETE 误读面。

---

## 9. 结论

| 判断 | 内容 |
|------|------|
| 治理是否“有” | **有，而且偏完整**：Main First、worktree 硬门禁、版本/依赖/签核、中文与 Rust 上位标准都齐 |
| 治理是否“好用” | **中等偏下**：入口多、真相漂移、审查与文档过重，Agent 与人类都容易迷失 |
| 最大风险 | **文档与实现/包名/架构不同步**，加上 **流程重量**，导致错误决策或虚假“已生产就绪”叙述 |
| 最大优势 | 机器强制（worktree/门禁）+ 明确的 NO-GO/签核红线，方向正确 |

---

## 10. 参考路径

| 类型 | 路径 |
|------|------|
| 宪章索引 | `docs/constitution/README.md` |
| 治理（Git / 变更） | `docs/constitution/06-governance.md` |
| AI 代理 | `docs/constitution/07-ai-agents.md` |
| 项目规则 SSOT | `.agents/rules/README.md` |
| 开发规则总览 | `.agents/rules/项目开发规则.md` |
| Worktree | `.agents/rules/worktree-policy.md` |
| 文档组织 | `.agents/rules/文档组织约定.md` |
| 生产签核模板 | `.agents/rules/prod-signoff-TEMPLATE.md` |
| SSOT 规则 | `.agents/ssot/SSOT.md` |
| Workspace 对齐 | `docs/ssot/workspace-ssot-alignment.md` |
| 决策索引 | `docs/decisions/README.md` |
| Agent 入口 | `AGENTS.md` / `CLAUDE.md` |

---

## 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-24 | 初版：会话只读分析落盘 |
