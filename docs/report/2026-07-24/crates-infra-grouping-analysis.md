# 深度归类分析：哪些模块适合进 `crates/infra/`

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-24 |
| 性质 | **只读架构分析**；非实现、非 Maintainer 签核、非 package stable |
| 范围 | `crates/` 全部 workspace 成员 + `tools/*` 交叉引用 |
| 输入 | `Cargo.toml` members、`cargo metadata` 依赖图、`docs/ssot/workspace-ssot-alignment.md`、STATUS 分层、七包双栏报告、`.agents/ssot/SSOT.md` |
| 结论类型 | 目录归组建议（路径迁移可选）；**不改** Cargo package 名 |

---

## 0. 结论先行

**`crates/infra/` 最合理的含义是「L1 平台能力层」**——可组合的运行时横切能力，**不是**把整个仓库都塞进 “infra” 这个大筐。

仓库身份虽叫 `infra.rs`，但 `crates/` 里已有多条正交平面；SSOT 也写过 **infra 平面曾展平**（`.agents/ssot/infra/` → `bootstrap/configx/…`），目录归组应和这条语义对齐，而不是再造一个大杂烩。

| 决策 | 内容 |
|------|------|
| **应收（7）** | `configx` · `schedulex` · `resiliencx` · `observex` · `transport(x)` · `evidence` · `bootstrap` |
| **明确不收** | `kernel` · `types/*` · `contracts` · `adapters/**` · `testkit` · `contract-testkit` · `tools/*` |
| **预留** | 未来落地的 `gate` → `crates/infra/gate` |
| **命名纪律** | 只改**路径**，不改 **Cargo package 名** |
| **是否立刻搬** | 治理对齐项；有价值但高摩擦，应独立 PR，勿夹功能变更 |

---

## 1. 先定边界：什么叫 “infra 目录”

用本仓已有分层（STATUS / workspace 对齐文 / 依赖图）来切：

| 平面 | 现有路径 | 角色 |
|------|----------|------|
| **L0 信任根** | `crates/kernel/` | Clock / lifecycle / 错误语义 |
| **types** | `crates/types/*` | 数值与跨层 DTO |
| **contracts** | `crates/contracts/` | adapter trait 出口（ports） |
| **L1 平台能力** | 散落在 `crates/` 根下 | 配置 / 调度 / 弹性 / 观测 / 传输 / 证据 / 组合根 |
| **adapters** | `crates/adapters/**` | 具体后端 / 交易所实现 |
| **test-support** | `testkit` + `contract-testkit` | 仅 dev-dep |
| **tools** | `tools/*` | CLI，不在 `crates/` |

`crates/infra/` 应只收 **L1 平台能力**，目标是：

- 路径语义与 `.agents/ssot/{bootstrap,configx,observex,…}` 一致
- 与已成组的 `types/`、`adapters/`、`test-support/` 对称
- **不**吃掉 L0、types、contracts、adapters

```text
依赖方向（简图）

  adapters ──► contracts ──► types ──► kernel
      │             ▲
      │             │
      └──► resiliencx / transportx 等 L1
                  ▲
            bootstrap（组装）
```

---

## 2. 推荐纳入：`crates/infra/*`（核心 6+1）

### 2.1 强推荐（纯 L1 平台能力）

这些 package 在 STATUS 里已是 **L1**，SSOT 也把它们算在 infra 平面；依赖几乎只有 `kernel`（部分再加 `contracts`），**不**依赖具体 adapter：

| Package | 现路径 | 建议路径 | 为何算 infra |
|---------|--------|----------|--------------|
| `configx` | `crates/configx` | `crates/infra/configx` | 本地配置合并 / reload / secret 脱敏 |
| `schedulex` | `crates/schedulex` | `crates/infra/schedulex` | 宿主驱动 tick / 任务登记（非分布式调度） |
| `resiliencx` | `crates/resiliencx` | `crates/infra/resiliencx` | 重试 / 熔断 / 限流 / 舱壁 |
| `observex` | `crates/observex` | `crates/infra/observex` | instrumentation + 有界进程内 sink |
| `transportx` | `crates/transport` | `crates/infra/transport`（或 `transportx`） | HTTP/WS 传输边界 |
| `evidence` | `crates/evidence` | `crates/infra/evidence` | 审计证据追加面（L1 库，不是 tools CLI） |

依据：

- workspace 对齐文把它们标成 **L1 平台**
- 「七包」报告核心集合：`configx · evidence · observex · resiliencx · schedulex · transportx · contracts`——其中除 `contracts` 外，其余都是平台能力
- 依赖实测：`configx`/`transportx` → 仅 `kernel`；`resiliencx`/`observex` → `kernel` + `contracts`；`evidence` 甚至无 workspace normal dep

### 2.2 条件推荐：`bootstrap`

| Package | 建议 | 理由 |
|---------|------|------|
| `bootstrap` | **可进 `infra/bootstrap`，但单独注明「组合根」** | 仍是 L1；ADR-016 唯一 composition root；组装 `contracts` + `observex` + `evidence`，dev 才碰 redisx/natsx |

它不是「又一个能力库」，而是 **平台组装层**。进 `infra/` 合理，因为：

- 规格与 SSOT 一直在 infra 平面（`.agents/ssot/bootstrap/`）
- 产品语义是「启动期平台上下文」，不是 adapter

但实现上要注意：它在依赖图上 **高于** 多数 L1，不宜和 `configx` 混成同一「无层级」心智——目录上可用：

```text
crates/infra/
  configx/ schedulex/ resiliencx/ observex/ transport/ evidence/   # 能力
  bootstrap/                                                       # 组合根（同树不同角色）
```

---

## 3. 边界案例：不建议 / 仅备选

### 3.1 `contracts` — **不进 `infra/`（更稳）**

| 选项 | 评价 |
|------|------|
| 留在 `crates/contracts/` | **推荐**：ports 层，被 adapters + 部分 L1 共用 |
| 塞进 `infra/contracts` | 会把「平台实现」与「端口定义」糊在一起；adapters 目录语义变脏 |

`contracts` 是 **trait 出口（R4 Additive Only）**，不是「可运行的基础设施实现」。  
七包分析把它和 L1 能力并列表格，是 **发布成熟度并查**，不是 **目录平面同一**。  
更干净的树：

```text
crates/
  contracts/          # ports（独立）
  infra/…             # 平台能力
  adapters/…          # 实现
```

### 3.2 `kernel` — **不要进 `infra/`**

- L0 信任根；types / 几乎所有包都依赖它
- 放进 `infra/` 会暗示它是「可选平台模块」，削弱 L0 地位
- 历史已废弃 `infra-core`；再把 kernel 包进 infra 容易重复踩坑

### 3.3 `types/*` — **保持 `crates/types/`**

- 已有稳定分组；ADR-001/006/007 是 **领域类型平面**
- 与 config/retry/transport 职责正交

### 3.4 `adapters/**` — **保持 `adapters/`，绝不当 infra 子树**

- 后端与交易所实现；交易 **NO-GO**、Cluster/HA 等边界都在 adapter 叙事
- 已有 `exchange/` + `storage/` 结构；再套一层 `infra/adapters` 无收益

### 3.5 `testkit` / `contract-testkit` — **不进 infra**

- T0、**仅 dev-dep**；production graph 禁止
- 应留在 `testkit` + `test-support/`（或未来 `crates/test-support/testkit`）

### 3.6 仅镜像、未落地的 SSOT 域

| 域 | 状态 | 与 `crates/infra/` 关系 |
|----|------|------------------------|
| `gate` | 仅 `.agents/ssot/gate/` | **未来**若落地，优先 `crates/infra/gate` |
| `testkitx` | 仅镜像 | 更偏 test 平面，**不要**默认塞进 infra |

---

## 4. 三套方案对比

### 方案 A — 窄 infra（最干净，推荐）

```text
crates/infra/
  configx/
  schedulex/
  resiliencx/
  observex/
  transport/      # package: transportx
  evidence/
  bootstrap/      # 组合根
```

| 进 | 不进 |
|----|------|
| 上表 7 个 | kernel、types、contracts、adapters、test*、tools |

**优点**：与 SSOT infra 平面一致；目录对称 `types/` / `adapters/` / `infra/`  
**成本**：path 依赖与文档大量改写；`bootstrap` → `../adapters` 会变成 `../../adapters` 等

### 方案 B — 七包对齐（能力 + contracts）

方案 A **再加** `contracts` → `crates/infra/contracts` 或并列 `crates/infra/../contracts` 仍独立。

| 评价 |
|------|
| 对齐「七包」报表范围，但 **语义混 ports 与 platform**，长期不如 A |

### 方案 C — 宽 infra（不推荐）

`kernel` + types + contracts + 全部 L1 全塞 `infra/`。

| 评价 |
|------|
| 目录名等于仓库名，**零信息增益**；破坏已有 `types/`/`adapters/` 心智；迁移爆炸 |

---

## 5. 依赖与「谁该动」一览（决策表）

| Package | 进 `infra/`？ | 置信度 | 关键理由 |
|---------|---------------|--------|----------|
| `configx` | ✅ 是 | 高 | 纯 L1；→ kernel only |
| `schedulex` | ✅ 是 | 高 | 纯 L1；几乎 std-only |
| `resiliencx` | ✅ 是 | 高 | L1 弹性；被 storage 复用 |
| `observex` | ✅ 是 | 高 | L1 观测；bootstrap 正式依赖 |
| `transportx` | ✅ 是 | 高 | L1 传输；exchange 依赖 |
| `evidence` | ✅ 是 | 中高 | L1 库；SSOT 已从 tools 收到顶层 domain |
| `bootstrap` | ✅ 是（组合根位） | 中高 | 平台组装；勿与 adapter 混目录 |
| `contracts` | ❌ 否 | 高 | ports 层，应独立 |
| `kernel` | ❌ 否 | 高 | L0 |
| `decimalx`/`canonical` | ❌ 否 | 高 | types 平面 |
| storage×7 / exchange×2 | ❌ 否 | 高 | adapters 平面 |
| `testkit` / `contract-testkit` | ❌ 否 | 高 | T0 dev-only |
| `gate`（未来） | ✅ 预留 | — | SSOT 已在 infra 平面 |
| `goalctl`/`verifyctl` | ❌（保持 `tools/`） | 高 | CLI 不是库平面 |

---

## 6. 若落地，目标树长什么样

```text
crates/
├── kernel/                 # L0（顶层固定）
├── types/                  # 类型平面（已有）
│   ├── decimal/
│   └── canonical/
├── contracts/              # ports（建议仍顶层）
├── infra/                  # ★ L1 平台平面（新）
│   ├── configx/
│   ├── schedulex/
│   ├── resiliencx/
│   ├── observex/
│   ├── transport/          # package transportx
│   ├── evidence/
│   └── bootstrap/
├── adapters/               # 已有
│   ├── exchange/…
│   └── storage/…
├── testkit/
└── test-support/
    └── contracts/          # contract-testkit
```

这与 SSOT 规则「保留 `adapters/`、`tools/` 层级；infra 域展平为多个 domain 名」**不冲突**：

- SSOT 树：域名在 `.agents/ssot/{configx,bootstrap,…}`
- 源码树：用 `crates/infra/*` **物理归组**，package 名仍可 `configx` / `bootstrap` 不变

---

## 7. 迁移成本与风险（决定「现在要不要做」）

| 项 | 影响 |
|----|------|
| `Cargo.toml` members + 全部 path 依赖 | 高：adapters 三级路径、bootstrap→adapters 等全要改 |
| CI / 脚本 / STATUS 生成器路径 | 中：`gen-crate-status`、ssot check、composition 脚本 |
| 文档 SSOT 对齐路径 | 中：大量 `crates/configx` → `crates/infra/configx` |
| package **名** | 建议 **不变**（只改目录）；避免破坏 `-p configx` 与下游引用 |
| 行为 / API | 应为 **纯搬迁**（no-op 行为变更） |

**ROI 判断**：

- **现在做**：目录更清晰，新人一眼看到「平台 vs 适配器」
- **暂缓也合理**：功能已稳定；`types/`/`adapters/` 已分组，根下 7 个 L1 扁平尚可扫读；搬迁是高摩擦、低功能收益的治理项

若做，应单独 `chore(infra): move L1 platform crates under crates/infra` 一类 PR，**禁止**夹带功能变更。

---

## 8. 依赖图证据（2026-07-24 快照）

`cargo metadata --no-deps` 中与归组相关的 workspace normal 依赖（摘要）：

| Package | path | normal → |
|---------|------|----------|
| `configx` | `crates/configx` | `kernel` |
| `schedulex` | `crates/schedulex` | （无 workspace normal） |
| `resiliencx` | `crates/resiliencx` | `contracts`, `kernel` |
| `observex` | `crates/observex` | `contracts`, `kernel` |
| `transportx` | `crates/transport` | `kernel` |
| `evidence` | `crates/evidence` | （无 workspace normal） |
| `bootstrap` | `crates/bootstrap` | `contracts`, `evidence`, `kernel`, `observex`（dev: natsx, redisx） |
| `contracts` | `crates/contracts` | `canonical`, `kernel` |
| `kernel` | `crates/kernel` | — |
| adapters（例） | `crates/adapters/**` | 普遍 → `contracts` + `kernel`；部分 → `resiliencx` / `transportx` / types |

---

## 9. 最终建议（可执行立场）

1. **`crates/infra/` 应收（7）**  
   `configx` · `schedulex` · `resiliencx` · `observex` · `transport(x)` · `evidence` · `bootstrap`

2. **明确不要收**  
   `kernel` · `types/*` · `contracts` · `adapters/**` · `testkit` · `contract-testkit` · `tools/*`

3. **预留位**  
   未来落地的 `gate` → `crates/infra/gate`

4. **命名纪律**  
   只改 **路径**，不改 **Cargo package 名**；避免再引入 `xhyper-*` 或 `infra-core` 式合并包

5. **是否立刻搬**  
   属于 **治理对齐** 而非功能必需；有价值，但应在独立 worktree + 全量 path/CI/文档同步后做，而不是顺手 refactor

---

## 10. 一句话

**`crates/infra/` = L1 平台能力 + 组合根；  
`contracts` 是端口、`adapters` 是实现、`kernel/types` 是更底层信任与数据，都不应塞进这个目录。**

---

## 11. 相关链接

| 文档 | 用途 |
|------|------|
| [workspace-ssot-alignment.md](../../ssot/workspace-ssot-alignment.md) | members 地图与依赖图 |
| [seven-l1-contracts-dual-bar-readiness.md](../2026-07-21/seven-l1-contracts-dual-bar-readiness.md) | 七包双栏（能力 + contracts） |
| [crates/AGENTS.md](../../../crates/AGENTS.md) | crate 标准布局与概览 |
| [ARCHITECTURE.md](../../../ARCHITECTURE.md) | 层次模型（部分叙述可能滞后） |
| [.agents/ssot/SSOT.md](../../../.agents/ssot/SSOT.md) | 域规格 SSOT 规则（infra 展平） |
| [文档组织约定](../../../.agents/rules/文档组织约定.md) | 报告落盘路径规范 |

---

## 变更日志

| 日期 | 说明 |
|------|------|
| 2026-07-24 | 初版：会话架构分析落盘；方案 A 推荐 |
