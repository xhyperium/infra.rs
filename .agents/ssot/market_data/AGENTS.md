# `.agents/ssot/` — Agent 域规格工作指引

本目录是仓库各领域的 **SSOT（Single Source of Truth）**，包含目标契约、设计、规格、证据矩阵和复审记录。实现代码在 `crates/`；对齐矩阵在 `docs/ssot/`。

## 1. 何时读这里

> 领域规格 (`domainx` / `domain_market` / `domain_exchange`) 已移至 `core/`。适配器/引擎开发时需同时阅读 `core/` 下的领域规格和本目录的适配器规格。

| 场景                                         | 读什么                                                                     |
| -------------------------------------------- | -------------------------------------------------------------------------- |
| 新增或修改领域类型/trait 规格                 | `core/<domain>/spec/spec.md` + `design/design.md`                         |
| 实现 adapter 映射与跨域交互                   | `CONTRACT.md` §1 主题映射 + §2 冲突裁决                                    |
| 关闭适配器/引擎质量门禁                      | 对照本目录 `spec/spec.md` 逐项验证 + `matrix/` 门禁表                      |
| 判断是否可宣称 ship                          | 参考下方「落地状态速查」+ `docs/ssot/workspace-ssot-alignment.md`          |
| 新增数据源适配器                             | 创建 goal → design → spec → matrix 四层，更新本文件索引                    |

## 2. 标准文档结构（域叶节点）

```text
goal/goal.md       # 目标：为什么需要、解决什么问题
design/design.md   # 设计：架构决策（ADR）、权衡
spec/spec.md       # 规格：API 契约、类型约束（主题 SSOT）
review/            # 复审记录：逐轮评审结论
evidence/          # 外部协议证据（供应商文档原文摘录）
matrix/            # 追溯矩阵（门禁 → 实现 → 测试）
```

**Code 不在本树**：实现路径在 `crates/<name>/`；禁止在 SSOT 目录写 `src/`、`Cargo.toml`、`*.rs` 副本。

## 3. 本仓域树

> 领域规格 (`domainx` / `domain_market` / `domain_exchange`) 已移至 **`.agents/ssot/core/`**，详见 `core/AGENTS.md`。
> 本目录仅保留适配器和引擎规格。

| 路径               | 层        | 角色                                                     |
| ------------------ | --------- | -------------------------------------------------------- |
| `binance/`         | L2 适配器 | Binance 交易所适配器                                     |
| `okx/`             | L2 适配器 | OKX 交易所适配器                                         |
| `coinbase/`        | L2 适配器 | Coinbase 交易所适配器                                    |
| `hyperliquid/`     | L2 适配器 | Hyperliquid 交易所适配器                                 |
| `coinglass/`       | L2 适配器 | Coinglass 数据源适配器                                   |
| `orderbook/`       | L2 引擎   | 通用订单簿内核与物化引擎（当前无 runtime crate）         |

`market_data`（L0 内核）的兼容 API 在 `crates/market_data/docs/`，不重复上述类型的 SSOT。

### 基础设施复用约定（强制）

本域所有 sink / 存储 / 消息基础设施**统一优先复用 infra.rs 本仓现有 7 个适配器**，禁止引入外部存储组件或重新实现连接逻辑：

| 用途 | 复用模块（intra-workspace path 依赖） |
|------|--------------------------------------|
| 行情流 / 消息总线 | `kafkax`、`natsx` |
| 热缓存 / KV | `redisx` |
| 时序 / 关系 / 对象 / 列式 | `taosx`、`postgresx`、`ossx`、`clickhousex` |

- market_data 仅构建**薄领域包装层**，将 `MarketEvent` 编码为各适配器期望格式（见 `binance/design/design.md` ADR-001）。
- 依赖以 **intra-workspace path** 引入（package 名无 `xhyper-` 前缀；intra-workspace 允许内联 version），**禁止**以 git/tag 外部依赖形式引入 infra.rs 自身模块。详见 `binance/infra-deps.md`。
- 各适配器落地边界（package stable / Cluster·EOS / 部分 live 证据 OPEN）以 [adapters-ssot-alignment](../../../docs/ssot/adapters-ssot-alignment.md) 为准。

## 4. 落地状态速查（门禁统计）

> 领域规格门禁见 `core/AGENTS.md`。

| 域                | 总门禁 | ✅ verified | 🔶 pending | 📋 specified | 🚫 blocked | ⏸️ deferred |
| ----------------- | ------ | ----------- | ---------- | ------------ | ---------- | ----------- |
| `binance`         | 7      | 1           | 5          | 1            | 0          | 0           |
| `okx`             | 7      | 0           | 6          | 1            | 0          | 0           |
| `coinbase`        | 7      | 1           | 5          | 1            | 0          | 0           |
| `hyperliquid`     | 8      | 0           | 5          | 2            | 0          | 1           |
| `coinglass`       | 8      | 1           | 6          | 1            | 0          | 0           |
| `orderbook`       | 13     | 0           | 8          | 2            | 0          | 3           |
| **合计**          | **50** | **3**       | **35**     | **8**        | **0**      | **4**       |

门禁语义：

- `specified`：目标契约已写定，实现尚未开始
- `skeleton`：类型/trait 骨架存在，运行时行为待实现
- `pending`：对应实现未完成或没有可重复的验证证据
- `verified`：有固定 fixture/mock + 可重复命令通过
- `blocked`：被外部依赖（如 `xhyper-canonical`）阻塞
- `deferred`：当前不纳入门禁范围的需求

### 各域关键差距

> 领域规格差距见 `core/AGENTS.md`。

| 域                | 阻塞项 / 关键差距

### 跨域阻塞项

> 领域规格阻塞项见 `core/AGENTS.md`。

| 阻塞项                          | 影响域                              | 状态                                                           |
| ------------------------------- | ----------------------------------- | -------------------------------------------------------------- |
| exchange adapter runtime 空实现 | binance、okx、coinbase、hyperliquid | 所有 `VenueAdapter` trait 方法返回骨架，需逐 adapter 实现      |
| orderbook 无 runtime crate      | orderbook（13 门禁）                | 领域规格完整，但无对应的 Rust crate                            |

## 5. 变更规则

1. **worktree + PR** 修改本树（禁止 main 直接改）
2. 改规格后同步 `docs/ssot/workspace-ssot-alignment.md` 若影响落地判定
3. 跨主题变更必须同时更新：受影响 `spec.md`、对应 `design/goal`、追溯矩阵、证据来源和 review 决议
4. 提交前运行 `git diff --check`、UTF-8/LF 检查、仓库规定的 Cargo gates

## 6. 验证命令

```bash
# build / test / lint
cargo build --workspace
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-features --all-targets -- -D warnings
cargo deny check
```
