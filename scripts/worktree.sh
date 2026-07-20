#!/usr/bin/env bash
# worktree.sh — Git Worktree 管理工具
#
# 用法:
#   ./scripts/worktree.sh create <branch>    # 创建 worktree
#   ./scripts/worktree.sh list               # 列出所有 worktree
#   ./scripts/worktree.sh remove <branch>    # 删除 worktree
#   ./scripts/worktree.sh prune              # 清理无效 worktree

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WT_BASE="$REPO_ROOT/.worktree"

cmd="${1:-}"
arg="${2:-}"

case "$cmd" in
    create)
        branch="${arg:?usage: worktree.sh create <branch>}"
        wt_path="$WT_BASE/$branch"
        mkdir -p "$WT_BASE"
        git -C "$REPO_ROOT" fetch origin
        git -C "$REPO_ROOT" worktree add "$wt_path" -b "$branch" "origin/main"
        echo "Worktree created: $wt_path"
        echo "cd $wt_path"
        ;;

    list)
        git -C "$REPO_ROOT" worktree list
        ;;

    remove)
        branch="${arg:?usage: worktree.sh remove <branch>}"
        wt_path="$WT_BASE/$branch"
        git -C "$REPO_ROOT" worktree remove "$wt_path" --force
        echo "Worktree removed: $wt_path"
        ;;

    prune)
        git -C "$REPO_ROOT" worktree prune
        echo "Pruned stale worktrees"
        ;;

    *)
        echo "usage: $0 {create|list|remove|prune} [branch]"
        exit 1
        ;;
esac
