# Git Worktree 强制开发策略

> 本文档为 `CONSTITUTION.md §6.0.6` 的实施细则。
> 工具脚本：`scripts/worktree.sh`

---

## 原则

**所有活跃开发必须在独立的 Git Worktree 中进行。** 禁止在 `main` 工作区直接创建或切换功能分支。

---

## 理由

- **隔离**：每个 branch 有独立的工作目录，避免文件交叉污染
- **并行**：可同时开发多个特性，无需 stash / checkout 切换
- **稳定**：`main` 工作区始终保持干净，可直接用于 review / 构建
- **安全**：不会因忘记切换分支而误改 main

---

## 强制规则

### 1. 创建 Worktree

新功能开发必须通过 Worktree 创建：

```bash
# 方式 A: 使用脚本
./scripts/worktree.sh create feat/my-feature

# 方式 B: 手动
git worktree add .worktrees/feat/my-feature -b feat/my-feature origin/main
```

### 2. 目录约定

```
infra.rs/                      # 主工作区 (main)
└── .worktrees/                 # Worktree 根（已 gitignore）
    ├── feat/                  # 功能分支
    │   └── my-feature/
    ├── fix/                   # 修复分支
    │   └── bug-123/
    └── chore/                 # 杂项分支
        └── update-deps/
```

### 3. 禁止行为

- 禁止在 main 工作区 `git checkout -b <feature>`
- 禁止在 main 工作区 `git switch <feature>`
- 禁止在 main 工作区执行 `cargo build/test` 以外的写操作
- Worktree 仅用于开发，不得用于 `cargo publish` 等发布操作

### 4. 清理

合并后删除 Worktree：

```bash
./scripts/worktree.sh remove feat/my-feature
./scripts/worktree.sh prune    # 清理残留
```

---

## CI 检查

钩子脚本 `scripts/worktree.sh` 将在 session 启动时验证当前工作区是否符合规则。

## 例外

- 紧急热修复可通过 PR 审查后直接操作 main（须记录原因）
- 单次微小修改（如 typo fix）可由 maintainer 在 main 工作区完成
