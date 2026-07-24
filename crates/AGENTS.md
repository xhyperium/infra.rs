# crates/ — Agent 行为规则

> 本文件定义 AI 代理在本仓库 Rust workspace crates 中的行为规范。  
> SSOT 源：`.agents/ssot/SSOT.md`、`docs/constitution/`（正文；根 `CONSTITUTION.md` 为兼容索引）。  
> 成员地图与落地边界：[`docs/ssot/workspace-ssot-alignment.md`](../docs/ssot/workspace-ssot-alignment.md)。  
> **权威 package 名**以 `cargo metadata --no-deps` / 各 crate `Cargo.toml` 为准（历史 `xhyper-*` 文档别名已废弃）。

---

## 适用范围

本文件覆盖 `crates/` 目录下所有 Rust crate 的 AI 代理操作规则。  
workspace 另有 `tools/goalctl`、`tools/verifyctl`（标准布局与版本规则同样适用；细则见 [tools-ssot-alignment.md](../docs/ssot/tools-ssot-alignment.md)），**不在**本目录树内。

---

## Crate 子模块标准布局（强制）

每个 workspace 成员 crate **必须**具备下列标准骨架。新增 crate 时先建齐，再写业务代码。

> **规范七项（顺序固定）**  
> `src/` · `tests/` · `docs/` · `benches/` · `README.md` · `review/` · `releases/`  
> 另加包清单 `Cargo.toml`（Cargo 硬性要求）。

```text
crates/<crate-name>/
├── Cargo.toml          # 包清单（必选）
├── src/                # 源码（必选）
│   └── lib.rs          # 库入口（lib crate 必选）
├── tests/              # 集成/契约/公开 API 测试（必选目录；暂无内容时 .gitkeep）
├── docs/               # crate 级设计/契约/迁移（必选目录；暂无内容时 .gitkeep）
├── benches/            # 基准测试（必选目录；暂无内容时 .gitkeep）
├── README.md           # 职责、用法、feature（必选）
├── review/             # 审查记录与指南（必选目录；暂无内容时 .gitkeep）
└── releases/           # 发布记录与签名（必选目录；暂无内容时 .gitkeep）
```

### 路径职责

| 路径 | 级别 | 职责 |
|------|------|------|
| `Cargo.toml` | 必选 | 包元数据、依赖、feature；**版本必须独立**（禁止 `version.workspace`，见 [VERSIONING.md](../.agents/rules/VERSIONING.md) R-C1） |
| `src/` | 必选 | 实现与单元测试（`#[cfg(test)] mod tests`） |
| `tests/` | 必选目录 | 集成测试、跨模块契约、公开 API 稳定性 |
| `docs/` | 必选目录 | 至少 `README.md`（入口索引 + 对齐链接）；设计笔记/API 契约/迁移；不替代 rustdoc |
| `benches/` | 必选目录 | `cargo bench` 基准（criterion 等）；无内容时保留 `.gitkeep` |
| `README.md` | 必选 | 给人类与外部消费者的入口文档 |
| `review/` | 必选目录 | 审查记录、审查指南、合规签名；无内容时保留 `.gitkeep` |
| `releases/` | 必选目录 | 版本发布记录、签名、校验和；无内容时保留 `.gitkeep` |

### 分层边界（避免重复）

| 层级 | 放什么 | 不放什么 |
|------|--------|----------|
| 仓库根 `.agents/rules/` + `docs/`（`ssot/` · `status/` · `decisions/`） | 跨 crate 治理、SSOT 对齐、状态记录、DDR | 单个 crate 的 API 契约 |
| 仓库根 `examples/` / `tests/` | 跨 crate 端到端示例与集成 | 单 crate 单元/契约测试 |
| `crates/<name>/docs/` | 该 crate 设计、边界、迁移 | 全仓治理规则 |
| `crates/<name>/tests/` | 本 crate 公开面契约 | 跨多个 crate 的 E2E |
| `crates/<name>/benches/` | 本 crate 性能基准 | 跨多个 crate 的端到端压测 |
| `crates/<name>/review/` | 本 crate 审查记录、审查指南 | 全仓审查策略 |
| `crates/<name>/releases/` | 本 crate 发布记录、签名 | workspace 级发布编排 |

### 新增 crate 检查清单

1. 在 `crates/<name>/` 按上表建齐骨架（含空目录的 `.gitkeep`）
2. `Cargo.toml`：`edition` / `license` / `repository` / `rust-version` 可用 `*.workspace = true`；**`version` 必须显式独立**（默认 `0.1.0`）
3. 第三方依赖先写入根 `[workspace.dependencies]`，成员用 `{ workspace = true }`（见根 `AGENTS.md`）
4. 在根 `Cargo.toml` 的 `workspace.members` 注册
5. 编写 `README.md`（职责一句话 + 最小用法）
6. 建立 `review/` 与 `releases/` 目录（暂无内容时 `.gitkeep`）
7. 更新本文件「Crate 概览」表
8. 更新 `ARCHITECTURE.md` 层次模型（如职责变更）
9. 确认 `node scripts/quality-gates/check-crate-versions.mjs` 与 `check-workspace-deps.mjs` 通过

### 版本规则（强制，SSOT: [VERSIONING.md](../.agents/rules/VERSIONING.md)）

| 规则 | 要求 |
|------|------|
| **独立版本** | 每个 `crates/**` package 在自身 `Cargo.toml` 写 `version = "x.y.z"`；**禁止** `version.workspace = true` |
| **统一更新** | 该 crate 每次交付性更新 → **PATCH +1**（`x.y.z` → `x.y.(z+1)`） |
| **只 bump 变更 crate** | 禁止无关 crate 齐涨 |
| **path 依赖** | `dep = { path = "...", version = "…" }` 的 version 必须与目标 package 一致 |
| **工具** | `node scripts/version/crate-bump.mjs <name>`；门禁 `node scripts/quality-gates/check-crate-versions.mjs` |

### 合规现状（自动同步）

**入库看板（勿手改）**：[根目录 STATUS.md](../STATUS.md)  
**本地实时副本（gitignore）**：`docs/status/CRATES_STATUS.local.md`（主仓可刷，不脏 git）

```bash
make status                    # 本地副本必写；非 main 顺带写 STATUS.md
make status-watch              # 定时监控
node scripts/docs/gen-crate-status.mjs --local-only  # 主仓只看本地
node scripts/docs/gen-crate-status.mjs --check       # CI 新鲜度
```

下表为规范说明快照；**权威入库完成度以 `STATUS.md` 为准**；日常查看用本地副本即可（改布局时在 feature PR 里顺带刷新入库文件）。

| 层 | Package（`cargo -p`） | 路径 | 标准七项 |
|----|----------------------|------|----------|
| L0 | `kernel` | `crates/kernel/` | 见 STATUS.md |
| T0 | `testkit` · `contract-testkit` | `crates/testkit/` · `crates/test-support/contracts/` | 见 STATUS.md |
| types | `decimalx` · `canonical` | `crates/types/{decimal,canonical}/` | 见 STATUS.md |
| L1 | `bootstrap` · `configx` · `evidence` · `observex` · `resiliencx` · `schedulex` · `transportx` | `crates/<name>/`（transport 目录名 `transport`，package `transportx`） | 见 STATUS.md |
| contracts | `contracts` | `crates/contracts/` | 见 STATUS.md |
| adapter | exchange×2 + storage×7 | `crates/adapters/**` | 见 STATUS.md |
| tools* | `goalctl` · `verifyctl` | `tools/*` | 见 STATUS.md（布局规则同七项） |

> \* tools 不在 `crates/` 树内，但纳入 workspace 与 STATUS 看板。  
> `docs/` / 暂无集成测试时的 `tests/` / 暂无基准时的 `benches/` 以 `.gitkeep` 占位。单元测试仍在 `src/` 内 `#[cfg(test)]`。  
> adapters 默认路径为真实客户端，旧内存实现仅在 `scaffold` feature；标准七项齐全与真实客户端存在仍 **≠** Production Ready / package stable。见 [adapters-ssot-alignment.md](../docs/ssot/adapters-ssot-alignment.md)。

---

## 规则

### C1: 遵循宪章

- 所有代码变更必须符合 `docs/constitution/` 工程宪章规范
- 提交前运行 `make ci` 验证强制门禁（至少：`fmt` + `clippy -D warnings` + `test`）

### C2: 模块边界

- 新增 crate 前先评估：是否可以用现有 crate 的模块替代
- crate 间依赖方向单向，禁止循环引用
- L0 信任根为 `kernel`；`testkit` / `contract-testkit` **仅允许 dev-dependency** 消费，禁止进入 production normal graph
- 依赖方向（摘要，详见图见 workspace 对齐文）：
  - `canonical` → `decimalx` → `kernel`
  - L1 平台包（`configx` / `schedulex` / `bootstrap` / `evidence` / `observex` / `resiliencx` / `transportx`）→ `kernel`（L1 之间默认禁止互依赖；`bootstrap` 组合根例外见 SSOT / ADR）
  - `contracts` → 白名单依赖（`decimalx` 等）；adapters → `contracts` + 相关 L1
  - **禁止** `kernel` / `types` 依赖 adapters
- 每个 crate 目录必须符合上文「子模块标准布局」

### C3: 错误处理

- 库代码禁止裸 `unwrap()` / `expect()`
- 使用 `thiserror` 定义 crate 专用错误类型
- 错误链不可断裂，保留 `source()`

### C4: 测试

- 每个公开函数至少一个单元测试
- 单元测试置于 `#[cfg(test)] mod tests` 模块
- 集成/契约测试置于 crate 内 `tests/`
- 性能基准置于 crate 内 `benches/`（`cargo bench`）
- doc-test 必须可编译运行

### C5: 文档

- 每个 `pub` 项有 `///` 注释（中文）
- 每个 `mod.rs` / `lib.rs` 顶部有 `//!` 模块文档
- 文档注释中的示例代码用 `` ``` `` 标记并确保可运行
- crate 级说明写在 `README.md`；设计/迁移写在 `docs/`

### C6: unsafe

- 库 crate 默认禁止 `unsafe`
- 如未来需要，必须封装在安全抽象中并附 SAFETY 注释

### C7: 变更日志

- 影响公共 API 或行为的变更必须写入本 crate `releases/` 发布记录
- 破坏性变更须在 PR 中显式声明，并同步仓库根 `CHANGELOG.md`（发布维度）

---

## 禁止行为

- 在 main 分支 / 主仓工作区直接修改已跟踪文件（须 worktree + PR；见 [worktree-policy.md](../.agents/rules/worktree-policy.md)）
- 提交含有 `todo!()` 的代码（须关联 issue）
- 使用 `as` 进行类型转换（用 From/TryFrom）
- 修改 `.cargo/config.toml` 未经明确授权
- 新增 crate 时跳过标准布局骨架
- 成员 crate 内联钉第三方 `version`（须 `workspace = true`）
- 把 SSOT 镜像 COMPLETE / 布局 100% 当成 Production Ready 或 package stable

---

## Crate 概览

> package 名 = `cargo test -p <name>` 选择器。版本与完成度以各 `Cargo.toml` + `STATUS.md` 为准（下表不钉死 PATCH）。

### L0 / T0 / types

| Package | 路径 | lib | 职责 |
|---------|------|-----|------|
| `kernel` | `crates/kernel/` | `kernel` | L0 语义信任根（Clock / Timestamp / Shutdown / XError） |
| `testkit` | `crates/testkit/` | `testkit` | T0 ManualClock 等确定性测试支持（**仅 dev-dep**） |
| `contract-testkit` | `crates/test-support/contracts/` | `contract_testkit` | T0 Fake/Recording + per-trait suite（**仅 dev-dep**） |
| `decimalx` | `crates/types/decimal/` | `decimalx` | 十进制数值 / Money（ADR-006/007） |
| `canonical` | `crates/types/canonical/` | `canonical` | 跨层共享纯 DTO（ADR-001；Money 复用 decimalx） |

### L1 平台

| Package | 路径 | lib | 职责 |
|---------|------|-----|------|
| `configx` | `crates/configx/` | `configx` | 本地多源配置（Memory/Env/File）+ 分层 + 宿主 reload/通知 + secret 脱敏；非远端配置中心 |
| `schedulex` | `crates/schedulex/` | `schedulex` | 任务 ID 登记 + 宿主驱动确定性 `JobRunner::tick`；非 runtime/分布式 scheduler |
| `bootstrap` | `crates/bootstrap/` | `bootstrap` | 唯一组合根；正式 KV/EventBus 固定槽位 + 显式 shutdown/drain |
| `evidence` | `crates/evidence/` | `evidence` | 审计证据追加 / 查询 / 签名面 |
| `observex` | `crates/observex/` | `observex` | instrumentation + 有界进程内遥测 sink；非 OTLP 实现 |
| `resiliencx` | `crates/resiliencx/` | `resiliencx` | 重试（含 async）+ 熔断 + 限流 + 舱壁 |
| `transportx` | `crates/transport/` | `transportx` | HTTP/WS 传输（TLS 模式、池、代理） |

### contracts / adapters

| Package | 路径 | 职责 / 边界 |
|---------|------|-------------|
| `contracts` | `crates/contracts/` | adapter trait 出口（Exchange/Storage）；Additive Only |
| `binancex` | `crates/adapters/exchange/binance/` | 签名 REST + 公共 WS 解析/注入；**交易 NO-GO**；非 package stable |
| `okxx` | `crates/adapters/exchange/okx/` | 四头签名 REST + 公共 WS 解析/注入；**交易 NO-GO**；非 package stable |
| `clickhousex` | `crates/adapters/storage/clickhouse/` | 默认 HTTP(S) AnalyticsSink 客户端；`scaffold` 可选 |
| `kafkax` | `crates/adapters/storage/kafka/` | 默认 rskafka 客户端（TLS/PLAIN 等）；group/native EOS NO-GO |
| `natsx` | `crates/adapters/storage/nats/` | 默认 async-nats Core/JetStream；同客户端重启恢复可证，断线窗口无回放 / Cluster·HA NO-GO |
| `ossx` | `crates/adapters/storage/oss/` | 默认 ObjectStore 客户端（streaming/SSE/presign 等）；非 package stable |
| `postgresx` | `crates/adapters/storage/postgres/` | 默认 deadpool/tokio-postgres 客户端；`scaffold` 可选 |
| `redisx` | `crates/adapters/storage/redis/` | 默认生产 Redis 客户端（KV + 可选 PubSub）；`scaffold` 可选 |
| `taosx` | `crates/adapters/storage/taos/` | 默认 TDengine REST 客户端；幂等/HA/package stable NO-GO |

### workspace 外 crate 树（交叉引用）

| Package | 路径 | 职责 |
|---------|------|------|
| `goalctl` | `tools/goalctl/` | 最小 Goal→Contract CLI（doctor/validate/compile） |
| `verifyctl` | `tools/verifyctl/` | 最小 plan/execute/report CLI；**非**生产 verifier |

> 领域分组路径（`crates/types/<name>/`、`crates/adapters/{exchange,storage}/<name>/`）合法；标准布局作用于每个 workspace 成员 crate 根目录。  
> adapters SSOT：`.agents/ssot/adapters/**`；本仓状态见 [adapters-ssot-alignment.md](../docs/ssot/adapters-ssot-alignment.md) 与分 package `docs/ssot/*-ssot-alignment.md`。  
> 无 `infra-core`；gate / xtask 等未宣称本仓落地。

---

## 版本

| 版本 | 日期 | 修订 |
|------|------|------|
| v1.7.0 | 2026-07-24 | 对齐 cargo metadata：修正 package 名（去掉废弃 `xhyper-*`）；补齐 evidence/observex/transportx；刷新 adapters 职责与 NO-GO 边界；合规表分层；交叉引用 tools |
| v1.6.0 | 2026-07-21 | 规范八项 → 七项（去 examples/CHANGELOG/AGENTS，增 review/releases） |
| v1.5.0 | 2026-07-21 | 标准布局新增 `benches/`（规范七项 → 八项） |
| v1.4.0 | 2026-07-21 | adapters/contracts 标准布局全绿；补 bootstrap/contracts 概览 |
| v1.3.0 | 2026-07-21 | 新增 `xhyper-bootstrap`（L1 组合根） |
| v1.2.0 | 2026-07-21 | 移除 `infra-core`；L0 改为 `xhyper-kernel` |
| v1.1.3 | 2026-07-21 | 合规表全绿：补齐 `types/decimal` / `types/canonical` 标准布局缺口 |
| v1.1.2 | 2026-07-21 | 合规表与概览补齐 `testkit` / `types/decimal` / `types/canonical`；标注布局缺口 |
| v1.1.1 | 2026-07-21 | 锁定标准条目顺序：src → examples → docs → tests → CHANGELOG → AGENTS → README |
| v1.1.0 | 2026-07-21 | 增加 crate 子模块标准布局（七项 + Cargo.toml） |
| v1.0.0 | 2026-07-21 | 初始代理规则 |

## testkit（test-support）

- path：`crates/testkit` · package：`testkit` · lib：`testkit`
- 仅允许 **dev-dependency** 消费；禁止进入生产 normal graph
- 生产依赖仅 `kernel`

## contract-testkit（test-support）

- path：`crates/test-support/contracts` · package：`contract-testkit` · lib：`contract_testkit`
- 仅允许 **dev-dependency** 消费；禁止进入 production normal graph
- Fake/Recording + `assert_*` suite；权威规格 SPEC-TESTKIT-002 §3.2
