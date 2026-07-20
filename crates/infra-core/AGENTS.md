# infra-core — Agent 行为规则

> 适用 crate：`crates/infra-core/`
> 父级规则：`crates/AGENTS.md`

---

## 职责

`infra-core` 是 workspace 的基础层 crate，提供：

- 核心错误类型 `Error` 与 `Result<T>` 别名
- 错误序列化/反序列化（serde）与 source 链保留
- 基础工具函数与通用类型

---

## 规则

### IC1: 零外部依赖
- `infra-core` 不得依赖外部框架（tokio、actix 等）
- 只能使用标准库 + `thiserror` + `serde`
- 违反立即拒绝

### IC2: 错误优先
- 所有可能失败的函数返回 `Result<T, Error>`
- 新增错误变体须评估是否纳入 `Error` 枚举 vs 新建子类型
- Error 枚举新增变体需更新序列化/反序列化 + 文档

### IC3: 非零退出
- `Error::Config` — 配置相关错误
- `Error::InvalidArgument` — 参数校验失败
- `Error::Internal` — 不应暴露的内部错误
- `Error::Io` — I/O 操作错误（保留 source 链）

### IC4: 公共 API 稳定性
- 破坏性变更需在 PR 中声明并记录到 CHANGELOG
- 新增 `pub` 项需附带 doc-test 文档示例

---

## 目录结构

> 完整标准见父级 [`crates/AGENTS.md`](../AGENTS.md)「Crate 子模块标准布局」。

```text
crates/infra-core/
├── Cargo.toml
├── src/
│   ├── lib.rs          # 根模块（lint attrs + re-export）
│   └── error.rs        # Error 类型 + serde 实现
├── examples/           # 可运行示例（暂无时 .gitkeep）
├── docs/               # 设计/迁移文档（暂无时 .gitkeep）
├── tests/              # 集成/契约测试（暂无时 .gitkeep）
├── CHANGELOG.md
├── AGENTS.md
└── README.md
```

---

## 版本

| 版本 | 日期 | 修订 |
|------|------|------|
| v1.1.0 | 2026-07-21 | 对齐 crates 子模块标准布局并补齐骨架 |
| v1.0.0 | 2026-07-21 | 初始规则 |
