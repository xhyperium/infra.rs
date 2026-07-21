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
└── .agents/ssot/           ← 上游 hyperware SSOT 镜像根

上游 hyperware 源 (xhyper.rs)
├── .agent/SSOT/kernel/     ← xhyper kernel 规格 SSOT
├── .agent/SSOT/testkit/    ← xhyper testkit 规格 SSOT
├── .agent/SSOT/types/      ← xhyper 类型系统 SSOT
├── .agent/SSOT/infra/      ← xhyper infra 平面（bootstrap/configx/gate/…）
└── .agent/SSOT/adapters/   ← xhyper adapters（exchange + storage）

投影层（只读，自动生成）
├── .agents/skills/         ← 从 .claude/skills/ 投影
├── .agents/ssot/kernel/    ← 从 xhyper.rs/.agent/SSOT/kernel/ 镜像
├── .agents/ssot/testkit/   ← 从 xhyper.rs/.agent/SSOT/testkit/ 镜像
├── .agents/ssot/types/     ← 从 xhyper.rs/.agent/SSOT/types/ 镜像
├── .agents/ssot/infra/     ← 从 xhyper.rs/.agent/SSOT/infra/ 镜像
│   ├── bootstrap/
│   ├── configx/
│   ├── gate/
│   ├── observex/
│   ├── resiliencx/
│   ├── schedulex/
│   ├── testkitx/
│   └── transport/
└── .agents/ssot/adapters/  ← 从 xhyper.rs/.agent/SSOT/adapters/ 镜像
    ├── exchange/           # binance, okx
    └── storage/            # clickhouse, kafka, nats, oss, postgres, redis, taos
```

> **路径约定**：保留上游 `infra/`、`adapters/` 层级，使镜像内相对链接（如 `../../kernel/`、`../../../../AGENTS.md`）与源树一致。

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
- `.agents/ssot/kernel/`、`.agents/ssot/testkit/`、`.agents/ssot/types/`、
  `.agents/ssot/infra/`、`.agents/ssot/adapters/`
  是 `xhyper.rs/.agent/SSOT/` 的只读镜像
- **禁止**在上述镜像目录内直接编辑
- 镜像更新必须使用**删除感知**同步（避免上游删改后残留陈旧文件）：
  ```bash
  # kernel / testkit / types：整目录覆盖（先删目标再拷，或 rsync --delete）
  rsync -a --delete /home/workspace/xhyper.rs/.agent/SSOT/kernel/  .agents/ssot/kernel/
  rsync -a --delete /home/workspace/xhyper.rs/.agent/SSOT/testkit/ .agents/ssot/testkit/
  rsync -a --delete /home/workspace/xhyper.rs/.agent/SSOT/types/   .agents/ssot/types/

  # infra / adapters：保留层级（勿把 * 展平到 .agents/ssot/）
  rsync -a --delete /home/workspace/xhyper.rs/.agent/SSOT/infra/    .agents/ssot/infra/
  rsync -a --delete /home/workspace/xhyper.rs/.agent/SSOT/adapters/ .agents/ssot/adapters/
  ```
- 同步后校验：`diff -rq <src> <dst>` 应无输出
- 上游变更后需手动执行镜像同步

---


### R7: 镜像文档 ≠ 本仓实现

- 镜像内 `review COMPLETE` / `Stable CLAIMED` 描述的是**上游 xhyper.rs 战役状态**
- 本仓是否落地以 `Cargo.toml` workspace members + `crates/` 路径为准
- **testkit**：本仓已落地 `crates/testkit`（`xhyper-testkit`）；`contract-testkit` 未落地
- **infra 八域**：当前仅镜像文档，本仓对应 `crates/*` **未**宣称落地
- **adapters 九域**：镜像已注册；本仓 9 个 crate 为 **scaffold**（#42），**未**宣称业务实现 / package stable
- 审计基线见 `docs/*-ssot-alignment.md` 与 `docs/workspace-ssot-alignment.md`

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
| xhyper Kernel | `xhyper.rs/.agent/SSOT/kernel/` | `.agents/ssot/kernel/` | `rsync --delete` |
| xhyper Testkit | `xhyper.rs/.agent/SSOT/testkit/` | `.agents/ssot/testkit/` | `rsync --delete` |
| xhyper Types | `xhyper.rs/.agent/SSOT/types/` | `.agents/ssot/types/` | `rsync --delete` |
| xhyper Infra | `xhyper.rs/.agent/SSOT/infra/` | `.agents/ssot/infra/` | `rsync --delete` |
| xhyper Adapters | `xhyper.rs/.agent/SSOT/adapters/` | `.agents/ssot/adapters/` | `rsync --delete` |
| SSOT 规则 | `.agents/ssot/SSOT.md` | — | 自引 |

---

## 版本

| 版本 | 日期 | 修订 |
|------|------|------|
| v1.4.0 | 2026-07-21 | 注册 adapters 镜像；R6/R7/清单/层级补 adapters；对齐文入口 |
| v1.3.0 | 2026-07-21 | infra 保留 `infra/` 层级；R6 改用 rsync --delete；清单补 infra |
| v1.2.0 | 2026-07-21 | R7：镜像≠实现；记录 testkit 本仓落地 |
| v1.1.0 | 2026-07-21 | 添加 xhyper.rs kernel/testkit/types 上游 SSOT 镜像条目 |
| v1.0.0 | 2026-07-21 | 初始 SSOT 规则定义 |
