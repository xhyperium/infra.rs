#!/usr/bin/env bash
# worktree.sh — Git Worktree 管理工具
#
# 用法:
#   ./scripts/worktree.sh create <branch>    # 创建 worktree
#   ./scripts/worktree.sh go <branch>        # 输出 cd 路径（配合 eval/source 使用）
#   ./scripts/worktree.sh list               # 列出所有 worktree
#   ./scripts/worktree.sh remove <branch>    # 删除 worktree
#   ./scripts/worktree.sh prune              # 清理无效 worktree
#   ./scripts/worktree.sh current            # 显示当前 worktree 信息

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
        echo "Worktree 已创建"
        echo "  cd $wt_path      # 或: wt $branch"
        ;;

    go)
        branch="${arg:?usage: worktree.sh go <branch>}"
        wt_path="$WT_BASE/$branch"
        if [ -d "$wt_path" ]; then
            echo "$wt_path"
        else
            echo "ERROR: worktree 不存在: $branch" >&2
            echo "  可用的 worktree:" >&2
            git -C "$REPO_ROOT" worktree list >&2
            exit 1
        fi
        ;;

    list)
        echo "Worktrees:"
        git -C "$REPO_ROOT" worktree list | while read -r line; do
            branch_name=$(echo "$line" | awk '{print $3}' | tr -d '[]')
            wt_path=$(echo "$line" | awk '{print $1}')
            if [ "$wt_path" = "$REPO_ROOT" ]; then
                echo "  [main]  $wt_path"
            else
                short="${wt_path#$WT_BASE/}"
                echo "  [$short]  $wt_path"
            fi
        done
        ;;

    remove)
        branch="${arg:?usage: worktree.sh remove <branch>}"
        wt_path="$WT_BASE/$branch"
        if [ -d "$wt_path" ]; then
            git -C "$REPO_ROOT" worktree remove "$wt_path" --force
            echo "Worktree 已删除: $branch"
        else
            echo "ERROR: worktree 不存在: $branch" >&2
            exit 1
        fi
        ;;

    prune)
        git -C "$REPO_ROOT" worktree prune
        echo "已清理过期 worktree"
        ;;

    current)
        git -C "$REPO_ROOT" worktree list | grep "$(pwd)" || true
        ;;

    *)
        echo "usage: $0 {create|go|list|remove|prune|current} [branch]"
        exit 1
        ;;
esac
