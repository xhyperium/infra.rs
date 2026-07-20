# CONSTITUTION.md — 工程宪章

本文件定义 `infra.rs` 项目的核心价值观、架构原则与工程纪律。所有参与者（人、AI、自动化）均受此宪章约束。

---

## 一、使命

`infra.rs` 是 [xhyper.rs](https://github.com/xhyperium/xhyper.rs) Rust HTTP 框架的**基础设施与治理仓库**，承载 CI/CD、AI 代理行为规范、构建标准与工程约定的单一事实源（SSOT）。

**核心理念**：正确的流程产生正确的代码。

---

## 二、核心价值观

### 2.1 安全优先
- 任何变更不得降低安全标准
- 依赖项须通过 `cargo-deny` 审计
- 敏感信息不得入库（密钥、token、证书）

### 2.2 可观测
- 关键路径必须有日志/指标/追踪
- 错误必须可追溯，不得吞没
- CI 状态必须透明、可解释

### 2.3 可验证
- 每条断言必须可被测试验证
- `cargo check` / `cargo test` / `cargo fmt --check` 是门禁底线
- 覆盖率不低于 80%（核心模块 95%）

### 2.4 自动化优先
- 重复操作优先脚本化
- CI 是唯一的真理仲裁者
- 能由机器保证的，不依赖人工审查

### 2.5 简单优于灵活
- 默认拒绝过度抽象
- 每增加一层间接，必须有可论证的收益
- YAGNI：先实现需求，再抽象模式

---

## 三、架构原则

### 3.1 模块边界
```
crates/
├── infra-core/     # 核心基础库（错误类型、Result 别名、工具函数）
├── ...             # 后续按需添加
```

- 每个 crate 有单一明确的职责
- 依赖方向：上层依赖下层，禁止循环引用
- `core` 层不得依赖外部框架或平台特定代码

### 3.2 接口设计
- 公共 API 必须有文档注释（`///`）
- 文档注释中的代码示例必须可编译（doc-test）
- 破坏性变更必须经过 deprecation 周期

### 3.3 错误处理
- 使用 `thiserror` 定义明确错误类型
- 错误链（`source()`）不可断裂
- `unwrap()` / `expect()` 仅在不可恢复或已证明不可能出错的场景使用

---

## 四、代码标准

### 4.1 格式
- 统一使用 `rustfmt`，配置见 `rustfmt.toml`
- CI 中 `cargo fmt --check` 必须通过
- 不讨论格式风格，工具即标准

### 4.2 Lint
- 启用 `clippy`，`-D warnings`
- 禁止 `#[allow(...)]` 无注释说明
- `unsafe` 代码须标注原因并附带 safety proof 注释

### 4.3 命名
- crate 名：`kebab-case`
- 类型/枚举：`UpperCamelCase`
- 函数/方法/变量：`snake_case`
- 常量/静态变量：`SCREAMING_SNAKE_CASE`

### 4.4 测试
- 单元测试与源码同文件，置于 `#[cfg(test)] mod tests`
- 集成测试置于 `tests/` 目录
- 测试命名描述行为，而非实现细节
- 优先使用 `cargo-nextest` 作为测试运行器

### 4.5 语言与编码（强制）

本仓库对**文本语言与字符编码**作出强制约定。细则见 [docs/编码与语言约定.md](./docs/编码与语言约定.md)；冲突时以本宪章为准。

#### 4.5.1 字符编码
- 全部文本源文件必须为 **UTF-8（无 BOM）**
- 换行符统一为 **LF**（Unix）
- 禁止提交 GBK / GB2312 / UTF-16 等其他编码
- 禁止出现替换字符 `U+FFFD`（``）—— 表示编码损坏
- 编辑器配置以 `.editorconfig` 的 `charset = utf-8` 为准

#### 4.5.2 语言
| 类别 | 要求 |
|------|------|
| 代码注释（`//`、`///`、`//!`） | **中文** |
| 项目文档（`*.md`、PR/Issue 模板） | **中文** |
| 用户可见错误信息（`Display` / 业务文案） | **中文** |
| 标识符（类型、函数、模块、字段名） | 英文（Rust 惯例） |
| 提交说明 | 中文，或 Conventional Commits（英文 type + 中文说明） |
| `LICENSE` 等法律文本 | 英文原文 |
| 第三方 skills / 上游文档 | 可保留原文；**新增内容优先中文** |

#### 4.5.3 技术术语
- 可保留英文术语本体：API、CI、PR、crate、workspace、Docker 等
- 解释性语句与叙述使用中文
- 禁止对已是 UTF-8 的中文再次错误转码（避免双重 UTF-8 / 乱码）

#### 4.5.4 合规检查
- 本地 / CI 应能检测：非 UTF-8、`U+FFFD`、明显双重编码痕迹
- 宪章校验脚本：`./scripts/check-constitution.sh` 包含 §4.5 检查

---

## 五、质量门禁

| 门禁 | 级别 | 说明 | 工作流 / 命令 |
|------|------|------|---------------|
| `cargo fmt --check` | **强制** | 格式一致性 | `validation.yml` / `make fmt-check` |
| `cargo clippy -- -D warnings` | **强制** | 代码质量 | `quality.yml` / `make lint` |
| `cargo test` / `cargo nextest run` | **强制** | 功能正确性 | `ci-rust.yml` / `make test` |
| `cargo-deny check` | **强制** | 安全审计 | `security.yml` / `make deny` |
| 宪章合规性（全部） | **强制** | `scripts/check-constitution.sh` | `constitution.yml` / `make check` |
| UTF-8 / 无 `U+FFFD`（§4.5） | **强制** | 编码完整性 | `constitution.yml` (已包含) |
| 覆盖率 >= 80% | **推荐** | 代码覆盖 | `ci-rust.yml` |
| `cargo-llvm-cov` | **推荐** | 覆盖率统计 | `ci-rust.yml` |

### 5.1 本地验证

提交 PR 前运行 `make ci` 模拟全部强制门禁：

```bash
make ci    # 等价于: make fmt-check lint test deny
make check # 等效: ./scripts/check-constitution.sh
```

---

## 六、治理

### 6.1 变更流程
1. **Issue** — 描述问题或提案
2. **PR** — 包含变更、测试、文档
3. **审查** — 至少一人 approve
4. **CI** — 所有强制门禁通过
5. **合并** — squash merge 到 `main`

### 6.2 版本策略
- 遵循语义化版本 [SemVer](https://semver.org/)
- `0.x.y` 期间不保证向后兼容
- `1.0.0` 后严格 SemVer

### 6.3 所有权
- 代码所有权由 `.github/CODEOWNERS` 定义
- 架构决策记录于 `docs/decisions/`（ADR 格式）

---

## 七、AI 代理章程

### 7.1 权限边界
- AI **不可** approve 或 merge PR
- AI **不可** 直接推送 `main` 分支
- AI **不可** 修改 `.github/CODEOWNERS`
- AI **不可** 绕过任何强制门禁

### 7.2 职责范围
- AI 可执行：编码、测试编写、代码审查建议、文档生成、issue 分类
- AI 不可执行：审批、合并、发布、权限变更、CI 配置修改（需人工审查）

### 7.3 输出标准
- AI 生成的代码须与手工代码同等质量
- AI 须明确标注不确定的部分
- AI 修改后须运行 `cargo test` + `cargo fmt --check` + `cargo clippy`
- AI 产出的**注释、文档、用户可见错误信息**须为**中文**（§4.5）
- AI 写入的文本文件须为 **UTF-8 无 BOM**；不得引入乱码或 ``

---

## 八、修订

### 8.1 提议
任何参与者可通过 PR 提议修订本宪章。

### 8.2 批准
- 修订需多数 maintainer approve
- 实质性变更须附带迁移计划

### 8.3 版本
本宪章遵循独立版本号，记录修订历史：

| 版本 | 日期 | 修订内容 |
|------|------|----------|
| v1.1.0 | 2026-07-21 | 新增 §4.5 语言与编码（中文注释/文档 + UTF-8 强制） |
| v1.0.0 | 2026-07-21 | 初始版本 |

---

> *好的流程不是限制，而是解放 — 它消除不确定，让创造力聚焦于真正的问题。*
