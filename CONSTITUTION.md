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
| 项目治理 / 协作文档（宪章、AGENTS、PR/Issue 模板等） | **中文** |
| 用户可见错误信息（`Display` / 业务文案） | **中文** |
| **英文技术文档**（手册、API 英文说明、运维英文 runbook 等） | **ASD-STE100（§4.6）** |
| 标识符（类型、函数、模块、字段名） | 英文（Rust 惯例） |
| 提交说明 | 中文，或 Conventional Commits（英文 type + 中文说明） |
| `LICENSE` 等法律文本 | 英文原文 |
| 第三方 skills / 上游文档 | 可保留原文；**新增中文内容优先中文；新增英文技术正文适用 STE** |

#### 4.5.3 技术术语
- 可保留英文术语本体：API、CI、PR、crate、workspace、Docker 等
- 中文叙述中的解释性语句使用中文
- 禁止对已是 UTF-8 的中文再次错误转码（避免双重 UTF-8 / 乱码）

#### 4.5.4 合规检查
- 本地 / CI 应能检测：非 UTF-8、`U+FFFD`、明显双重编码痕迹
- 宪章校验脚本：`./scripts/check-constitution.sh` 包含 §4.5 检查

### 4.6 文档标准：ASD-STE100（强制）

**ASD-STE100**（*Simplified Technical English*，简化技术英语，简称 **STE**）是用于编写技术文档的**受控自然语言**与国际通行规范。  
本仓库将 **ASD-STE100 作为全局英文技术文档标准**。

> 落地指南见 [docs/ASD-STE100.md](./docs/ASD-STE100.md)。  
> 官方规范受版权保护；本宪章只规定**适用边界与强制原则**，不复制官方词表全文。

#### 4.6.1 适用范围
以下类型的**英文**文本必须符合 STE（或项目批准的 STE 兼容子集）：

- 用户 / 运维 / 集成类技术手册与 runbook（英文版）
- 对外 API 的英文说明与操作步骤
- 可交付的英文故障排查、安装、配置说明
- crate / 产品的**对外英文 README 技术正文**（非法律文本）

**不适用**（仍遵循 §4.5）：

- 中文治理与协作文档
- 代码注释（中文）
- 标识符与纯代码
- `LICENSE` 等法律原文
- 已存在的第三方英文 skills 原文（新增英文技术交付物时适用 STE）

#### 4.6.2 强制原则（摘要）
英文技术文档至少满足：

1. **一词一义** — 同一词不得在文中切换含义；术语全文一致  
2. **短句** — 一句一个主题；描述句宜短；避免深层嵌套从句  
3. **语态与时态** — 描述优先主动语态 + 简单现在时；操作步骤用祈使语气  
4. **步骤可执行** — 程序类内容用编号步骤；一步一动作  
5. **警告在前** — Warning / Caution / Note 出现在相关操作之前  
6. **可翻译** — 避免俚语、双关、文化隐喻与不必要的缩写堆叠  

#### 4.6.3 与中文文档的关系
- **双轨制**：中文管协作与项目内说明；英文技术交付用 STE  
- 中英双语同一主题时，**术语与步骤顺序必须一致**  
- 中文文档借鉴 STE 精神：短句、一步一事、少歧义（不强制 STE 英文词表）

#### 4.6.4 AI 与审查
- AI 撰写英文技术文档时必须按 §4.6 自检（见 `docs/ASD-STE100.md` 清单）  
- 审查英文技术 PR 时，审查者应抽查 STE 合规（词汇一致、句长、步骤结构）  
- 完整词典与规则集以官方 ASD-STE100 版本为准；项目指南不得与官方冲突

#### 4.6.5 合规检查
- 宪章校验脚本检查：`CONSTITUTION.md` 含 §4.6、`docs/ASD-STE100.md` 存在  
- 深度 STE 词表校验不强制自动化（依赖官方工具/人工）；结构与原则抽查为强制审查义务

---

## 五、质量门禁

| 门禁 | 级别 | 说明 | 工作流 / 命令 |
|------|------|------|---------------|
| `cargo fmt --check` | **强制** | 格式一致性 | `validation.yml` / `make fmt-check` |
| `cargo clippy -- -D warnings` | **强制** | 代码质量 | `quality.yml` / `make lint` |
| `cargo test` / `cargo nextest run` | **强制** | 功能正确性 | `ci-rust.yml` / `make test` |
| `cargo-deny check` | **强制** | 安全审计 | `security.yml` / `make deny` |
| 宪章合规性（全部） | **强制** | `scripts/check-constitution.sh` | `constitution.yml` / `make check` |
| UTF-8 / 无 `U+FFFD`（§4.5） | **强制** | 编码完整性 | `constitution.yml`（已包含） |
| Git Main First（§6.0） | **强制** | 主干唯一、PR 收敛 | 分支保护 + 宪章脚本条款检查 |
| ASD-STE100（§4.6） | **强制** | 英文技术文档受控语言 | 审查清单 + `docs/ASD-STE100.md` |
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

### 6.0 Git Main First（强制）

**Main First**：`main` 是唯一长期真实主干与集成分支。一切有价值的工作必须周期性收敛至 `main`，禁止长期并行「第二真相」。

#### 6.0.1 主干唯一
- 默认分支名为 **`main`**
- 禁止维护与 `main` 长期分叉、互不合并的并行主线
- 功能分支、修复分支均为**短期**；合并后应及时删除远程与本地分支
- **「长期」定义**：超过 **30 天**未向 `main` 合并的功能分支视为违规并行（release 维护分支须在文档中显式声明生命周期）

#### 6.0.2 禁止在 main 上直接开发
- **禁止**在 `main` 上直接编码、提交、推送
- 合法路径唯一：`feature / fix / chore 分支 → PR → 审查 → CI 通过 → 合并入 main`
- 紧急热修复也须走 PR；可缩短审查窗口，但**不可**跳过门禁与合并路径

#### 6.0.3 分支与同步
- 新分支须从**最新** `origin/main` 创建（先 `git fetch` + 基于最新 main 建支）
- 开发中定期与 `main` 同步；首选 **rebase** 保持线性历史，冲突过大时可用 merge 并在 PR 中说明
- PR 合并前须相对 `main` 可合并（无未解决冲突）
- 默认 **squash merge** 进 `main`，保持主干历史清晰

#### 6.0.4 推送与保护
- `main` 受分支保护：禁止 force push、禁止绕过 CI 的直推
- 禁止 `git push --force` / `git push --force-with-lease` 到 `main`
- 禁止 `git push --no-verify` 绕过钩子向共享分支推送
- 历史重写（orphan / force push 覆盖远程）仅在**维护者明确授权**且团队知情时执行

#### 6.0.5 工作区隔离（推荐 / substantial 强制）
- 实质性任务（多文件或大 diff）应使用独立分支；推荐 git worktree 隔离
- Worktree 规范路径：`.worktree/workspaces/<branch-name>`（分支名中的 `/` 可映射为 `-`）
- 禁止多任务混用同一 worktree / 分支

#### 6.0.6 与 AI 协作
- AI **不得**在 `main` 检出上直接改业务代码并提交
- AI 开工前应确认当前分支 ≠ `main`（或仅作只读说明 / 文档性例外须人工明示）
- Session 钩子（如 `branch-protect`）可告警，但**宪章效力不依赖钩子是否启用**

#### 6.0.7 一句话
> **先对齐 main，再开分支；先 PR 进 main，再谈完成。**

#### 6.0.5 分支保护验证

分支保护已启用 `enforce_admins` 测试，并记录于本宪章：

- **开启 `enforce_admins: true`** 时：管理员直接推送 `main` 被拒绝，验证规则正确性
- **生产环境**：保持 `enforce_admins: false`，允许管理员应急绕过（如紧急热修复跳过 CI 等待），但需在 PR 中注明绕过原因
- 验证结果（2026-07-21）：
  ```
  remote: error: GH006: Protected branch update failed for refs/heads/main.
  remote: - Changes must be made through a pull request.
  remote: - 2 of 2 required status checks are expected.
  ! [remote rejected] main -> main (protected branch hook declined)
  ```

### 6.1 变更流程
1. **从 main 同步** — `fetch` 最新主干并建分支（§6.0）
2. **Issue** — 描述问题或提案（可追溯）
3. **PR** — 包含变更、测试、文档
4. **审查** — 至少一人 approve（或项目规定的 maintainer 规则）
5. **CI** — 所有强制门禁通过
6. **合并** — squash merge 到 `main`
7. **清理** — 删除已合并分支；必要时同步本地 main

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
- AI **不可** 直接推送 `main` 分支（§6.0 Git Main First）
- AI **不可** 在 `main` 上直接开发或提交（§6.0.2）
- AI **不可** 修改 `.github/CODEOWNERS`
- AI **不可** 绕过任何强制门禁

### 7.2 职责范围
- AI 可执行：编码、测试编写、代码审查建议、文档生成、issue 分类
- AI 不可执行：审批、合并、发布、权限变更、CI 配置修改（需人工审查）

### 7.3 输出标准
- AI 生成的代码须与手工代码同等质量
- AI 须明确标注不确定的部分
- AI 修改后须运行 `cargo test` + `cargo fmt --check` + `cargo clippy`
- AI 产出的**注释、中文文档、用户可见错误信息**须为**中文**（§4.5）
- AI 产出的**英文技术文档**须符合 **ASD-STE100**（§4.6）
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
| v1.3.0 | 2026-07-21 | 新增 §4.6 ASD-STE100 作为全局英文技术文档标准 |
| v1.2.0 | 2026-07-21 | 新增 §6.0 Git Main First（主干唯一、禁 main 直推、PR 收敛） |
| v1.1.0 | 2026-07-21 | 新增 §4.5 语言与编码（中文注释/文档 + UTF-8 强制） |
| v1.0.0 | 2026-07-21 | 初始版本 |

---

> *好的流程不是限制，而是解放 — 它消除不确定，让创造力聚焦于真正的问题。*
