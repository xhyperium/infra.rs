# crates/ — Agent 行为规则

> 本文件定义 AI 代理在本仓库 Rust workspace crates 中的行为规范。
> SSOT 源：`.agents/ssot/SSOT.md`、`CONSTITUTION.md`。

---

## 适用范围

本文件覆盖 `crates/` 目录下所有 Rust crate 的 AI 代理操作规则。

---

## 规则

### C1: 遵循宪章
- 所有代码变更必须符合 `CONSTITUTION.md` 规范
- 提交前运行 `make ci` 验证强制门禁

### C2: 模块边界
- 新增 crate 前先评估：是否可以用现有 crate 的模块替代
- crate 间依赖方向单向，禁止循环引用
- `infra-core` 是基础层，不得依赖其他 workspace crate

### C3: 错误处���
- 库代码禁止裸 `unwrap()` / `expect()`
- 使用 `thiserror` 定义 crate 专用错误类型
- 错误链不可断裂，保留 `source()`

### C4: 测试
- 每个公开函数至少一个单元测试
- 测试置于 `#[cfg(test)] mod tests` 模块
- doc-test 必须可编译运行

### C5: 文档
- 每个 `pub` 项有 `///` 注释
- 每个 `mod.rs` / `lib.rs` 顶部有 `//!` 模块文档
- 文档注释中的示例代码用 `` ``` `` 标记并确保可运行

### C6: unsafe
- 禁止在 infra-core 中使用 `unsafe`
- 如未来需要，必须封装在安全抽象中并附 SAFETY 注释

---

## 禁止行为

- 在 main 分支直接修改文件（须走 PR 流程）
- 提交含有 `todo!()` 的代码（须关联 issue）
- 使用 `as` 进行类型转换（用 From/TryFrom）
- 修改 `.cargo/config.toml` 未经明确授权

---

## Crate 概览

| Crate | 路径 | 职责 |
|-------|------|------|
| `infra-core` | `crates/infra-core/` | 核心错误类型、Result 别名、基础工具 |

---

## 版本

| 版本 | 日期 | 修订 |
|------|------|------|
| v1.0.0 | 2026-07-21 | 初始代理规则 |
