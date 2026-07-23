# .gitmessage 提交信息模板使用说明

## 概述

`.gitmessage` 是仓库根目录的 Git 提交信息模板。激活后，每次执行 `git commit`（不带 `-m`）时自动加载到编辑器中，引导开发者按 [Conventional Commits](https://www.conventionalcommits.org/) 规范编写提交说明。

## 激活

首次克隆仓库后执行一条命令即可激活：

```bash
git config commit.template .gitmessage
```

此命令将模板路径写入仓库级 `.git/config`，仅对当前克隆生效。团队其他成员各自克隆后同样需要执行一次。

验证是否激活：

```bash
git config commit.template
# 输出应为: .gitmessage
```

## 模板结构

模板分为四个区域：

### 标题行（必填）

```text
<type>[!][(scope)]: <中文简述>
```

| 字段 | 说明 | 示例 |
|------|------|------|
| `type`（必选） | 变更类型 | `feat`、`fix`、`chore`、`docs`、`refactor`、`test`、`perf`、`ci`、`style`、`build` |
| `!`（可选） | 破坏性变更标记 | `feat!:` 或 `fix!:` |
| `scope`（可选） | 影响范围 | `kernel`、`configx`、`bootstrap` |
| `简述`（必选） | 一句话中文描述 | _空格 + 中文说明_ |

### 正文（可选）

与标题行之间**空一行**。叙述变更原因、影响范围、注意事项。

### 破坏性变更声明（条件必填）

当标题行带有 `!` 时，正文中必须包含：

```text
BREAKING CHANGE: <说明具体破坏内容及迁移方式>
```

### 引用（可选）

```text
Refs: #123
```

或关联多个 Issue：

```text
Refs: #123, #456
```

## 使用方式

```bash
# 1. 暂存变更
git add <files>

# 2. 不带 -m 提交 → 自动加载模板到编辑器
git commit

# 3. 按模板编写，完成后保存并关闭编辑器
```

模板行以 `#` 开头，不会被 Git 记录到提交信息中。可直接在模板上填写，或删除注释行后编写。

如果只需一行简述，也可直接使用 `-m`（此时模板不加载）：

```bash
git commit -m "chore: 更新 Rust 工具链为 1.85"
```

## 完整示例

### 普通功能提交

```text
feat(kernel): 新增生命周期 on_stop 回调

组件销毁前触发 on_stop，允许外部在清理阶段注入收尾逻辑。
该回调在 on_close 之前、资源释放之前触发，保证收尾时依赖仍可用。
```

### 破坏性变更提交

```text
feat!(contracts): Exchange trait 新增 validate_order 方法

BREAKING CHANGE: 所有 Exchange 实现者（binancex、okxx）必须实现
validate_order。迁移方式：为现有实现添加方法签名并返回 Ok(()) 即可。
```

### 修复提交

```text
fix(schedulex): 修复任务登记表重复分配

两个并发 caller 在 claim_task 时可能分配到同一个 ID，
原因是 AtomicCounter 的 compare_exchange 未正确处理弱序失败。
改为 load→fetch_add→store 三段式，结合 Acquire/Release 内存序。
```

### 文档提交

```text
docs: 将 .gitmessage 模板规则写入宪章与 AGENTS.md

- docs/constitution/04-code-standards.md §4.3.3：补充模板激活说明
- AGENTS.md 检查清单：附加激活命令
```

## 常见问题

### 模板不显示？

确认仓库根存在 `.gitmessage` 文件，且 `git config commit.template` 输出正确。

### 想使用自己的编辑器？

`git commit` 启动的编辑器由 `core.editor` 或 `$EDITOR` 环境变量控制：

```bash
git config core.editor "vim"
# 或
export EDITOR=vim
```

### 模板被意外修改？

`.gitmessage` 已纳入版本控制。如本地被误改，可用 `git checkout .gitmessage` 恢复。

### 想在特定仓库禁用模板？

```bash
git config --unset commit.template
```

## 相关

- [Conventional Commits 规范](https://www.conventionalcommits.org/)
- [工程宪章 §4.3.3](../../docs/constitution/04-code-standards.md#433-分支与标签)
- [编码与语言约定](编码与语言约定.md)
