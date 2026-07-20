#!/usr/bin/env bash
# starship-wt.sh — 输出 worktree 状态供 Starship 自定义模块使用
#
# 用法: 在 starship.toml 中配置 [custom.wt] module 调用此脚本
#
# 输出:
#   main       — 在 infra.rs 主工作区
#   feat/xxx   — 在 worktree 中
#   (无输出)    — 不在 infra.rs 仓库中

CURRENT="$PWD"

# 查找仓库根（Git 感知，无需硬编码路径）
REPO_ROOT=$(git -C "$CURRENT" rev-parse --show-toplevel 2>/dev/null) || exit 0

# 仅在 infra.rs 仓库内
if [[ "$(basename "$REPO_ROOT")" != "infra.rs" ]]; then
    exit 0
fi

# 在 worktree 中
if [[ "$CURRENT" == "$REPO_ROOT/.worktree/"* ]]; then
    rel="${CURRENT#$REPO_ROOT/.worktree/}"
    echo "${rel%%/*}"
    exit 0
fi

# 在 main 工作区
echo "main"
