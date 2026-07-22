# crates/ — Agent 行为规则

> 本文件定义 AI 代理在本仓库 Rust workspace crates 中的行为规范。
> SSOT 源：`.agents/ssot/SSOT.md`、`docs/constitution/`（正文；根 `CONSTITUTION.md` 为兼容索引）。

---

## 适用范围

本文件覆盖 `crates/` 目录下所有 Rust crate 的 AI 代理操作规则。

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
| `Cargo.toml` | 必选 | 包元数据、依赖、feature；**版本必须独立**（禁止 `version.workspace`，见 [VERSIONING.md](../docs/governance/VERSIONING.md) R-C1） |
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
| 仓库根 `docs/`（`governance/` · `ssot/` · `status/` · `decisions/`） | 跨 crate 治理、SSOT 对齐、状态记录、DDR | 单个 crate 的 API 契约 |
| 仓库根 `examples/` / `tests/` | 跨 crate 端到端示例与集成 | 单 crate 单元/契约测试 |
| `crates/<name>/docs/` | 该 crate 设计、边界、迁移 | 全仓治理规则 |
| `crates/<name>/tests/` | 本 crate 公开面契约 | 跨多个 crate 的 E2E |
| `crates/<name>/benches/` | 本 crate 性能基准 | 跨多个 crate 的端到端压测 |
| `crates/<name>/review/` | 本 crate 审查记录、审查指南 | 全仓审查策略 |
| `crates/<name>/releases/` | 本 crate 发布记录、签名 | workspace 级发布编排 |

### 新增 crate 检查清单

1. 在 `crates/<name>/` 按上表建齐骨架（含空目录的 `.gitkeep`）
2. `Cargo.toml`：`edition` / `license` / `repository` / `rust-version` 可用 `*.workspace = true`；**`version` 必须显式独立**（默认 `0.1.0`）
3. 在根 `Cargo.toml` 的 `workspace.members` 注册
4. 编写 `README.md`（职责一句话 + 最小用法）
5. 建立 `review/` 与 `releases/` 目录（暂无内容时 `.gitkeep`）
6. 更新本文件「Crate 概览」表
7. 更新 `ARCHITECTURE.md` 层次模型（如职责变更）
8. 确认 `node scripts/quality-gates/check-crate-versions.mjs` 通过

### 版本规则（强制，SSOT: [VERSIONING.md](../docs/governance/VERSIONING.md)）

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

| Crate | 路径 | 标准七项 |
|-------|------|----------|
| `kernel`（lib `kernel`） | `crates/kernel/` | 见 STATUS.md |
| `testkit`（lib `testkit`） | `crates/testkit/` | 见 STATUS.md |
| `xhyper-configx` / `schedulex` / `bootstrap` / `evidence` / `observex` / `resiliencx` / `transportx` | `crates/<name>/` | 见 STATUS.md |
| `decimalx` / `canonical` | `crates/types/*` | 见 STATUS.md |
| `xhyper-contracts` | `crates/contracts/` | 见 STATUS.md |
| adapters 九 package | `crates/adapters/**` | 见 STATUS.md |

> `docs/` / 暂无集成测试时的 `tests/` / 暂无基准时的 `benches/` 以 `.gitkeep` 占位。单元测试仍在 `src/` 内 `#[cfg(test)]`。  
> adapters 默认路径为真实客户端，旧内存实现仅在 `scaffold` feature；标准七项齐全与真实客户端存在仍 **≠** Production Ready。见 [docs/ssot/adapters-ssot-alignment.md](../docs/ssot/adapters-ssot-alignment.md)。

---

## 规则

### C1: 遵循宪章

- 所有代码变更必须符合 `docs/constitution/` 工程宪章规范
- 提交前运行 `make ci` 验证强制门禁

### C2: 模块边界

- 新增 crate 前先评估：是否可以用现有 crate 的模块替代
- crate 间依赖方向单向，禁止循环引用
- L0 信任根为 `kernel`；`testkit` 仅允许 dev-dependency 消费
- 依赖方向：`canonical` → `decimalx` → `kernel`；`testkit` → `kernel`；`configx` → `kernel`（L1，禁止其他 L1）；`resiliencx` → `kernel`；`bootstrap` → `kernel`；`transportx` → `kernel`（L1，R3 禁止其他 L1）
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

- 每个 `pub` 项有 `///` 注释
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

- 在 main 分支直接修改文件（须走 PR 流程）
- 提交含有 `todo!()` 的代码（须关联 issue）
- 使用 `as` 进行类型转换（用 From/TryFrom）
- 修改 `.cargo/config.toml` 未经明确授权
- 新增 crate 时跳过标准布局骨架

---

## Crate 概览

| Crate | 路径 | 职责 |
|-------|------|------|
| `kernel`（lib `kernel`） | `crates/kernel/` | L0 语义信任根（clock / lifecycle） |
| `testkit`（lib `testkit`） | `crates/testkit/` | ManualClock 等测试支持（仅 dev-dep） |
| `contract-testkit`（lib `contract_testkit`） | `crates/test-support/contracts/` | Fake + per-trait suite（仅 dev-dep） |
| `xhyper-configx`（lib `configx`） | `crates/configx/` | L1 配置存储（MemoryConfigStore） |
| `xhyper-schedulex`（lib `schedulex`） | `crates/schedulex/` | L1 任务 ID 登记表（active SSOT：无真实定时器） |
| `decimalx`（lib `decimalx`） | `crates/types/decimal/` | 十进制数值 / Money（ADR-006/007） |
| `canonical`（lib `canonical`） | `crates/types/canonical/` | 跨层共享纯 DTO（ADR-001；Money 复用 decimalx） |
| `resiliencx`（lib `resiliencx`） | `crates/resiliencx/` | L1 重试 + 熔断 + 限流 + 舱壁 + async retry（infra-s9t） |
| `bootstrap` | `crates/bootstrap/` | L1 唯一组合根；正式 KV/EventBus typed composition |
| `xhyper-contracts` | `crates/contracts/` | adapter trait 出口（Exchange/Storage） |
| `binancex` | `crates/adapters/exchange/binance/` | Binance exchange adapter（生产默认 REST+WS，#210+#214） |
| `okxx` | `crates/adapters/exchange/okx/` | OKX exchange adapter（生产默认 REST+WS，#210+#214） |
| `clickhousex` | `crates/adapters/storage/clickhouse/` | 默认 HTTP(S) 客户端；scaffold 可选 |
| `kafkax` | `crates/adapters/storage/kafka/` | 默认 rskafka 客户端；scaffold 可选 |
| `natsx` | `crates/adapters/storage/nats/` | 默认 async-nats Core/JetStream；自动恢复 NO-GO |
| `ossx` | `crates/adapters/storage/oss/` | OSS storage adapter（scaffold） |
| `postgresx` | `crates/adapters/storage/postgres/` | 默认 deadpool/tokio-postgres 客户端；scaffold 可选 |
| `redisx` | `crates/adapters/storage/redis/` | Redis storage adapter（scaffold） |
| `taosx` | `crates/adapters/storage/taos/` | TDengine storage adapter（scaffold） |

> 领域分组路径（如 `crates/types/<name>/`、`crates/adapters/{exchange,storage}/<name>/`）合法；标准布局作用于每个 workspace 成员 crate 根目录。  
> adapters SSOT 镜像：`.agents/ssot/adapters/**`；本仓状态见 [docs/ssot/adapters-ssot-alignment.md](../docs/ssot/adapters-ssot-alignment.md)。

---

## 版本

| 版本 | 日期 | 修订 |
|------|------|------|
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
