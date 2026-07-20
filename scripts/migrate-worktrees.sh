#!/usr/bin/env bash
# migrate-worktrees.sh — 迁移旧格式 worktree 路径到新规范 `.worktrees/<branch>`
#
# 迁移规则：
#   旧: .worktrees/workspaces/<branch-with-dashes>  →  新: .worktrees/<branch-with-slashes>
#   旧: ~/.worktrees/<project>/                      →  新: .worktrees/<branch>
#
# 用法:
#   ./scripts/migrate-worktrees.sh           # 列出待迁移项（dry-run）
#   ./scripts/migrate-worktrees.sh --apply   # 执行迁移

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WT_BASE="$REPO_ROOT/.worktrees"
APPLY=false

[[ "${1:-}" == "--apply" ]] && APPLY=true

echo -e "${CYAN}╔══════════════════════════════════╗${NC}"
echo -e "${CYAN}║   Worktree 路径迁移              ║${NC}"
echo -e "${CYAN}║   → .worktrees/<branch>         ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════╝${NC}"
echo ""

# ── 1. 扫描 workspaces/ 旧格式 ──────────────

WS_DIR="$WT_BASE/workspaces"
HAS_WORKSPACES=false

if [[ -d "$WS_DIR" ]]; then
    HAS_WORKSPACES=true
    echo -e "${YELLOW}发现旧 workspaces/ 目录: $WS_DIR${NC}"
    echo ""

    for item in "$WS_DIR"/*; do
        [[ -d "$item" ]] || continue
        old_path="$item"
        dir_name="$(basename "$item")"

        # 尝试从 git 获取真实分支名
        pushd "$item" > /dev/null 2>&1
        branch=$(git branch --show-current 2>/dev/null || echo "")
        popd > /dev/null 2>&1

        if [[ -n "$branch" ]]; then
            new_path="$WT_BASE/$branch"
        else
            # 无法获取分支名，保持原名但去除-分隔（保守处理）
            new_path="$WT_BASE/$dir_name"
        fi

        if $APPLY; then
            echo -e "  ${GREEN}迁移${NC}: $old_path → $new_path"
            mkdir -p "$(dirname "$new_path")"
            mv "$old_path" "$new_path"
            echo -e "  ${GREEN}  ✓ 完成${NC}"
        else
            echo -e "  ${YELLOW}待迁移${NC}: $old_path"
            echo -e "           → $new_path"
        fi
        echo ""
    done

    if $APPLY; then
        # 清理空的 workspaces/ 目录
        rmdir "$WS_DIR" 2>/dev/null && echo -e "  ${GREEN}清理: 空目录 $WS_DIR 已删除${NC}" || true
    fi
else
    echo -e "  ${GREEN}✓${NC} 无 workspaces/ 旧格式残留"
fi

# ── 2. 扫描全局 ~/.worktrees/ 旧格式 ──────────

HOME_WT="$HOME/.worktrees"
PROJECT_NAME="$(basename "$REPO_ROOT")"
GLOBAL_LEGACY="$HOME_WT/$PROJECT_NAME"

echo ""
if [[ -d "$GLOBAL_LEGACY" ]]; then
    echo -e "${YELLOW}发现全局旧路径: $GLOBAL_LEGACY${NC}"
    echo ""

    if $APPLY; then
        # 迁移全局目录下的内容
        mkdir -p "$WT_BASE"
        for item in "$GLOBAL_LEGACY"/*; do
            [[ -d "$item" ]] || continue
            dir_name="$(basename "$item")"
            new_path="$WT_BASE/$dir_name"
            echo -e "  ${GREEN}迁移${NC}: $item → $new_path"
            mv "$item" "$new_path"
            echo -e "  ${GREEN}  ✓ 完成${NC}"
        done
    else
        echo -e "  ${YELLOW}手动迁移步骤:${NC}"
        echo "    mkdir -p '$WT_BASE'"
        echo "    mv '$GLOBAL_LEGACY'/* '$WT_BASE/'"
        echo "    rmdir '$GLOBAL_LEGACY'"
    fi
    echo ""
else
    echo -e "  ${GREEN}✓${NC} 无 ~/.worktrees/ 全局旧格式残留"
fi

# ── 3. 最终状态 ──────────────────────────────

echo ""
echo -e "${CYAN}─── 当前 Worktree 列表 ───${NC}"
git -C "$REPO_ROOT" worktree list 2>/dev/null || true

if ! $APPLY; then
    echo ""
    echo -e "${YELLOW}这是 dry-run 模式。执行迁移:${NC}"
    echo -e "  ${CYAN}./scripts/migrate-worktrees.sh --apply${NC}"
fi
