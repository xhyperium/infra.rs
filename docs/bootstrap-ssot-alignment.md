# bootstrap SSOT 对齐矩阵（本仓）

| 字段 | 值 |
|------|-----|
| 审计日期 | 2026-07-21 |
| SSOT（只读） | `.agents/ssot/infra/bootstrap/spec/spec.md` ≡ `spec/xhyper-bootstrap-complete-spec.md`（`cmp` 同构） |
| 实现路径 | `crates/bootstrap`（package `xhyper-bootstrap` / lib `bootstrap`） |
| 权威 | 本文件描述 **本仓** 落地状态；**不**编辑 `.agents/ssot/**` 镜像 |
| 上游参考 | `xhyper.rs/crates/infra/bootstrap`（可移植源，非本仓 member） |

## 路径映射

| SSOT / 上游表述 | 本仓 |
|-----------------|------|
| `crates/bootstrap`（infra.rs README） | `crates/bootstrap` |
| `crates/infra/bootstrap`（上游 monorepo） | 映射为本仓扁平 `crates/bootstrap` |
| `.agent/ssot/bootstrap` | `.agents/ssot/infra/bootstrap`（R6 保留 `infra/`） |

## §1 定位与边界

| 要求 | 判定 | 证据 |
|------|------|------|
| ADR-016 唯一组合根；runtime gate 已退役 | **PASS** | 无 `Gate` 类型；`rg 'pub (struct\|enum\|type) Gate\|fn register\|fn resolve' crates/bootstrap` 无匹配 |
| 运行时依赖经 typed `PlatformContext` / `AppContext` / bounded contexts | **PASS** | `src/lib.rs`、`src/bounded.rs` |
| 禁止字符串 / `Any` / `TypeId` Service Locator；禁止通用 register/resolve | **PASS** | 公开 API 仅 builder + 只读访问器；`tests/public_api.rs` |
| 可依赖其他 L1 完成装配，但不跨层 re-export adapter 类型 | **PASS** | 生产 dep：kernel + contracts + observex；不 re-export 交易所 adapter |
| 非目标：通用 DI、配置解析、重试/调度/传输、业务状态机、Evidence 核心实现 | **PASS** | 本 crate 不实现上述能力 |

## §2 依赖

| SSOT 依赖 | 本仓 | 判定 |
|-----------|------|------|
| `xhyper-kernel`（Shutdown / ErrorKind） | path `crates/kernel` | **PASS** |
| `xhyper-contracts`（Instrumentation） | path `crates/contracts`；re-export `Instrumentation` | **PASS**（ADR-005 trait 权威）/ 全量 venue async 仍 **DEFER**（`traits` 最小替面） |
| `xhyper-observex`（`TracingInstrumentation`） | path `crates/observex`；`Bootstrap::new` 默认 | **PASS**（ADR-005 默认实现） |
| `xhyper-evidence`（`EvidenceAppender`） | 最小 `EvidenceAppender` + `EvidenceError` | **PASS**（注入/可选/require）/ 全量 evidence **DEFER** |
| dev：binance / redisx / canonical / tokio e2e | 无 monorepo adapters | **DEFER**（见非目标）；以 stub trait double + 单元/集成测试替代组合证明 |

## §3 公开 API

| 类型 | 判定 | 证据 |
|------|------|------|
| `Bootstrap` | **PASS** | `src/lib.rs` |
| `PlatformContext` | **PASS** | instrumentation / shutdown / optional evidence |
| `AppContext` | **PASS** | platform + 窄访问器 |
| `MarketDataContext` | **PASS** | `src/bounded.rs` + stub 构造测试 |
| `ExecutionContext` | **PASS** | 同上 |
| `BootstrappedApp` | **PASS** | `into_parts` / `trigger_shutdown` |
| `ShutdownController` | **PASS** | `trigger` / `has_guard`；drop 不触发 |
| `BootstrapError` | **PASS** | `src/error.rs` |
| 无 `Gate` / `Capability` / `register_capability` / 动态 mutation | **PASS** | 静态检查 + 公开导出列表 |

## §4 构建与错误语义

| 路径 / 规则 | 判定 | 证据 |
|-------------|------|------|
| `build` → `AppContext`（`debug_assert!(validate)`） | **PASS** | `src/lib.rs`；默认成功路径有测 |
| `try_build` → `Result<…, BootstrapError>` 强制校验 | **PASS** | `require_evidence` 无注入 → Missing |
| `build_app` → `BootstrappedApp` | **PASS** | 单元 + 集成 + example |
| `try_build_app` | **PASS** | fail + ok 双路径 |
| 可选 evidence 未注入为 `None` | **PASS** | tests |
| `require_evidence` 仅 `try_*` fail-closed（infallible 路径已知差距） | **PASS** | 与 SSOT 当前兼容事实一致；非长期安全合同 |
| Missing→`Missing` / Invalid→`Invalid` / Unavailable→`Unavailable` | **PASS** | `error.rs` + tests；`Into<XError>` / `kind()` / `into_xresult` |
| 关停单次触发、controller/guard 共享 signal | **PASS** | lifecycle 经 kernel；trigger 后 `is_triggered` |

## §5 成熟度与开放项

| 项 | 判定 | 说明 |
|----|------|------|
| workspace 非测试 consumer | **PASS（本仓）** | `examples/minimal.rs` 为库外 consumer 路径（非生产 app） |
| 真实 app 生命周期 / async drain | **DEFER** | SSOT 开放项；本目标非目标 |
| composition manifest（BOOT-MAN-001） | **DEFER** | 非目标 |
| 异步组件启动/逆序补偿 | **DEFER** | 非目标 |
| 生产就绪 / package stable | **未宣称** | SSOT Status：非生产就绪 |

## §6 验收命令（本仓）

```bash
cargo test -p xhyper-bootstrap --all-targets
cargo check -p xhyper-bootstrap --all-targets
cargo clippy -p xhyper-bootstrap --all-targets -- -D warnings
cargo fmt --all --check
cargo llvm-cov -p xhyper-bootstrap --all-targets --fail-under-lines 100 --summary-only
# 静态：无 Service Locator / Gate
rg -n 'fn register|fn resolve|pub struct Gate|pub enum Gate' crates/bootstrap/src || true
```

| 门禁 | 期望 |
|------|------|
| 测试 | 绿 |
| clippy `-D warnings` | 绿 |
| fmt | 绿 |
| 行覆盖率 | **100%**（目标门禁，高于上游 SSOT 仅测集声明） |
| `xtl lint-deps` / `xtl no-new-gate` | **DEFER**（本仓无 `cargo xtl` 工具链时跳过；语义由静态 rg 覆盖） |

## 验证记录指针

本地会话证据目录（非 git）：实现者 scratch 下的 `bootstrap-*.log` / `bootstrap-cov.txt`。  
可复现命令见上文 §6。

## 追溯

- SSOT：`.agents/ssot/infra/bootstrap/spec/spec.md`
- 上游：`xhyper.rs/crates/infra/bootstrap`
- 本仓实现：`crates/bootstrap/**`
