# shell:source scripts/worktree-activate.sh
#
# 加载后可使用 wt <branch> 快速切换到任意 worktree
#
# 加载方式:
#   source scripts/worktree-activate.sh
#   # 或在 ~/.bashrc 中添加:
#   # source /path/to/infra.rs/scripts/worktree-activate.sh

SCRIPT_DIR="$(dirname "${BASH_SOURCE[0]:-$0}")"
WT_SCRIPT="$SCRIPT_DIR/worktree.sh"

wt() {
    local target="$1"

    if [ -z "$target" ]; then
        # 无参数: 列出所有 worktree
        bash "$WT_SCRIPT" list
        echo ""
        echo "用法: wt <branch-name>"
        return
    fi

    if [ "$target" = "main" ]; then
        cd "$(git -C "$(dirname "$WT_SCRIPT")" rev-parse --show-toplevel 2>/dev/null || echo "$SCRIPT_DIR/..")" || return 1
        echo "→ main"
        return
    fi

    local wt_path
    wt_path=$(bash "$WT_SCRIPT" go "$target" 2>/dev/null)
    if [ -n "$wt_path" ] && [ -d "$wt_path" ]; then
        cd "$wt_path" || return 1
        echo "→ $target  ($(git branch --show-current 2>/dev/null))"
    else
        echo "wt: worktree '$target' 不存在"
        echo "  创建并切换:"
        echo "  bash $WT_SCRIPT create $target && cd .worktree/$target"
        return 1
    fi
}

# Tab 补全
_wt_complete() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local repo_root
    repo_root=$(git -C "$(dirname "$WT_SCRIPT")" rev-parse --show-toplevel 2>/dev/null || echo "$SCRIPT_DIR/..")
    COMPREPLY=($(compgen -W "$(ls "$repo_root/.worktree/" 2>/dev/null)" -- "$cur"))
}
complete -F _wt_complete wt 2>/dev/null || true
