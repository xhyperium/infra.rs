# kernel — Agent 行为规则

> 适用 crate：`crates/kernel/`（包名 `xhyper-kernel`，lib 名 `kernel`）  
> 父级规则：[`crates/AGENTS.md`](../AGENTS.md)  
> 实现契约 SSOT：`.agents/ssot/kernel/spec/spec.md`（**SPEC-KERNEL-002**）

---

## 职责

`kernel` 是 xhyper L0 语义信任根，仅提供：

- `error` — `ErrorKind` / `XError` / `XResult` / `BoxError`
- `clock` — `Timestamp` / `MonotonicInstant` / `Clock` / `SystemClock`
- `lifecycle` — `ComponentState` / `ShutdownSignal` / `ShutdownGuard`

---

## 规则

### K1: 零运行时依赖

- 不得依赖 tokio、async-std、log/tracing 框架、配置库
- 仅允许标准库 + workspace 批准的轻量依赖（当前：`thiserror`）
- 生产依赖白名单到 `thiserror` 为止；`[features] default = []`
- `proptest` / `static_assertions` 仅为 dev-deps；`loom` 仅为 `cfg(loom)` target 依赖
- 新增依赖必须走 RFC，并更新本文件与 `README.md`

### K2: 语义稳定优先

- 公开类型变更默认视为破坏性变更
- `ErrorKind`、`ComponentState` 等分类枚举保持「按反应分类」原则
- 禁止通过字符串匹配驱动控制流的公开 API 设计
- **禁止**：`not_found` / `other`、`From<&str>` / `From<String>` for `XError`、默认 `monotonic`、公开 `Component` trait、serde/async、workspace 内部依赖
- 公开 API 须对齐 SPEC §8 冻结面

### K3: 时间语义

- 墙钟与单调钟不得混用
- 时间源必须可注入（`Clock` trait），禁止在库内隐式读系统时间作为唯一路径（`SystemClock` 是显式实现）
- 获取失败返回错误，禁止零值哨兵
- `SystemClock::monotonic` 必须经 **`origin.elapsed()` → `MonotonicInstant::from_clock_elapsed`**
- `Timestamp::checked_*` 须覆盖完整 `i64` 纳秒域（宽中间值，不得 panic/饱和）

### K4: 关停语义

- 关停一次触发、多方观察、不可逆
- 不提供启动编排、健康检查、自动重启
- `ShutdownSignal` 使用 `Mutex<bool>+Condvar` 协议；锁中毒经 `into_inner` 恢复
- 并发正确性以 loom 模型测试为准（`tests/lifecycle_concurrency_loom.rs`）

### K5: 公共 API 稳定性

- 破坏性变更须在 PR 声明，并写入 `CHANGELOG.md`
- 新增 `pub` 项须有 `///` 文档；契约测试放在 `tests/`

---

## 目录结构

> 完整标准见父级 [`crates/AGENTS.md`](../AGENTS.md)「Crate 子模块标准布局」。

```text
crates/kernel/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── clock.rs
│   ├── error.rs
│   └── lifecycle.rs
├── examples/           # 暂无示例时保留 .gitkeep
├── docs/               # 设计/迁移文档；暂无时保留 .gitkeep
├── tests/
│   ├── api_compile.rs
│   ├── clock_contract.rs
│   ├── lifecycle_concurrency.rs
│   ├── lifecycle_concurrency_loom.rs
│   └── public_api.rs
├── CHANGELOG.md
├── AGENTS.md
└── README.md
```

---

## 验证

```bash
cargo test -p xhyper-kernel --all-targets
cargo clippy -p xhyper-kernel --all-targets -- -D warnings
cargo fmt --all -- --check
RUSTFLAGS='--cfg loom' cargo test -p xhyper-kernel --test lifecycle_concurrency_loom --release
```

## 变更流程

1. 读 SPEC 对应章节与本 AGENTS
2. 仅改 `crates/kernel/**`（SSOT 镜像只读）
3. 补齐 §11 测试合同（单元 / proptest / static_assertions / loom）
4. `cargo test` + `clippy -D warnings` 全绿后提交

---

## 版本

| 版本 | 日期 | 修订 |
|------|------|------|
| v1.1.0 | 2026-07-21 | 对齐 SPEC-KERNEL-002 可移植合同（loom / proptest / static_assertions） |
| v1.0.0 | 2026-07-21 | 初始规则；对齐 crates 子模块标准布局 |
