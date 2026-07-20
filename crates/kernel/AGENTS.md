# kernel — Agent 行为规则

> 适用 crate：`crates/kernel/`（包名 `xhyper-kernel`，lib 名 `kernel`）  
> 父级规则：[`crates/AGENTS.md`](../AGENTS.md)

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
- 新增依赖必须走 RFC，并更新本文件与 `README.md`

### K2: 语义稳定优先

- 公开类型变更默认视为破坏性变更
- `ErrorKind`、`ComponentState` 等分类枚举保持「按反应分类」原则
- 禁止通过字符串匹配驱动控制流的公开 API 设计

### K3: 时间语义

- 墙钟与单调钟不得混用
- 时间源必须可注入（`Clock` trait），禁止在库内隐式读系统时间作为唯一路径（`SystemClock` 是显式实现）
- 获取失败返回错误，禁止零值哨兵

### K4: 关停语义

- 关停一次触发、多方观察、不可逆
- 不提供启动编排、健康检查、自动重启

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
│   └── public_api.rs
├── CHANGELOG.md
├── AGENTS.md
└── README.md
```

---

## 版本

| 版本 | 日期 | 修订 |
|------|------|------|
| v1.0.0 | 2026-07-21 | 初始规则；对齐 crates 子模块标准布局 |
