# SSOT 规则 — 单一事实源

> .agents/ssot/ — 定义本仓库代理层（Agent Layer）的单一事实源规则。
> 本文件是 SSOT 本身的 SSOT：所有投影、同步、缓存行为必须以此为准。

---

## 定义

**SSOT (Single Source of Truth)**：整个项目中，任何一个事实（数据、配置、规则、技能）有且仅有一个权威来源。所有其他位置必须是从 SSOT 派生的只读投影。

---

## SSOT 层级

```
源层（可编辑）
├── .claude/skills/         ← 技能定义的 SSOT
├── CONSTITUTION.md         ← 工程宪章的 SSOT
├── AGENTS.md               ← 多 Agent 协作规则的 SSOT
├── .github/                ← CI/CD 的 SSOT
├── docs/                   ← 项目文档的 SSOT
└── .agents/ssot/           ← 上游 hyperware SSOT 镜像

上游 hyperware 源 (xhyper.rs)
├── .agent/SSOT/kernel/     ← xhyper kernel 规格 SSOT
├── .agent/SSOT/testkit/    ← xhyper testkit 规格 SSOT
└── .agent/SSOT/types/      ← xhyper 类型系统 SSOT

投影层（只读，自动生成）
├── .agents/skills/         ← 从 .claude/skills/ 投影
├── .agents/ssot/kernel/    ← 从 xhyper.rs/.agent/SSOT/kernel/ 镜像
├── .agents/ssot/testkit/   ← 从 xhyper.rs/.agent/SSOT/testkit/ 镜像
└── .agents/ssot/types/     ← 从 xhyper.rs/.agent/SSOT/types/ 镜像
```

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
- 同时创建对应的投影规则和同步脚本

### R5: 废除与迁移
- SSOT 源位置变更时，旧位置保留重定向说明（至少一个版本周期）
- 投影层必须同步更新

### R6: 上游 hyperware 镜像
- `.agents/ssot/kernel/`、`.agents/ssot/testkit/`、`.agents/ssot/types/` 是 `xhyper.rs/.agent/SSOT/` 的只读镜像
- 镜像更新命令：
  ```bash
  cp -rf /home/workspace/xhyper.rs/.agent/SSOT/kernel  .agents/ssot/
  cp -rf /home/workspace/xhyper.rs/.agent/SSOT/testkit .agents/ssot/
  cp -rf /home/workspace/xhyper.rs/.agent/SSOT/types   .agents/ssot/
  ```
- 禁止在镜像副本中直接编辑
- 上游变更后需手动执行镜像同步

---


### R7: 镜像文档 ≠ 本仓实现

- 镜像内 `review COMPLETE` / `Stable CLAIMED` 描述的是**上游 xhyper.rs 战役状态**
- 本仓是否落地以 `Cargo.toml` workspace members + `crates/` 路径为准
- **testkit**：本仓已落地 `crates/testkit`（`xhyper-testkit`）；`contract-testkit` 未落地
- 审计基线见 `docs/testkit-ssot-alignment.md`

---

## 当前 SSOT 清单

| 事实域 | SSOT 位置 | 投影位置 | 同步方式 |
|--------|----------|---------|---------|
| Agent 技能 | `.claude/skills/` | `.agents/skills/` | 脚本投影 |
| 工程宪章 | `CONSTITUTION.md` | — | 直接引用 |
| Agent 行为 | `AGENTS.md` | — | 直接引用 |
| CI/CD | `.github/workflows/` | — | 直接引用 |
| 项目文档 | `docs/` | — | 直接引用 |
| 变更日志 | `CHANGELOG.md` | — | 直接引用 |
| Cargo 配置 | `.cargo/config.toml` | — | 直接引用 |
| Crate 规则 | `crates/AGENTS.md` | `crates/*/AGENTS.md` | 按 crate 细化 |
| 宪章合规 | `scripts/check-constitution.sh` | — | 直接引用 |
| xhyper Kernel | `xhyper.rs/.agent/SSOT/kernel/` | `.agents/ssot/kernel/` | 手动镜像 (`cp -rf`) |
| xhyper Testkit | `xhyper.rs/.agent/SSOT/testkit/` | `.agents/ssot/testkit/` | 手动镜像 (`cp -rf`) |
| xhyper Types | `xhyper.rs/.agent/SSOT/types/` | `.agents/ssot/types/` | 手动镜像 (`cp -rf`) |
| SSOT 规则 | `.agents/ssot/SSOT.md` | — | 自引 |

---

## 版本

| 版本 | 日期 | 修订 |
|------|------|------|
| v1.2.0 | 2026-07-21 | R7：镜像≠实现；记录 testkit 本仓落地 |
| v1.1.0 | 2026-07-21 | 添加 xhyper.rs kernel/testkit/types 上游 SSOT 镜像条目 |
| v1.0.0 | 2026-07-21 | 初始 SSOT 规则定义 |
