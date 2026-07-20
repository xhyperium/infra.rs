# shell:source scripts/worktree-activate.sh
#
# 加载后可使用 wt <branch> 快速切换到任意 worktree，并在 PROMPT 显示当前 worktree。
#
# 加载方式:
#   source scripts/worktree-activate.sh
#   # 或在 ~/.bashrc 中添加:
#   # source /path/to/infra.rs/scripts/worktree-activate.sh
#
# 环境变量:
#   WT_PROMPT=1     启用 worktree 提示（默认启用）
#   WT_PROMPT_COLOR 提示颜色（默认 36=cyan）
#   WT_PROMPT_STYLE "icon" 使用图标; "short" 简写; "full" 全路径

SCRIPT_DIR="$(dirname "${BASH_SOURCE[0]:-$0}")"
WT_SCRIPT="$SCRIPT_DIR/worktree.sh"

REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# ── wt 命令 ──────────────────────────────────

wt() {
    local target="$1"

    if [ -z "$target" ]; then
        bash "$WT_SCRIPT" list
        echo ""
        echo "用法: wt <branch-name>"
        return
    fi

    if [ "$target" = "main" ]; then
        cd "$REPO_ROOT" || return 1
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
        echo "  bash $WT_SCRIPT create $target && wt $target"
        return 1
    fi
}

# Tab 补全
_wt_complete() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    COMPREPLY=($(compgen -W "$(ls "$REPO_ROOT/.worktree/" 2>/dev/null)" -- "$cur"))
}
complete -F _wt_complete wt 2>/dev/null || true

# ── PROMPT 提示 ──────────────────────────────

WT_PROMPT_COLOR="${WT_PROMPT_COLOR:-36}"
WT_PROMPT_STYLE="${WT_PROMPT_STYLE:-short}"

__wt_prompt() {
    # 仅在 infra.rs 仓库内生效
    local toplevel
    toplevel=$(git rev-parse --show-toplevel 2>/dev/null) || return
    [[ "$toplevel" == "$REPO_ROOT" ]] || return

    local wt_info=""
    local current_pwd="$PWD"

    # 判断是否在 worktree 中
    if [[ "$current_pwd" == "$REPO_ROOT/.worktree/"* ]]; then
        local rel="${current_pwd#$REPO_ROOT/.worktree/}"
        local branch_name="${rel%%/*}"
        case "$WT_PROMPT_STYLE" in
            icon)  wt_info="🔀$branch_name" ;;
            short) wt_info="wt:$branch_name" ;;
            full)  wt_info="worktree:$branch_name" ;;
        esac
    else
        # 在 main 工作区
        case "$WT_PROMPT_STYLE" in
            icon)  wt_info="🏠main" ;;
            short) wt_info="main" ;;
            full)  wt_info="main" ;;
        esac
    fi

    # 注入到 PS1（在已有 PS1 左侧插入彩色标记）
    if [[ "$PS1" != *"__wt_marker__"* ]]; then
        local marker="\[\033[${WT_PROMPT_COLOR}m\][${wt_info}]\[\033[0m\] "
        PS1="${marker}${PS1}"
    fi
}

# 启用 PROMPT 注入（默认开启）
if [[ "${WT_PROMPT:-1}" != "0" ]]; then
    PROMPT_COMMAND="__wt_prompt; ${PROMPT_COMMAND:-:}"
fi
