# bootstrap SSOT 对齐矩阵（本仓）

| 字段 | 值 |
|------|-----|
| 审计日期 | 2026-07-23；三轮 production hardening + 正式 contracts 注入复核 |
| SSOT | `.agents/ssot/infra/bootstrap/spec/spec.md` ≡ `spec/xhyper-bootstrap-complete-spec.md`（active / complete 双镜像，`cmp` 同构） |
| 实现路径 | `crates/infra/bootstrap`（package `bootstrap` / lib `bootstrap`）· **v0.3.3** |
| 权威 | active spec 描述合同；本文件记录 **本仓** 落地与证据 |
| 上游参考 | `xhyper.rs/crates/infra/bootstrap`（可移植源，非本仓 member） |
| OBJECTIVE | 所列实现/证据已由本地独立 reviewer 审查，独立 verifier 已完成技术/证据初验；GitHub 固定提交 CI artifact 与交付流程 pending；上述不等于跨资源事务、交易栈全量装配或 Production Ready |

## 路径映射

| SSOT / 上游表述 | 本仓 |
|-----------------|------|
| `crates/infra/bootstrap`（infra.rs README） | `crates/infra/bootstrap` |
| `crates/infra/bootstrap`（上游 monorepo） | 映射为本仓扁平 `crates/infra/bootstrap` |
| `.agent/ssot/bootstrap` | `.agents/ssot/infra/bootstrap`（SSOT 已展平） |

## §1 定位与边界

| 要求 | 判定 | 证据 |
|------|------|------|
| ADR-016 唯一组合根；runtime gate 已退役 | **PASS** | 无 `Gate` / `register_capability` / `resolve` 动态服务定位面；drain 的 `register` 仅注册关停 hook |
| 运行时依赖经 typed `PlatformContext` / `AppContext` / bounded contexts | **PASS** | `src/lib.rs`、`src/bounded.rs` |
| 禁止字符串 / `Any` / `TypeId` Service Locator；禁止通用 register/resolve | **PASS** | 公开 API 仅 builder + 只读访问器；`tests/public_api.rs` |
| 可依赖其他 L1 完成装配，但不跨层 re-export adapter 类型 | **PASS** | 生产 dep：kernel + contracts + observex + evidence；不 re-export 交易所 adapter |
| 非目标：通用 DI、配置解析、重试/调度/传输、业务状态机、Evidence 核心实现 | **PASS** | 本 crate 不实现上述能力 |

## §2 依赖

| SSOT 依赖 | 本仓 | 判定 |
|-----------|------|------|
| `kernel`（Shutdown / ErrorKind） | path `crates/kernel` | **PASS** |
| `contracts`（Instrumentation / storage traits） | path `crates/contracts`；re-export `Instrumentation` | **PASS**（ADR-005 trait 权威） |
| 正式 storage contracts | `ContractStoreSet` 固定 `KeyValueStore` / `EventBus` trait-object 槽位 | **PASS**（additive；无动态注册） |
| 有界 venue/storage 替面 | `BoundedMarketDataSource` / `BoundedKeyValueStore` / …（**非** contracts 同名 trait） | **PASS**（命名收敛） |
| `StoreSet` 适配器接线面 | `src/store_set.rs`；`Bootstrap::with_store_set` | **PASS**（类型化注入；禁止动态 register/resolve） |
| `AsyncDrain` 关停排空 | `src/drain.rs`；`register_drain` / `graceful_shutdown` | **PASS**（同步 hook；signal 先触发；批内 LIFO） |
| `observex`（`TracingInstrumentation`） | path `crates/infra/observex`；`Bootstrap::new` 默认 | **PASS**（ADR-005 默认实现） |
| `evidence`（`EvidenceAppender`） | path `crates/infra/evidence`；re-export + `InMemoryEvidenceAppender` | **PASS**（注入/可选/require；远程见 evidence 对齐） |
| dev-dependencies | 集中声明的 async/tokio 辅助依赖 + `natsx 0.3.2` / `redisx 0.3.4` path 依赖 | **PASS**（仅固定摘要组合实验；不进入生产依赖） |
| 真实 exchange 业务装配 e2e | 非本包职责 | **OPEN**（exchange 生产默认 REST+WS（业务 live 签名下单仍 NO-GO 默认 CI）；**≠** StoreSet API 缺失） |

## §3 公开 API

| 类型 | 判定 | 证据 |
|------|------|------|
| `Bootstrap` | **PASS** | `src/lib.rs` |
| `PlatformContext` | **PASS** | instrumentation / shutdown / optional evidence |
| `AppContext` | **PASS** | platform + 窄访问器 + 唯一 shutdown owner + 可失败 graceful shutdown |
| `MarketDataContext` | **PASS** | `src/bounded.rs` + stub 构造测试 |
| `ExecutionContext` | **PASS** | 同上 |
| `BootstrappedApp` | **PASS** | `into_parts` / trigger-only / `Result` graceful shutdown |
| `ShutdownController` | **PASS** | `into_parts` 显式转移的唯一 guard；drop 不触发 |
| `StoreSet` | **PASS** | `src/store_set.rs`；KV/Tx/Bus/Repo/Venue 可选句柄 |
| `ContractStoreSet` | **PASS** | `src/contract_store_set.rs`；正式 KV/EventBus typed slots |
| `AsyncDrain` / `DrainStepResult` | **PASS** | 同步快照；批内 LIFO；错误后继续；逐步结果 |
| `BootstrapError` | **PASS** | Display / XError context 简体中文；DependencyUnavailable 顶层不内插 source，kind/source 链保持结构化 |
| 无 `Gate` / `Capability` / `register_capability` / 动态 mutation | **PASS** | 静态检查 + 公开导出列表 |

## §4 构建与错误语义

| 路径 / 规则 | 判定 | 证据 |
|-------------|------|------|
| `build` → `AppContext` | **PASS** | release/debug 强制校验；成功产物持唯一 guard |
| `try_build` → `Result<…, BootstrapError>` | **PASS** | 强制校验；成功产物持唯一 guard |
| `build_app` → `BootstrappedApp` | **PASS** | 间接走 `build`；单元 + 集成 + example |
| `try_build_app` | **PASS** | 间接走 `try_build`；fail + ok + graceful 路径 |
| 可选 evidence 未注入为 `None` | **PASS** | tests |
| `require_evidence` **release/debug 均 fail-closed**（`build` panic + `try_*` Err） | **PASS** | infra-s9t.4 / #168：禁止 release 静默成功 |
| Missing→`Missing` / Invalid→`Invalid` / Unavailable→`Unavailable` | **PASS** | `error.rs` + tests；`Into<XError>` / `kind()` / `into_xresult` |
| 用户可见错误语言 | **PASS（实现）** | 三类 Display/context 与 drain 自有文本为简体中文；精确测试覆盖 |
| 外部错误边界 | **PASS（有界）** | re-export、下层 source/hook context 由定义方负责；bootstrap 不吞 source |
| signal → drain 组合顺序 | **PASS** | `AppContext` / `BootstrappedApp::graceful_shutdown`；hook 内断言 signal 已触发 |
| ownerless graceful | **PASS** | signal 未触发 → `Missing(shutdown_guard)` 且 hook 零执行；外部预触发 → 允许 drain |
| drain 注册锁中毒 | **PASS** | 低层 `Internal`；builder 映射 `DependencyUnavailable(drain)` 并保留 source |
| `trigger_shutdown` 兼容语义 | **PASS** | 只触发 signal，不隐式执行 hook |
| `into_parts` 所有权转移 | **PASS** | guard 移入 controller；拆出的 context 须先由 controller 触发再 graceful drain |

## §5 成熟度与开放项

| 项 | 判定 | 说明 |
|----|------|------|
| workspace 非测试 consumer | **LIMITED** | `examples/minimal.rs` 为库外示例；不是生产 app 证据 |
| StoreSet 适配器接线 API | **PASS** | `with_store_set` + `StoreSet::with_*`；**诚实边界**：注入句柄 ≠ 交易所业务协议完成 |
| 正式 contracts 接线 | **PASS** | `with_contract_store_set`；真实 Redis/NATS 仅从 `AppContext` trait 访问器调用 |
| 跨资源事务 / 泛型 Repository 注册 | **NO-GO** | 明确非目标；无全局 locator |
| AsyncDrain 关停排空 | **PASS（有界）** | 同步、进程内、快照批内 LIFO；**无** timeout / 取消 / panic 隔离 |
| composition manifest（BOOT-MAN-001） | **OPEN** | 非本轮 OBJECTIVE |
| 异步组件启动/逆序补偿（全量） | **OPEN** | drain 提供 hook 面；完整 async 启动编排非目标 |
| 生产就绪 / package stable / Agent L5 | **未宣称** | 人签模板未填 |
| 交易栈端到端装配 | **NO-GO** | exchange 生产默认 REST+WS（live 签名交易非默认 CI）；StoreSet 只能接线已有 trait 实现 |

## §6 验收命令（本仓）

```bash
cargo test -p bootstrap --all-targets
cargo check -p bootstrap --all-targets
cargo clippy -p bootstrap --all-targets -- -D warnings
cargo fmt --all --check
node scripts/storage-composition-conformance.mjs
node scripts/quality-gates/cov-gate-100.mjs -p bootstrap --filter crates/infra/bootstrap/src
cmp .agents/ssot/infra/bootstrap/spec/spec.md \
    .agents/ssot/infra/bootstrap/spec/xhyper-bootstrap-complete-spec.md
# 静态：无 Service Locator / Gate
rg -n 'register_capability|fn resolve|pub struct Gate|pub enum Gate' crates/infra/bootstrap/src || true
```

| 门禁 | 期望 |
|------|------|
| 测试 | 绿 |
| clippy `-D warnings` | 绿 |
| fmt | 绿 |
| 行覆盖率 | **PASS**：最终错误文本修复后 root 串行 exit 0；`963 / 963`，zeros 0，100.0000% |
| `xtl lint-deps` / `xtl no-new-gate` | **DEFER**（本仓无 `cargo xtl` 工具链时跳过；语义由静态 rg 覆盖） |

## 验证记录指针

本地会话证据目录（非 git）：实现者 scratch 下的 `bootstrap-*.log` / `bootstrap-cov.txt`。  
可复现命令见上文 §6。

## OBJECTIVE 处置（2026-07-22 defer-close）

| 项 | 前状态 | 现状态 | 证据 |
|----|--------|--------|------|
| StoreSet / adapter 接线 | DEFER | **PASS** | `crates/infra/bootstrap/src/store_set.rs` · `Bootstrap::with_store_set` |
| 同步 drain | DEFER | **PASS（有界）** | `crates/infra/bootstrap/src/drain.rs` · signal→快照内 LIFO；无 timeout/cancel |
| 正式 storage trait 注入 | DEFER | **PASS** | `ContractStoreSet` + 固定摘要 Redis/NATS E2E |

## 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-23 | 合并 main 正式 `ContractStoreSet` KV/EventBus 注入与固定摘要 Redis/NATS 组合实验；保留三轮关停/错误 hardening；不声明跨资源事务 |
| 2026-07-23 | 当前 Cargo 版本对齐为 v0.3.3；最终测试 46 + 10 + 4 = 60 passed、1 ignored；最终错误文本修复后覆盖率 `963 / 963`；`975 / 975` 与 `961 / 961` 均为中间树历史 |
| 2026-07-23 | 第 3 轮 P0：三类 `BootstrapError` Display/XError context、drain 自有错误与示例输出统一为简体中文；thiserror 修复前 root 串行覆盖率 `975 / 975`、zeros 0、100.0000%、exit 0 |
| 2026-07-23 | 第 3 轮早期阶段：root 曾完成 PATCH bump 至 v0.3.2；错误语言补丁前历史覆盖率为 887/887；后续当前版本已为 v0.3.3 |
| 2026-07-23 | 第 2 轮：ownerless graceful fail-closed；外部预触发允许 drain；mutex poison 错误映射与 100% 行覆盖率门禁 |
| 2026-07-23 | 第 1 轮：修复 `build` / `try_build` 丢 guard；新增 AppContext / BootstrappedApp graceful shutdown；同步 active/complete spec 与 Cargo/测试事实 |
| 2026-07-22 | **defer-close**：StoreSet + AsyncDrain PASS；交易装配仍 NO-GO |
| 2026-07-22 | 对齐 Cargo 真相：package/lib `bootstrap` v0.3.1；组合根装配 kernel+contracts+observex+evidence，**非**完整应用运行时 |
| 2026-07-21 | 生产就绪：`Bounded*` 有界面命名收敛；与 contracts 权威 trait 区分；PR #98 **合入 main** |
| 2026-07-21 | infra-s9t.4：`require_evidence` 在 `build`/`build_app` 路径 release panic fail-closed；#168 |

## 追溯

- SSOT：`.agents/ssot/infra/bootstrap/spec/spec.md`
- 上游：`xhyper.rs/crates/infra/bootstrap`
- 本仓实现：`crates/infra/bootstrap/**`
- contracts 对齐：[contracts-ssot-alignment.md](./contracts-ssot-alignment.md)
