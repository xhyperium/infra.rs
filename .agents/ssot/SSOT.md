# SSOT 规则 — 单一事实源

> `.agents/ssot/` — 本仓库代理层与域规格的**本仓单一事实源**。
> 本文件是 SSOT 本身的 SSOT：域目录结构、落地判定与变更规则以此为准。

---

## 定义

**SSOT (Single Source of Truth)**：整个项目中，任何一个事实（数据、配置、规则、技能、域规格）有且仅有一个权威来源。其他位置必须是派生投影，不得分叉维护。

---

## SSOT 层级

```text
源层（可编辑）
├── .claude/skills/         ← 技能定义
├── docs/constitution/      ← 工程宪章正文 SSOT
├── CONSTITUTION.md         ← 宪章兼容索引（指向 docs/constitution/）
├── AGENTS.md               ← 多 Agent 协作规则
├── .agents/rules/          ← 项目规则 SSOT
├── .github/                ← CI/CD
├── docs/                   ← 项目文档（含 docs/ssot/ 对齐矩阵）
└── .agents/ssot/           ← 域规格 SSOT 根（本仓）
    ├── kernel/
    ├── testkit/
    ├── types/
    ├── infra/              # L1 平台平面（与 crates/infra/ 对齐）
    │   ├── bootstrap/
    │   ├── configx/
    │   ├── evidence/       # canonical evidence current-state 规格
    │   ├── gate/           # 规格镜像；本仓 OOS/未 member
    │   ├── observex/
    │   ├── resiliencx/
    │   ├── schedulex/
    │   ├── testkitx/       # 规格镜像；非 crates/testkit
    │   └── transport/
    ├── adapters/           # exchange + storage
    ├── contracts/
    └── tools/              # goalctl / xtask / verifyctl；evidence 子目录仅历史入口

投影层（只读派生）
└── .agents/skills/         ← 从 .claude/skills/ 投影
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
- 同时创建对应的投影规则或对齐文档
- 各域三轮生产加固允许在既有 `plan/` 下新增
  `round-01-findings.md`、`round-02-findings.md`、`round-03-findings.md`；
  文件只记录本轮发现、修复、验证与残余边界，不替代 active `spec/spec.md`
- 上述轮次证据必须同步到对应 `docs/ssot/*-ssot-alignment.md`，且不得把
  “结构完成”表述为 package stable、完整平台或远端生产能力

### R5: 废除与迁移

- SSOT 源位置变更时，旧位置保留重定向说明（至少一个版本周期）
- 投影层与对齐文档必须同步更新

### R6: 本仓域规格树

- `.agents/ssot/{kernel,testkit,types,infra,adapters,contracts,tools}/` 是 **infra.rs 本仓** 的域规格 SSOT
  - **infra 平面**（与 `crates/infra/` 对齐）：`.agents/ssot/infra/{bootstrap,configx,evidence,gate,observex,resiliencx,schedulex,testkitx,transport}/`
  - 旧根路径 `.agents/ssot/{bootstrap,configx,…}/` 仅保留 **R5 重定向 README**（非 active spec）
- 禁止在 SSOT 树内写入 `src/`、`Cargo.toml`、`*.rs` 实现副本
- 禁止用 SSOT 文档中的 COMPLETE / Stable 叙事冒充 crate 已 ship
- 路径一律使用 `.agents/ssot/`（禁止旧 monorepo 单数 agent 路径写法）
- 域树变更经 worktree + PR 合入；不从外仓路径覆盖本树
- 保留层级：`adapters/`、`tools/`、**`infra/`**（与 crates 平面一致；2026-07-24 起取消 v2.1 展平）

### R7: 规格文档 ≠ 本仓实现

- SSOT 内 `review COMPLETE` / `Stable CLAIMED` / Phase Approved **仅**描述规格或历史战役状态
- 本仓是否落地以 `Cargo.toml` workspace members + `crates/` 路径 + 本仓测试为准
- **testkit**：本仓已落地 `crates/testkit`；`contract-testkit` 已落地 `crates/test-support/contracts`
- **infra**：按各域对齐文；未在 members 中的域不得宣称落地
  - **configx**：内存 KV + Memory/Env/File source + 分层合并 + 宿主触发 reload/进程内通知 + `SecretString` 已实现；远端配置中心、自动文件监听和 secret manager 仍 OPEN
  - **schedulex**：任务 ID 登记表 + 宿主驱动的确定性 `JobRunner::tick` 已实现；非 runtime / 分布式调度平台
- **adapters 九域**（2026-07-22 · #188–#191）：
  - **storage×7**（redis/postgres/kafka/nats/oss/clickhouse/taos）：**生产默认客户端 P0 已落地** + live `#[ignore]` + benches；scaffold 改 `feature = "scaffold"`
  - **exchange×2**（binance/okx）：可观察实现为签名 REST + 公共 WS 解析/注入；该实现入口不等于可交易
  - 交易执行精度 filters、限流、时钟偏移、私有 WS、重连与受控 live 交易证据仍 OPEN，交易 **NO-GO**
  - **未**宣称 package stable / Cluster·JetStream·EOS 全量 / crates.io
- **tools**（2026-07-22 · #188–#191）：
  - `crates/infra/evidence` 最小面已落地；current-state spec 唯一入口为 `.agents/ssot/infra/evidence/spec/spec.md`
  - `.agents/ssot/tools/evidence/` 仅为历史重定向入口，不持有 active spec
  - `tools/goalctl` · `tools/verifyctl` **workspace members**（最小 CLI；verifyctl 非生产 verifier）
  - `tools/xtask` **未**落地
- 落地说明：`plan/infra-rs-landing.md`（各域）；draft 快照：`plan/infra-rs-draft-*.md`
- 审计基线见 `docs/ssot/*-ssot-alignment.md` 与 `docs/ssot/workspace-ssot-alignment.md`

---

## 当前 SSOT 清单

| 事实域 | SSOT 位置 | 说明 |
|--------|----------|------|
| Agent 技能 | `.claude/skills/` | 投影至 `.agents/skills/` |
| 工程宪章 | `docs/constitution/` | 正文 SSOT；根 `CONSTITUTION.md` 仅为兼容索引 |
| 项目规则 | `.agents/rules/` | 工程约定与加严落地；`docs/governance/` 仅为 stub |
| Agent 行为 | `AGENTS.md` | 直接引用 |
| CI/CD | `.github/workflows/` | 直接引用 |
| 项目文档 | `docs/` | 直接引用 |
| 变更日志 | `CHANGELOG.md` | 直接引用 |
| Cargo 配置 | `.cargo/config.toml` | 直接引用 |
| Crate 规则 | `crates/AGENTS.md` | 按 crate 细化 |
| Kernel | `.agents/ssot/kernel/` | L0 规格；实现 `crates/kernel` |
| Testkit | `.agents/ssot/testkit/` | 规格；实现 `crates/testkit` |
| Types | `.agents/ssot/types/` | decimal / canonical |
| Evidence | `.agents/ssot/infra/evidence/` | canonical current-state 规格；实现 `crates/infra/evidence` |
| Infra | `.agents/ssot/infra/{bootstrap,configx,evidence,gate,observex,resiliencx,schedulex,testkitx,transport}/` | L1 平台平面（与 `crates/infra/` 对齐） |
| Adapters | `.agents/ssot/adapters/` | exchange + storage |
| Contracts | `.agents/ssot/contracts/` | trait 出口规格 |
| Tools | `.agents/ssot/tools/` | goalctl / xtask / verifyctl；`tools/evidence` 仅历史重定向 |
| SSOT 规则 | `.agents/ssot/SSOT.md` | 自引 |

---

## 版本

| 版本 | 日期 | 修订 |
|------|------|------|
| v2.3.0 | 2026-07-24 | **恢复 infra 层级**：与 `crates/infra/` 对齐，域归组到 `.agents/ssot/infra/*`；根路径保留 R5 重定向 README |
| v2.2.1 | 2026-07-23 | 在 v2.2.0 current-state 基线上声明域级三轮 findings 证据文件约定；清理重复域目录清单 |
| v2.2.0 | 2026-07-22 | 唯一化顶层 evidence current-state 入口；冻结 24 package 与 configx/schedulex/exchange 当前边界 |
| v2.1.0 | 2026-07-21 | **展平 SSOT 结构**：移除 `infra/` 子目录；bootstrap/configx/gate/observex/resiliencx/schedulex/testkitx/transport 直接位于 `.agents/ssot/` 根 |
| v2.0.0 | 2026-07-21 | **彻底本仓化**：`.agents/ssot` 不再表述为外仓镜像；清除外仓路径字面量；R6/R7/清单重写 |
| v1.6.0 | 2026-07-21 | 本仓化 tools SSOT（evidence/goalctl/xtask/verifyctl） |
| v1.5.0 | 2026-07-21 | 注册 contracts |
| v1.4.0 | 2026-07-21 | 注册 adapters |
| v1.3.0 | 2026-07-21 | infra 保留 `infra/` 层级 |
| v1.2.0 | 2026-07-21 | R7：规格≠实现；记录 testkit 落地 |
| v1.1.0 | 2026-07-21 | 添加 kernel/testkit/types 条目 |
| v1.0.0 | 2026-07-21 | 初始 SSOT 规则定义 |
