# SSOT 规则 — 单一事实源

> `.agents/ssot/` — 本仓库代理层与域规格的**本仓单一事实源**。
> 本文件是 SSOT 本身的 SSOT：域目录结构、落地判定与变更规则以此为准。

---

## 定义

**SSOT (Single Source of Truth)**：整个项目中，任何一个事实（数据、配置、规则、技能、域规格）有且仅有一个权威来源。其他位置必须是派生投影，不得分叉维护。

---

## SSOT 层级

```
源层（可编辑）
├── .claude/skills/         ← 技能定义
├── CONSTITUTION.md         ← 工程宪章
├── AGENTS.md               ← 多 Agent 协作规则
├── .github/                ← CI/CD
├── docs/                   ← 项目文档（含 docs/ssot/ 对齐矩阵）
└── .agents/ssot/           ← 域规格 SSOT 根（本仓）
    ├── kernel/
    ├── testkit/
    ├── types/
    ├── infra/              # bootstrap / configx / gate / observex / …
    ├── adapters/           # exchange + storage
    ├── contracts/
    └── tools/              # evidence / goalctl / xtask / verifyctl

投影层（只读派生）
└── .agents/skills/         ← 从 .claude/skills/ 投影
```

> **路径约定**：保留 `infra/`、`adapters/`、`tools/` 层级，勿展平到 `.agents/ssot/` 根，以免相对链接断裂。

---

## 规则

### R1: 源优先

- 任何变更必须先修改源层文件
- 禁止在投影层手工编辑

### R2: 投影同步

- 投影层由自动化脚本/钩子从源层生成
- 同步时机：源文件变更时 / 会话启动时 / 手动触发
- 同步失败必须显式报错，不得静默

### R3: 一致性校验

- CI 中验证投影层与源层一致
- 不一致时阻止合并
- 校验命令：`bd prime --stealth --readonly --hook-json`（Beads）或其他同步工具

### R4: 新增 SSOT

- 新增源文件前先声明在本文档中
- 同时创建对应的投影规则或对齐文档

### R5: 废除与迁移

- SSOT 源位置变更时，旧位置保留重定向说明（至少一个版本周期）
- 投影层与对齐文档必须同步更新

### R6: 本仓域规格树

- `.agents/ssot/{kernel,testkit,types,infra,adapters,contracts,tools}/` 是 **infra.rs 本仓** 的域规格 SSOT
- 禁止在 SSOT 树内写入 `src/`、`Cargo.toml`、`*.rs` 实现副本
- 禁止用 SSOT 文档中的 COMPLETE / Stable 叙事冒充 crate 已 ship
- 路径一律使用 `.agents/ssot/`（禁止旧 monorepo 单数 agent 路径写法）
- 域树变更经 worktree + PR 合入；不从外仓路径覆盖本树

### R7: 规格文档 ≠ 本仓实现

- SSOT 内 `review COMPLETE` / `Stable CLAIMED` / Phase Approved **仅**描述规格或历史战役状态
- 本仓是否落地以 `Cargo.toml` workspace members + `crates/` 路径 + 本仓测试为准
- **testkit**：本仓已落地 `crates/testkit`（`xhyper-testkit`）；`contract-testkit` 未落地
- **infra**：按各域对齐文；未在 members 中的域不得宣称落地
- **adapters 九域**：crate 为 scaffold，**未**宣称业务实现 / package stable
- **tools**：仅 `crates/evidence` 最小面落地；goalctl / xtask / verifyctl **未**落地
- 审计基线见 `docs/ssot/*-ssot-alignment.md` 与 `docs/ssot/workspace-ssot-alignment.md`

---

## 当前 SSOT 清单

| 事实域 | SSOT 位置 | 说明 |
|--------|----------|------|
| Agent 技能 | `.claude/skills/` | 投影至 `.agents/skills/` |
| 工程宪章 | `CONSTITUTION.md` | 直接引用 |
| Agent 行为 | `AGENTS.md` | 直接引用 |
| CI/CD | `.github/workflows/` | 直接引用 |
| 项目文档 | `docs/` | 直接引用 |
| 变更日志 | `CHANGELOG.md` | 直接引用 |
| Cargo 配置 | `.cargo/config.toml` | 直接引用 |
| Crate 规则 | `crates/AGENTS.md` | 按 crate 细化 |
| Kernel | `.agents/ssot/kernel/` | L0 规格；实现 `crates/kernel` |
| Testkit | `.agents/ssot/testkit/` | 规格；实现 `crates/testkit` |
| Types | `.agents/ssot/types/` | decimal / canonical |
| Infra | `.agents/ssot/infra/` | bootstrap / configx / gate / … |
| Adapters | `.agents/ssot/adapters/` | exchange + storage |
| Contracts | `.agents/ssot/contracts/` | trait 出口规格 |
| Tools | `.agents/ssot/tools/` | evidence / goalctl / xtask / verifyctl |
| SSOT 规则 | `.agents/ssot/SSOT.md` | 自引 |

---

## 版本

| 版本 | 日期 | 修订 |
|------|------|------|
| v2.0.0 | 2026-07-21 | **彻底本仓化**：`.agents/ssot` 不再表述为外仓镜像；清除外仓路径字面量；R6/R7/清单重写 |
| v1.6.0 | 2026-07-21 | 本仓化 tools SSOT（evidence/goalctl/xtask/verifyctl） |
| v1.5.0 | 2026-07-21 | 注册 contracts |
| v1.4.0 | 2026-07-21 | 注册 adapters |
| v1.3.0 | 2026-07-21 | infra 保留 `infra/` 层级 |
| v1.2.0 | 2026-07-21 | R7：规格≠实现；记录 testkit 落地 |
| v1.1.0 | 2026-07-21 | 添加 kernel/testkit/types 条目 |
| v1.0.0 | 2026-07-21 | 初始 SSOT 规则定义 |
