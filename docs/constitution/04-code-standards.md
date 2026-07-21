# 四、代码标准

## 4.1 格式

- 统一使用 `rustfmt`，配置见 `rustfmt.toml`
- CI 中 `cargo fmt --check` 必须通过
- 不讨论格式风格，工具即标准

## 4.2 Lint

- 启用 `clippy`，`-D warnings`
- 禁止 `#[allow(...)]` 无注释说明
- `unsafe` 代码须标注原因并附带 safety proof 注释

## 4.3 命名

### 4.3.1 Rust 标识符

- crate 名：`kebab-case`，Cargo 包名与目录名一致
- 类型/枚举：`UpperCamelCase`（`BinanceAdapter`, `OrderSide`）
- 函数/方法/变量：`snake_case`（`fetch_ticker`, `base_url`）
- 常量/静态变量：`SCREAMING_SNAKE_CASE`（`MAX_POSITION_SIZE`）
- 测试函数：`snake_case`，描述行为而非实现（`test_connect_disconnect`）

### 4.3.2 Crate 命名

| 类型 | 包名 (Cargo.toml) | 目录 | 示例 |
|------|-------------------|------|------|
| 核心 crate | `<domain>` 或 `<domain>x` | `crates/<domain>/` | `kernel`, `configx` |
| 适配器 | `<provider>x` | `crates/adapters/<kind>/<provider>/` | `binancex`, `redisx` |

**规则**：

- **`x` 后缀**：推荐新 crate 以 `x` 结尾（xhyper extension）
- **无前缀**：包名不含 `xhyper-` 前缀
- **目录与包名一致**：`crates/configx/` → 包名 `configx`
- **适配器**：统一 `{provider}x` 模式，目录保持 `crates/adapters/{kind}/{provider}/`

### 4.3.3 分支与标签

- **分支**：`{type}/{description}`，type ∈ `feat | fix | chore | docs | test | refactor`
  - 例：`feat/order-balance`, `fix/miri-isolation`, `chore/update-deps`
- **标签**：`v{MAJOR}.{MINOR}.{PATCH}`（[SemVer](https://semver.org/)）
  - 例：`v0.3.0`, `v1.0.0`
- **commit**：[Conventional Commits](https://www.conventionalcommits.org/)
  - 例：`feat(binancex): add order management scaffold`

### 4.3.4 文件与目录

- **脚本**：`.mjs`（ESM），禁止 `.sh`（[§4.8](#48-脚本语言ecmascript-module强制)）
- **配置**：`.toml` 优先，避免 `.yaml` / `.json` 碎片化
- **Markdown**：仓库根级标志文件可用 `SCREAMING_SNAKE_CASE.md`（`CHANGELOG.md`）；宪章正文分章见本目录
- **Rust 模块**：`snake_case.rs`
- **Cargo 包目录**：与包名一致（`crates/configx/` → 包名 `configx`）

## 4.4 测试

- 单元测试与源码同文件，置于 `#[cfg(test)] mod tests`
- 集成测试置于 `tests/` 目录
- 测试命名描述行为，而非实现细节
- 优先使用 `cargo-nextest` 作为测试运行器

## 4.5 语言与编码（强制）

本仓库对**文本语言与字符编码**作出强制约定。细则见 [docs/governance/编码与语言约定.md](../governance/编码与语言约定.md)；冲突时以本宪章为准。

### 4.5.1 字符编码

- 全部文本源文件必须为 **UTF-8（无 BOM）**
- 换行符统一为 **LF**（Unix）
- 禁止提交 GBK / GB2312 / UTF-16 等其他编码
- 禁止出现替换字符 `U+FFFD`（表示编码损坏）
- 编辑器配置以 `.editorconfig` 的 `charset = utf-8` 为准

### 4.5.2 语言

| 类别 | 要求 |
|------|------|
| 代码注释（`//`、`///`、`//!`） | **中文** |
| 项目治理 / 协作文档（宪章、AGENTS、PR/Issue 模板等） | **中文** |
| 用户可见错误信息（`Display` / 业务文案） | **中文** |
| **英文技术文档**（手册、API 英文说明、运维英文 runbook 等） | **ASD-STE100（[§4.6](#46-文档标准asd-ste100强制)）** |
| 标识符（类型、函数、模块、字段名） | 英文（Rust 惯例） |
| 提交说明 | 中文，或 Conventional Commits（英文 type + 中文说明） |
| `LICENSE` 等法律文本 | 英文原文 |
| 第三方 skills / 上游文档 | 可保留原文；**新增中文内容优先中文；新增英文技术正文适用 STE** |

### 4.5.3 技术术语

- 可保留英文术语本体：API、CI、PR、crate、workspace、Docker 等
- 中文叙述中的解释性语句使用中文
- 禁止对已是 UTF-8 的中文再次错误转码（避免双重 UTF-8 / 乱码）

### 4.5.4 合规检查

- 本地 / CI 应能检测：非 UTF-8、`U+FFFD`、明显双重编码痕迹
- 宪章校验脚本：`./scripts/quality-gates/check-constitution.mjs` 包含 §4.5 检查

## 4.6 文档标准：ASD-STE100（强制）

**ASD-STE100**（*Simplified Technical English*，简化技术英语，简称 **STE**）是用于编写技术文档的**受控自然语言**与国际通行规范。  
本仓库将 **ASD-STE100 作为全局英文技术文档标准**。

> 落地指南见 [docs/governance/ASD-STE100.md](../governance/ASD-STE100.md)。  
> 官方规范受版权保护；本宪章只规定**适用边界与强制原则**，不复制官方词表全文。

### 4.6.1 适用范围

以下类型的**英文**文本必须符合 STE（或项目批准的 STE 兼容子集）：

- 用户 / 运维 / 集成类技术手册与 runbook（英文版）
- 对外 API 的英文说明与操作步骤
- 可交付的英文故障排查、安装、配置说明
- crate / 产品的**对外英文 README 技术正文**（非法律文本）

**不适用**（仍遵循 [§4.5](#45-语言与编码强制)）：

- 中文治理与协作文档
- 代码注释（中文）
- 标识符与纯代码
- `LICENSE` 等法律原文
- 已存在的第三方英文 skills 原文（新增英文技术交付物时适用 STE）

### 4.6.2 强制原则（摘要）

英文技术文档至少满足：

1. **一词一义** — 同一词不得在文中切换含义；术语全文一致  
2. **短句** — 一句一个主题；描述句宜短；避免深层嵌套从句  
3. **语态与时态** — 描述优先主动语态 + 简单现在时；操作步骤用祈使语气  
4. **步骤可执行** — 程序类内容用编号步骤；一步一动作  
5. **警告在前** — Warning / Caution / Note 出现在相关操作之前  
6. **可翻译** — 避免俚语、双关、文化隐喻与不必要的缩写堆叠  

### 4.6.3 与中文文档的关系

- **双轨制**：中文管协作与项目内说明；英文技术交付用 STE  
- 中英双语同一主题时，**术语与步骤顺序必须一致**  
- 中文文档借鉴 STE 精神：短句、一步一事、少歧义（不强制 STE 英文词表）

### 4.6.4 AI 与审查

- AI 撰写英文技术文档时必须按 §4.6 自检（见 `docs/governance/ASD-STE100.md` 清单）  
- 审查英文技术 PR 时，审查者应抽查 STE 合规（词汇一致、句长、步骤结构）  
- 完整词典与规则集以官方 ASD-STE100 版本为准；项目指南不得与官方冲突

### 4.6.5 合规检查

- 宪章校验脚本检查：宪章目录含 §4.6 条款、`docs/governance/ASD-STE100.md` 存在  
- 深度 STE 词表校验不强制自动化（依赖官方工具/人工）；结构与原则抽查为强制审查义务

## 4.8 脚本语言：ECMAScript Module（强制）

**`scripts/` 下的自动化脚本统一使用 `.mjs`（ECMAScript Module）。**

- 禁止新增 `.sh` / `.bash` 脚本
- 现有 `.sh` 脚本须逐步迁移至 `.mjs`
- **例外**（须 shell 集成）：
  - `worktree-activate.mjs` — 需 `source` 注入函数与补全
  - `starship-wt.mjs` — Starship 调用（子进程，轻量）
  - `worktree.mjs` — 需 `cd` 改变 shell 状态

理由：跨平台兼容、统一依赖（Node.js ≥ 18）、类型安全潜力。

---

← [上一章：架构原则](./03-architecture.md) · [索引](./README.md) · 下一章：[五、质量门禁](./05-quality-gates.md) →
