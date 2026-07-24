# `.agents/ssot/` — Agent 域规格工作指引

本目录是仓库各领域的 **SSOT（Single Source of Truth）**，包含目标契约、设计、规格、证据矩阵和复审记录。实现代码在 `crates/`；对齐矩阵在 `docs/ssot/`。

## 1. 何时读这里

| 场景                                         | 读什么                                                                     |
| -------------------------------------------- | -------------------------------------------------------------------------- |
| 新增或修改 crate 的类型、trait、公开 API     | 对应域 `spec/spec.md` + `design/design.md`                                 |
| 实现跨域交互逻辑（adapter 映射、订单簿物化） | `CONTRACT.md` §1 主题映射 + §2 冲突裁决                                    |
| 关闭质量门禁                                 | 对照 `spec/spec.md` 逐项验证 + `matrix/` 门禁表                            |
| 判断是否可宣称 ship                          | 参考下方「落地状态速查」门禁统计 + `docs/ssot/workspace-ssot-alignment.md` |
| 新增域                                       | 创建 goal → design → spec → matrix 四层，更新本文件索引                    |

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

| 路径               | 层        | 角色                                                     |
| ------------------ | --------- | -------------------------------------------------------- |
| `domainx/`         | L1 域模型 | 域共享值对象：Order、Position、Trade、Portfolio          |
| `domain_market/`   | L1 域模型 | 市场数据域模型：Tick、Quote、Bar、OrderBook              |
| `domain_exchange/` | L1 域模型 | 交易所抽象：VenueAdapter trait、StreamType、AdapterError |
| `binance/`         | L2 适配器 | Binance 交易所适配器                                     |
| `okx/`             | L2 适配器 | OKX 交易所适配器                                         |
| `coinbase/`        | L2 适配器 | Coinbase 交易所适配器                                    |
| `hyperliquid/`     | L2 适配器 | Hyperliquid 交易所适配器                                 |
| `coinglass/`       | L2 适配器 | Coinglass 数据源适配器                                   |
| `orderbook/`       | L2 引擎   | 通用订单簿内核与物化引擎（当前无 runtime crate）         |

`market_data`（L0 内核）的兼容 API 在 `crates/market_data/docs/`，不重复上述类型的 SSOT。

## 4. 落地状态速查（门禁统计）

| 域                | 总门禁 | ✅ verified | 🔶 pending | 📋 specified | 🚫 blocked | ⏸️ deferred |
| ----------------- | ------ | ----------- | ---------- | ------------ | ---------- | ----------- |
| `domainx`         | 5      | 4           | 0          | 0            | 1          | 0           |
| `domain_market`   | 6      | 5           | 0          | 0            | 1          | 0           |
| `domain_exchange` | 6      | 6           | 0          | 0            | 0          | 0           |
| `binance`         | 7      | 1           | 5          | 1            | 0          | 0           |
| `okx`             | 7      | 0           | 6          | 1            | 0          | 0           |
| `coinbase`        | 7      | 1           | 5          | 1            | 0          | 0           |
| `hyperliquid`     | 8      | 0           | 5          | 2            | 0          | 1           |
| `coinglass`       | 8      | 1           | 6          | 1            | 0          | 0           |
| `orderbook`       | 13     | 0           | 8          | 2            | 0          | 3           |
| **合计**          | **67** | **18**      | **35**     | **8**        | **2**      | **4**       |

门禁语义：

- `specified`：目标契约已写定，实现尚未开始
- `skeleton`：类型/trait 骨架存在，运行时行为待实现
- `pending`：对应实现未完成或没有可重复的验证证据
- `verified`：有固定 fixture/mock + 可重复命令通过
- `blocked`：被外部依赖（如 `xhyper-canonical`）阻塞
- `deferred`：当前不纳入门禁范围的需求

### 各域关键差距

| 域                | 阻塞项 / 关键差距                                                                                                                                                                      |
| ----------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `domainx`         | **DX-CAN-001** (blocked)：等待 `xhyper-canonical` 依赖就绪后迁移 Instrument 所有权                                                                                                     |
| `domain_market`   | **DM-CAN-001** (blocked)：与 domainx 共享 canonical 阻塞；DM-BOOK-001 仅通过纯检查，provider 恢复状态机在 adapter 域                                                                   |
| `domain_exchange` | 全部 verified 但含作用域限定：DE-LIFE-001 mock 级、DE-PAGE-001 默认单页、DE-ERR-001 结构化类型已有但 adapter HTTP 映射待实现                                                           |
| `binance`         | 5/7 pending：WS 映射、Book 快照→差分恢复、ping/pong/重连、REST mock、限频逻辑；当前所有 `VenueAdapter` 方法返回骨架 `Internal`                                                         |
| `okx`             | 6/7 pending 无 verified；所有 trait 方法返回骨架；**spec 重要修正**：OKX checksum 已官方废弃（固定为 0），禁止实现旧 CRC32 算法                                                        |
| `coinbase`        | 5/7 pending：WS 订阅/映射、Book 级别恢复、REST 分页、认证边界；`CoinbaseChannel` 缺失 `MarketTrades`；`sequence_num` 不能假定 per-book 严格递增                                        |
| `hyperliquid`     | 5/8 pending + 2 specified：`allMids` 禁止映射为 `Quote`（需未来 `MidPrice` 类型）；`webbook2` 因公开文档不足 deferred                                                                  |
| `coinglass`       | 6/8 pending：URL/认证、schema 映射、instrument 映射、分页/时间窗口、限频/Retry-After；响应包装器 `data: Vec<T>` 硬编码需同时支持 object 和 array                                       |
| `orderbook`       | 0/13 verified，无 runtime crate；三种同步模型（A: Binance 外部快照+差分，B: OKX/Coinbase 流引导+差分，C: Hyperliquid 全量刷新）已指定，8 个运行时门禁 pending，3 个服务层门禁 deferred |

### 跨域阻塞项

| 阻塞项                          | 影响域                              | 状态                                                           |
| ------------------------------- | ----------------------------------- | -------------------------------------------------------------- |
| `xhyper-canonical` 未引入       | domainx、domain_market              | 等待 infra.rs 提供，当前所有 Instrument 字段使用 `String` 占位 |
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
