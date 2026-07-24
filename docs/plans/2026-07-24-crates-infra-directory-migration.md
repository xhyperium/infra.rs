# PLAN-INFRA-DIR-001 — `crates/infra/` 目录迁移实施计划（方案 A）

| 字段 | 值 |
|------|-----|
| 计划 ID | `PLAN-INFRA-DIR-001` |
| 日期 | 2026-07-24 |
| 状态 | **IMPLEMENTED · 待 PR 合并**（分支 `chore/infra-dir-layout`） |
| 方案 | 方案 A（窄 infra）— 见 [归组分析](../report/2026-07-24/crates-infra-grouping-analysis.md) |
| 性质 | **纯路径搬迁**；禁止夹带 API / 行为 / 依赖升级 |
| 分支建议 | `chore/infra-dir-layout`（worktree：`.worktrees/chore/infra-dir-layout`） |
| package 名 | **全部保持不变**（`cargo test -p configx` 等选择器不变） |

---

## 0. 目标与非目标

### 0.1 目标

将 L1 平台能力 crate **物理归组**到 `crates/infra/`，使源码树与已有 `types/`、`adapters/` 对称：

```text
crates/
├── kernel/                 # L0（不动）
├── types/                  # 不动
├── contracts/              # ports（不动）
├── infra/                  # ★ 新建
│   ├── configx/
│   ├── schedulex/
│   ├── resiliencx/
│   ├── observex/
│   ├── transport/          # package: transportx
│   ├── evidence/
│   └── bootstrap/          # 组合根（同树不同角色）
├── adapters/               # 不动
├── testkit/                # 不动
└── test-support/           # 不动
```

### 0.2 非目标

| 禁止 | 说明 |
|------|------|
| 改 package / lib 名 | 不引入 `infra-configx`、不恢复 `xhyper-*` |
| 合并为 `infra-core` | 历史已废弃；保持一域一 package |
| 搬 `kernel` / `types` / `contracts` / `adapters` / `test*` / `tools` | 方案 A 明确排除 |
| 改业务逻辑、feature 矩阵、依赖版本 | 另开任务 |
| 重写历史 `docs/report/**` 路径 | 历史快照可保留旧路径；活跃文档必更新 |

### 0.3 成功标准（验收）

全部满足才可标 DONE：

1. `cargo metadata --no-deps` 中 7 包 `manifest_path` 均在 `crates/infra/**`
2. `cargo test -p configx -p schedulex -p resiliencx -p observex -p transportx -p evidence -p bootstrap --all-targets` 全绿
3. `cargo test --workspace --all-features --all-targets` 全绿（或与主干同等已知 ignore）
4. `cargo fmt --all -- --check` + `cargo clippy --workspace --all-targets --all-features -- -D warnings`
5. `node scripts/quality-gates/check.mjs` 全绿
6. `node scripts/quality-gates/check-ssot-current-state.mjs` 全绿
7. `node scripts/quality-gates/check-workspace-deps.mjs` + `check-crate-versions.mjs` 全绿
8. `node scripts/docs/gen-crate-status.mjs --tracked` 后 `STATUS.md` 路径列为 `crates/infra/...`
9. 活跃文档（`AGENTS.md` / `ARCHITECTURE.md` / `crates/AGENTS.md` / `docs/ssot/*-ssot-alignment.md` / 各 crate README）无过期顶层路径
10. **零** functional diff：`git diff` 中 `*.rs` 仅允许因路径字符串出现在注释/include 中的必要改动；优先 `git mv` 保持 blob 不变

---

## 1. 迁移映射表（权威）

| # | Package（`cargo -p`） | 现路径 | 目标路径 | 目录名备注 |
|---|----------------------|--------|----------|------------|
| 1 | `configx` | `crates/configx` | `crates/infra/configx` | — |
| 2 | `schedulex` | `crates/schedulex` | `crates/infra/schedulex` | — |
| 3 | `resiliencx` | `crates/resiliencx` | `crates/infra/resiliencx` | — |
| 4 | `observex` | `crates/observex` | `crates/infra/observex` | — |
| 5 | `transportx` | `crates/transport` | `crates/infra/transport` | **目录保持 `transport`，package 名 `transportx`** |
| 6 | `evidence` | `crates/evidence` | `crates/infra/evidence` | — |
| 7 | `bootstrap` | `crates/bootstrap` | `crates/infra/bootstrap` | 组合根 |

**版本策略（冻结）**：纯路径迁移 **不** 强制 PATCH +1（无 API/行为变更）。path 依赖里的 `version = "…"` 与目标 package 现版本保持一致即可。若后续 review 要求「凡触 crate 目录必 bump」，再统一 +1 并同步全部 path version——作为 **可选 W1b**，默认跳过。

---

## 2. path 依赖改写矩阵

### 2.1 被搬迁 crate 内部（self → 外层）

搬迁后相对深度多一层：`crates/infra/<name>/` → 仓库内其它顶层 crate 用 `../../…`。

| Crate | 依赖 | 现 path | 新 path |
|-------|------|---------|---------|
| `configx` | `kernel` | `../kernel` | `../../kernel` |
| `schedulex` | （无 workspace path） | — | — |
| `resiliencx` | `kernel` | `../kernel` | `../../kernel` |
| `resiliencx` | `contracts` | `../contracts` | `../../contracts` |
| `observex` | `kernel` | `../kernel` | `../../kernel` |
| `observex` | `contracts` | `../contracts` | `../../contracts` |
| `transportx` | `kernel`（含 dev） | `../kernel` | `../../kernel` |
| `evidence` | （无 workspace path） | — | — |
| `bootstrap` | `kernel` | `../kernel` | `../../kernel` |
| `bootstrap` | `contracts` | `../contracts` | `../../contracts` |
| `bootstrap` | `observex` | `../observex` | `../observex`（**同 infra 层，不变**） |
| `bootstrap` | `evidence` | `../evidence` | `../evidence`（**同 infra 层，不变**） |
| `bootstrap` | `natsx`（dev） | `../adapters/storage/nats` | `../../adapters/storage/nats` |
| `bootstrap` | `redisx`（dev） | `../adapters/storage/redis` | `../../adapters/storage/redis` |

### 2.2 外部消费者 → 被搬迁 crate

| 消费者 | 依赖 | 现 path | 新 path |
|--------|------|---------|---------|
| `crates/adapters/storage/redis` | `resiliencx` | `../../../resiliencx` | `../../../infra/resiliencx` |
| `crates/adapters/storage/postgres` | `resiliencx` | `../../../resiliencx` | `../../../infra/resiliencx` |
| `crates/adapters/storage/oss` | `resiliencx` | `../../../resiliencx` | `../../../infra/resiliencx` |
| `crates/adapters/exchange/binance` | `transportx` | `../../../transport` | `../../../infra/transport` |
| `crates/adapters/exchange/okx` | `transportx` | `../../../transport` | `../../../infra/transport` |
| `tools/verifyctl` | `evidence` | `../../crates/evidence` | `../../crates/infra/evidence` |

### 2.3 根 `Cargo.toml` members

```toml
# 删除
"crates/configx",
"crates/schedulex",
"crates/resiliencx",
"crates/evidence",
"crates/bootstrap",
"crates/observex",
"crates/transport",

# 新增（建议按层排序，与现 members 风格一致）
"crates/infra/configx",
"crates/infra/schedulex",
"crates/infra/resiliencx",
"crates/infra/evidence",
"crates/infra/bootstrap",
"crates/infra/observex",
"crates/infra/transport",
```

### 2.4 迁移后快速自检命令

```bash
# 不应再命中旧路径（除 docs/report 历史与本计划/分析文）
rg -n 'path = "\.\./(configx|schedulex|resiliencx|observex|transport|evidence|bootstrap)"' \
  crates tools --glob '**/Cargo.toml' || true

rg -n 'path = "\.\./\.\./\.\./(resiliencx|transport)"' \
  crates/adapters --glob '**/Cargo.toml' || true

# members 与磁盘一致
cargo metadata --no-deps --format-version 1 \
  | python3 -c 'import json,sys; d=json.load(sys.stdin); print(len(d["packages"]))'
```

---

## 3. 脚本 / CI / 门禁触达清单

> 下列路径为 2026-07-24 仓库扫描结果；开工前在实现分支上再 `rg 'crates/(configx|schedulex|resiliencx|observex|transport|evidence|bootstrap)'` 扫一次。

### 3.1 必须改（阻塞验收）

| 路径 | 改什么 |
|------|--------|
| `scripts/docs/gen-crate-status.mjs` | `SSOT_DOC_BY_PREFIX` 前缀：`crates/configx` → `crates/infra/configx` 等 7 条 |
| `scripts/quality-gates/check-ssot-current-state.mjs` | package→Cargo.toml 映射；`required` 字符串；源文件存在性检查路径（configx src、evidence path 文案） |
| `scripts/quality-gates/check-ssot-current-state.test.mjs` | 同上 fixture 路径 |
| `scripts/quality-gates/verify-seven-dualbar.mjs` | `crates/${p}` / `crates/transport` → `crates/infra/...`（七包双栏脚本） |
| `scripts/quality-gates/check-decimal-no-panicking-ops.mjs` | `crates/bootstrap/src` 等扫描根 → `crates/infra/.../src` |
| `.github/workflows/configx-coverage.yml` | `paths:` 与 `cov-gate-100 --filter` |
| `.github/workflows/schedulex-coverage.yml` | 同上 |
| `.github/workflows/resiliencx-coverage.yml` | 同上 |
| `.github/workflows/observex-coverage.yml` | 同上 |
| `.github/workflows/evidence-coverage.yml` | 同上 |
| `.github/workflows/public-api.yml` | 若含 `crates/schedulex/**` 等 path filter |

`layerOf()` 对未知路径默认 `"L1"`，`crates/infra/*` **无需**改 layer 逻辑即可继续标 L1；可选增强：`if (cratePath.startsWith("crates/infra/")) return "L1"` 以自文档化。

### 3.2 建议改（活跃文档 / 代理入口）

| 路径 | 说明 |
|------|------|
| 根 `AGENTS.md` / `CLAUDE.md` | members 路径列表 |
| `ARCHITECTURE.md` | 树与层次图 |
| `crates/AGENTS.md` | 概览表路径列 |
| `README.md` | crate 表路径 |
| `docs/ssot/workspace-ssot-alignment.md` | members 表「路径」列 |
| `docs/ssot/{configx,schedulex,bootstrap,evidence,observex,resiliencx,transport}-ssot-alignment.md` | crate path 字段 |
| 各被搬 crate 内 `README.md` / `docs/README.md` / `docs/标准.md` | 自述 path、cov-gate filter |
| `STATUS.md` | **生成器刷新**，勿手改正文 |

### 3.3 可延后 / 不改

| 类别 | 策略 |
|------|------|
| `docs/report/2026-07-2{1,2,3}/**` 历史审查 | **不改**（快照）；如需可注脚「路径已迁移」 |
| `CHANGELOG.md` 历史条目 | **不改**旧条；可选在 Unreleased 记一条 chore |
| `evidence/**` 战役目录（非 crate） | 勿与 `crates/infra/evidence` 混淆；路径字符串按需检查 |
| `.agents/ssot/**` 域名 | **不搬 SSOT 树**（v2.3.0 已恢复 infra 层级）；仅更新「本仓 path」表述 |

---

## 4. 波次执行（推荐单 PR 原子迁移）

> path 依赖网是连通的：分 7 个 PR 搬会中间态红。**默认一个 PR 完成 W1–W4**。  
> 若必须拆 PR：仅允许「文档先改分析/计划」与「代码搬迁」二分；**不可**只搬子集 package。

### W0 — 开工冻结（Lead，≤30min）

- [ ] 确认方案 A 仍有效（本计划 + 归组分析无异议）
- [ ] `node scripts/worktree/worktree.mjs create chore/infra-dir-layout`
- [ ] `cd .worktrees/chore/infra-dir-layout`
- [ ] 记录基线：`git rev-parse HEAD`、`cargo test --workspace --all-features --all-targets` 出口码
- [ ] 再扫一遍 path 引用，把 §3 清单补全到 PR 描述

### W1 — `git mv` + Cargo 图修复（Executor）

顺序建议（可脚本化，但须可 review）：

```bash
mkdir -p crates/infra
git mv crates/configx    crates/infra/configx
git mv crates/schedulex  crates/infra/schedulex
git mv crates/resiliencx crates/infra/resiliencx
git mv crates/observex   crates/infra/observex
git mv crates/transport  crates/infra/transport
git mv crates/evidence   crates/infra/evidence
git mv crates/bootstrap  crates/infra/bootstrap
```

- [ ] 按 §2.1 / §2.2 改写全部相关 `Cargo.toml` path
- [ ] 更新根 `workspace.members`
- [ ] `cargo metadata --no-deps` 成功且 24 members 仍齐全
- [ ] `cargo check --workspace` 通过

**禁止**：`cp -r` 后删旧目录（破坏 git 历史追踪）；禁止同时改业务代码。

### W2 — 脚本与 CI（Executor）

- [ ] §3.1 全部路径更新
- [ ] 跑：

```bash
node scripts/quality-gates/check-ssot-current-state.mjs
node --test scripts/quality-gates/check-ssot-current-state.test.mjs
node scripts/quality-gates/check-decimal-no-panicking-ops.mjs
node scripts/docs/gen-crate-status.mjs --tracked
```

### W3 — 活跃文档（Executor / Writer）

- [ ] §3.2 列表
- [ ] `crates/infra/` 可选增加极简 `crates/infra/README.md`（说明平面职责 + 链到归组分析与本计划；**非**标准七项强制项，但推荐）
- [ ] 更新 `docs/plans/README.md` 本计划状态（合并后 → DONE）

### W4 — 全量门禁与 STATUS（Verifier）

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features --all-targets
node scripts/quality-gates/check.mjs
node scripts/quality-gates/check-crate-versions.mjs
node scripts/quality-gates/check-workspace-deps.mjs
node scripts/docs/gen-crate-status.mjs --check
```

- [ ] PR 描述贴关键命令出口摘要
- [ ] 确认 `*.rs` 无无关 diff（`git diff --stat` 以 `Cargo.toml` / 脚本 / 文档 / rename 为主）

### W5 — 合并后（可选，不阻塞）

- [ ] 在 `CHANGELOG.md` Unreleased 记：`chore: L1 平台 crate 迁入 crates/infra/`
- [ ] 关闭/关联 beads（若有）
- [ ] 通知依赖本仓 path 的外部消费者（若有 git 依赖且写死路径——通常只写 package 名则无感）

---

## 5. 风险与缓解

| 风险 | 等级 | 缓解 |
|------|------|------|
| 漏改 CI path filter → coverage job 不触发或挂 | P1 | W2 清单勾选 + PR 后看 Actions |
| 漏改 adapter path → workspace 解析失败 | P0 | W1 后立即 `cargo metadata` / `cargo check -p redisx -p binancex` |
| `transport` 目录 vs `transportx` 包名混淆 | P1 | 映射表固定；文档写清「路径 transport / 选择器 transportx」 |
| SSOT check 硬编码 `crates/evidence` 字符串 | P1 | 同步改 `check-ssot-current-state.mjs` 与测试 |
| 历史报告链接失效 | P2 | 不改历史；新分析文链新路径 |
| 大 rename 难 review | P2 | 坚持 `git mv`；PR 描述附映射表；避免混杂格式化 |
| 与并行 PR 改同一 crate 冲突 | P1 | 合并窗口与 storage/exchange 热 PR 错开；优先 rebase main |

---

## 6. 回滚方案

1. **合并前**：`git revert` 整提交或丢弃 worktree 分支  
2. **合并后**：`git revert` 迁移 PR（rename 可逆）；避免手工再 mv 一半  
3. **禁止**在回滚时夹带功能修复  

---

## 7. 预估工作量（Agent 会话）

| 波次 | 预估 |
|------|------|
| W0 | 0.5 人时 |
| W1 | 1–2 人时（含 path 矩阵核对） |
| W2 | 1 人时 |
| W3 | 1–1.5 人时 |
| W4 | 0.5–1 人时（测试墙钟） |
| **合计** | **约 4–6 人时 / 单 PR** |

---

## 8. 决策记录（开工前勾选）

| # | 决策 | 默认 | 确认 |
|---|------|------|------|
| D1 | 采用方案 A（7 包 + 不含 contracts） | ✅ | [ ] |
| D2 | 单 PR 原子迁移（不拆 package） | ✅ | [ ] |
| D3 | package 名不变；仅路径变 | ✅ | [ ] |
| D4 | 默认不 PATCH bump | ✅ | [ ] |
| D5 | 历史 `docs/report` 不改写 | ✅ | [ ] |
| D6 | 新增 `crates/infra/README.md` | ✅ 推荐 | [ ] |
| D7 | `transport` 目录名保持（非改名为 transportx/） | ✅ | [ ] |

---

## 9. 实现检查清单（PR 模板可复制）

```markdown
## 迁移 PR 检查

- [ ] git mv 七目录至 crates/infra/
- [ ] 根 members 已更新
- [ ] §2.1 内部 path 已更新
- [ ] §2.2 外部消费者 path 已更新
- [ ] gen-crate-status SSOT 前缀已更新
- [ ] check-ssot-current-state(+test) 已更新
- [ ] coverage workflows path filter 已更新
- [ ] AGENTS / ARCHITECTURE / crates/AGENTS / workspace-ssot-alignment 已更新
- [ ] STATUS.md 已生成刷新
- [ ] cargo fmt / clippy / test workspace 绿
- [ ] quality-gates check 绿
- [ ] 无业务逻辑 diff
```

---

## 10. 相关文档

| 文档 | 关系 |
|------|------|
| [crates-infra-grouping-analysis.md](../report/2026-07-24/crates-infra-grouping-analysis.md) | 方案 A 决策依据 |
| [workspace-ssot-alignment.md](../ssot/workspace-ssot-alignment.md) | members 与依赖图 SSOT |
| [crates/AGENTS.md](../../crates/AGENTS.md) | 标准布局与概览 |
| [VERSIONING.md](../../.agents/rules/VERSIONING.md) | 版本策略（本计划 D4） |
| [worktree-policy.md](../../.agents/rules/worktree-policy.md) | 必须在 worktree 实施 |

---

## 变更日志

| 日期 | 说明 |
|------|------|
| 2026-07-24 | 初版 DRAFT：方案 A 原子迁移计划 + path 矩阵 + 脚本/CI 清单 |
| 2026-07-24 | 实施落地：`git mv` 七包 + path/CI/SSOT/文档；门禁全绿 |
