# SSOT 规则 — 单一事实源

> `.agents/ssot/` — 本仓库宏观数据域规格的**本仓单一事实源**。
> 本文件是 SSOT 本身的 SSOT：域目录结构、落地判定与变更规则以此为准。

---

## 定义

**SSOT (Single Source of Truth)**：整个项目中，任何一个事实（数据、配置、规则、技能、域规格）有且仅有一个权威来源。其他位置必须是派生投影，不得分叉维护。

---

## SSOT 层级

```
源层（可编辑）
├── .claude/skills/         ← 技能定义
├── AGENTS.md               ← 多 Agent 协作规则
├── .github/                ← CI/CD
├── docs/                   ← 项目文档
└── .agents/ssot/           ← 域规格 SSOT 根（本仓）
    ├── domain_macro/       ← 核心宏观数据域
    ├── bea/                ← BEA 数据源适配器
    ├── eastmoney/          ← 东方财富数据源适配器
    ├── ecb/                ← ECB 数据源适配器
    ├── fred/               ← FRED 数据源适配器
    ├── japan_cb/           ← 日本央行数据源适配器
    ├── jin10/              ← 金十数据源适配器
    ├── treasury/           ← 美国财政部数据源适配器
    ├── uk_cb/              ← 英国央行数据源适配器
    ├── yahoo/              ← Yahoo 财经数据源适配器
    ├── yield_curve/        ← 收益率曲线统一契约（来源无关 kernel）

治理文档层（allowlist 管理）
├── docs/ssot/              ← 治理文档源，不复制域规格正文
└── .agents/skills/         ← 从 .claude/skills/ 生成的只读投影
```

---

## 规则

### R1: 源优先

- 任何变更必须先修改源层文件
- `docs/ssot/` 作为治理文档源由 allowlist 管理；`.agents/skills/` 禁止手工编辑，由同步脚本重建

### R2: 投影同步

- `.agents/skills/` 由 `scripts/skills/sync-skill-projection.mjs` 从 `.claude/skills/` 单向生成；`docs/ssot/` 是经过 allowlist 管理的治理文档源
- 同步时机：源文件变更时 / 会话启动时 / 手动触发
- 同步失败必须显式报错，不得静默

### R3: 一致性校验

- CI 中验证 manifest、域层、状态和投影边界一致
- 不一致时阻止合并
- 校验命令：`node scripts/quality-gates/check-ssot.mjs`；Beads 仅管理任务，不替代 SSOT 校验

### R4: 新增 SSOT

- 新增域或层前先更新 `.agents/ssot/manifest.json`
- 同时创建对应的投影规则或对齐文档

### R5: 废除与迁移

- SSOT 源位置变更时，旧位置保留重定向说明（至少一个版本周期）
- 投影层与对齐文档必须同步更新

### R6: 本仓域规格树

- `.agents/ssot/core/domain_macro/` + 各数据源适配器 是 **infra.rs 本仓** macro_data 域规格 SSOT
- 禁止在 SSOT 树内写入 `src/`、`Cargo.toml`、`*.rs` 实现副本
- 禁止用 SSOT 文档中的 COMPLETE / Stable 叙事冒充 crate 已 ship
- `spec_status` 与 `implementation_status` 必须分开；`draft` 规格不提供生产合同
- 路径一律使用 `.agents/ssot/`（禁止旧 monorepo 单数 agent 路径写法）
- 域树变更经 worktree + PR 合入；不从外仓路径覆盖本树

### R7: 规格文档 ≠ 本仓实现

- SSOT 内 `review COMPLETE` / `Stable CLAIMED` / Phase Approved **仅**描述规格或历史战役状态
- 本仓是否落地以 `Cargo.toml` workspace members + `crates/` 路径 + 本仓测试为准

---

## 当前 SSOT 清单

| 事实域 | SSOT 位置 | 说明 |
|--------|----------|------|
| Agent 技能 | `.claude/skills/` | 投影至 `.agents/skills/` |
| Agent 行为 | `AGENTS.md` | 直接引用 |
| CI/CD | `.github/workflows/` | 直接引用 |
| 项目文档 | `docs/` | 直接引用 |
| 变更日志 | `CHANGELOG.md` | 直接引用 |
| Cargo 配置 | `.cargo/config.toml` | 直接引用 |
| 宏观数据域 | `.agents/ssot/domain_macro/` | 核心宏观数据域规格 |
| 数据源适配器 | `.agents/ssot/{bea,eastmoney,ecb,fred,japan_cb,jin10,treasury,uk_cb,yahoo}/` | 宏观经济数据源适配器 |
| 统一契约 kernel | `.agents/ssot/{domain_macro,yield_curve}/` | 来源无关的宏观指标与收益率曲线模型 |
| SSOT 规则 | `.agents/ssot/SSOT.md` | 自引 |
| SSOT 结构与状态 | `.agents/ssot/manifest.json` | 机器可读源；由 `check-ssot.mjs` 校验 |

---

## 版本

| 版本 | 日期 | 修订 |
|------|------|------|
| v1.0.0 | 2026-07-22 | **macro_data 域初始化**：清空上一项目遗留 SSOT，建立宏观数据域规格体系 |
| v1.1.0 | 2026-07-22 | **生产级治理补强**：统一 13 层结构，分离规格/实现状态，增加 manifest 与 fail-closed 门禁 |
