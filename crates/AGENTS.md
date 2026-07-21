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
> `src/` · `examples/` · `docs/` · `tests/` · `CHANGELOG.md` · `AGENTS.md` · `README.md`  
> 另加包清单 `Cargo.toml`（Cargo 硬性要求）。

```text
crates/<crate-name>/
├── Cargo.toml          # 包清单（必选）
├── src/                # 源码（必选）
│   └── lib.rs          # 库入口（lib crate 必选）
├── examples/           # 可运行示例（必选目录；暂无内容时 .gitkeep）
├── docs/               # crate 级设计/契约/迁移（必选目录；暂无内容时 .gitkeep）
├── tests/              # 集成/契约/公开 API 测试（必选目录；暂无内容时 .gitkeep）
├── CHANGELOG.md        # 本 crate 变更日志（Keep a Changelog + SemVer）（必选）
├── AGENTS.md           # 本 crate Agent 行为规则（必选）
└── README.md           # 职责、用法、feature（必选）
```

### 路径职责

| 路径 | 级别 | 职责 |
|------|------|------|
| `Cargo.toml` | 必选 | 包元数据、依赖、feature；版本跟 `workspace.package` |
| `src/` | 必选 | 实现与单元测试（`#[cfg(test)] mod tests`） |
| `examples/` | 必选目录 | 可 `cargo run --example` 的示例；无内容时保留 `.gitkeep` |
| `docs/` | 必选目录 | 至少 `README.md`（入口索引 + 对齐链接）；设计笔记/API 契约/迁移；不替代 rustdoc |
| `tests/` | 必选目录 | 集成测试、跨模块契约、公开 API 稳定性 |
| `CHANGELOG.md` | 必选 | 本 crate 版本变更；仓库根 `CHANGELOG.md` 记整体发布 |
| `AGENTS.md` | 必选 | 本 crate 专属 Agent 规则；父级为 `crates/AGENTS.md` |
| `README.md` | 必选 | 给人类与外部消费者的入口文档 |

### 分层边界（避免重复）

| 层级 | 放什么 | 不放什么 |
|------|--------|----------|
| 仓库根 `docs/`（`governance/` · `ssot/` · `status/` · `decisions/`） | 跨 crate 治理、SSOT 对齐、状态记录、DDR | 单个 crate 的 API 契约 |
| 仓库根 `examples/` / `tests/` | 跨 crate 端到端示例与集成 | 单 crate 单元/契约测试 |
| `crates/<name>/docs/` | 该 crate 设计、边界、迁移 | 全仓治理规则 |
| `crates/<name>/examples/` | 只依赖本 crate（及声明的依赖）的示例 | workspace 级演示 |
| `crates/<name>/tests/` | 本 crate 公开面契约 | 跨多个 crate 的 E2E |

### 新增 crate 检查清单

1. 在 `crates/<name>/` 按上表建齐骨架（含空目录的 `.gitkeep`）
2. `Cargo.toml` 使用 `*.workspace = true` 对齐 workspace 元数据
3. 在根 `Cargo.toml` 的 `workspace.members` 注册
4. 编写 `README.md`（职责一句话 + 最小用法）
5. 编写 `AGENTS.md`（职责、本 crate 专属规则、目录树）
6. 初始化 `CHANGELOG.md`（`## [Unreleased]`）
7. 更新本文件「Crate 概览」表
8. 更新 `ARCHITECTURE.md` 层次模型（如职责变更）

### 合规现状（2026-07-21）

| Crate | 路径 | src | examples | docs | tests | CHANGELOG | AGENTS | README |
|-------|------|:---:|:--------:|:----:|:-----:|:---------:|:------:|:------:|
| `xhyper-kernel`（lib `kernel`） | `crates/kernel/` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `xhyper-testkit`（lib `testkit`） | `crates/testkit/` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `xhyper-configx`（lib `configx`） | `crates/configx/` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `xhyper-schedulex`（lib `schedulex`） | `crates/schedulex/` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `xhyper-decimalx`（lib `decimalx`） | `crates/types/decimal/` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `xhyper-canonical`（lib `canonical`） | `crates/types/canonical/` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `xhyper-resiliencx`（lib `resiliencx`） | `crates/resiliencx/` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `xhyper-bootstrap`（lib `bootstrap`） | `crates/bootstrap/` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `xhyper-contracts` | `crates/contracts/` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| adapters 九 package（见概览） | `crates/adapters/**` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `xhyper-transportx`（lib `transportx`） | `crates/transport/` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

> `examples/` / `docs/` / 暂无集成测试时的 `tests/` 以 `.gitkeep` 占位。单元测试仍在 `src/` 内 `#[cfg(test)]`。  
> adapters / contracts 为 scaffold；标准七项已补齐，**≠** 业务实现完成。见 [docs/ssot/adapters-ssot-alignment.md](../docs/ssot/adapters-ssot-alignment.md)。

---

## 规则

### C1: 遵循宪章

- 所有代码变更必须符合 `docs/constitution/` 工程宪章规范
- 提交前运行 `make ci` 验证强制门禁

### C2: 模块边界

- 新增 crate 前先评估：是否可以用现有 crate 的模块替代
- crate 间依赖方向单向，禁止循环引用
- L0 信任根为 `xhyper-kernel`；`testkit` 仅允许 dev-dependency 消费
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

- 影响公共 API 或行为的变更必须写入本 crate `CHANGELOG.md`
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
| `xhyper-kernel`（lib `kernel`） | `crates/kernel/` | xhyper L0 语义信任根（clock / lifecycle） |
| `xhyper-testkit`（lib `testkit`） | `crates/testkit/` | ManualClock 等测试支持（仅 dev-dep） |
| `xhyper-configx`（lib `configx`） | `crates/configx/` | L1 配置存储（MemoryConfigStore） |
| `xhyper-schedulex`（lib `schedulex`） | `crates/schedulex/` | L1 任务 ID 登记表（active SSOT：无真实定时器） |
| `xhyper-decimalx`（lib `decimalx`） | `crates/types/decimal/` | 十进制数值 / Money（ADR-006/007） |
| `xhyper-canonical`（lib `canonical`） | `crates/types/canonical/` | 跨层共享纯 DTO（ADR-001；Money 复用 decimalx） |
| `xhyper-resiliencx`（lib `resiliencx`） | `crates/resiliencx/` | L1 重试（active SSOT §2；熔断/限流未实现） |
| `xhyper-bootstrap`（lib `bootstrap`） | `crates/bootstrap/` | L1 唯一组合根（ADR-016；typed composition） |
| `xhyper-contracts` | `crates/contracts/` | adapter trait 出口（Exchange/Storage） |
| `binancex` | `crates/adapters/exchange/binance/` | Binance exchange adapter（scaffold） |
| `okxx` | `crates/adapters/exchange/okx/` | OKX exchange adapter（scaffold） |
| `clickhousex` | `crates/adapters/storage/clickhouse/` | ClickHouse storage adapter（scaffold） |
| `kafkax` | `crates/adapters/storage/kafka/` | Kafka storage adapter（scaffold） |
| `natsx` | `crates/adapters/storage/nats/` | NATS storage adapter（scaffold） |
| `ossx` | `crates/adapters/storage/oss/` | OSS storage adapter（scaffold） |
| `postgresx` | `crates/adapters/storage/postgres/` | Postgres storage adapter（scaffold） |
| `redisx` | `crates/adapters/storage/redis/` | Redis storage adapter（scaffold） |
| `taosx` | `crates/adapters/storage/taos/` | TDengine storage adapter（scaffold） |

> 领域分组路径（如 `crates/types/<name>/`、`crates/adapters/{exchange,storage}/<name>/`）合法；标准布局作用于每个 workspace 成员 crate 根目录。  
> adapters SSOT 镜像：`.agents/ssot/adapters/**`；本仓状态见 [docs/ssot/adapters-ssot-alignment.md](../docs/ssot/adapters-ssot-alignment.md)。

---

## 版本

| 版本 | 日期 | 修订 |
|------|------|------|
| v1.4.0 | 2026-07-21 | adapters/contracts 标准布局全绿；补 bootstrap/contracts 概览 |
| v1.3.0 | 2026-07-21 | 新增 `xhyper-bootstrap`（L1 组合根） |
| v1.2.0 | 2026-07-21 | 移除 `infra-core`；L0 改为 `xhyper-kernel` |
| v1.1.3 | 2026-07-21 | 合规表全绿：补齐 `types/decimal` / `types/canonical` 标准布局缺口 |
| v1.1.2 | 2026-07-21 | 合规表与概览补齐 `testkit` / `types/decimal` / `types/canonical`；标注布局缺口 |
| v1.1.1 | 2026-07-21 | 锁定标准条目顺序：src → examples → docs → tests → CHANGELOG → AGENTS → README |
| v1.1.0 | 2026-07-21 | 增加 crate 子模块标准布局（七项 + Cargo.toml） |
| v1.0.0 | 2026-07-21 | 初始代理规则 |

## testkit（test-support）

- path：`crates/testkit` · package：`xhyper-testkit` · lib：`testkit`
- 仅允许 **dev-dependency** 消费；禁止进入生产 normal graph
- 生产依赖仅 `xhyper-kernel`

