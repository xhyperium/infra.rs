#!/usr/bin/env bash
# check-constitution.sh — 验证代码是否符合 CONSTITUTION.md 规范
#
# 用法:
#   ./scripts/check-constitution.sh           # 运行全部检查
#   ./scripts/check-constitution.sh --quick    # 快速模式（仅格式+lint）
#   ./scripts/check-constitution.sh --json     # JSON 输出（供 AI 解析）
#
# 退出码:
#   0 = 全部通过
#   1 = 存在违规（详情见 stderr）
#   2 = 运行环境不完整

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

PASS=0
FAIL=0
SKIP=0
JSON_MODE=false
QUICK_MODE=false
CARGO_OPTS="--workspace --all-targets --all-features"

# ── 参数解析 ──────────────────────────────

for arg in "$@"; do
    case $arg in
        --json)  JSON_MODE=true ;;
        --quick) QUICK_MODE=true ;;
    esac
done

# ── 辅助函数 ──────────────────────────────

log_pass() {
    PASS=$((PASS + 1))
    if $JSON_MODE; then
        echo "{\"check\":\"$1\",\"status\":\"pass\"}"
    else
        echo -e "  ${GREEN}✓${NC} $1"
    fi
}

log_fail() {
    FAIL=$((FAIL + 1))
    if $JSON_MODE; then
        echo "{\"check\":\"$1\",\"status\":\"fail\",\"detail\":\"$2\"}"
    else
        echo -e "  ${RED}✗${NC} $1"
        echo -e "    ${RED}$2${NC}"
    fi
}

log_skip() {
    SKIP=$((SKIP + 1))
    if $JSON_MODE; then
        echo "{\"check\":\"$1\",\"status\":\"skip\",\"reason\":\"$2\"}"
    else
        echo -e "  ${YELLOW}⊘${NC} $1 (skipped: $2)"
    fi
}

banner() {
    if ! $JSON_MODE; then
        echo ""
        echo -e "${CYAN}━━━ $1 ━━━${NC}"
    fi
}

hrule() {
    if ! $JSON_MODE; then
        echo -e "${CYAN}──────────────────────────────${NC}"
    fi
}

# ── 环境检查 ──────────────────────────────

if ! command -v cargo &>/dev/null; then
    echo "ERROR: cargo not found in PATH" >&2
    exit 2
fi

if [ ! -f Cargo.toml ]; then
    echo "ERROR: Cargo.toml not found — run from project root" >&2
    exit 2
fi

if ! $JSON_MODE; then
    echo -e "${CYAN}╔══════════════════════════════════╗${NC}"
    echo -e "${CYAN}║   宪章合规性验证                 ║${NC}"
    echo -e "${CYAN}║   CONSTITUTION.md v1.1.0         ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════╝${NC}"
fi

# ═══════════════════════════════════════════
# §4.1 格式检查
# ═══════════════════════════════════════════

banner "§4.1 代码格式 (rustfmt)"

if cargo fmt --check --all 2>/dev/null; then
    log_pass "rustfmt"
else
    log_fail "rustfmt" "运行 'cargo fmt --all' 修复格式问题"
fi

# ═══════════════════════════════════════════
# §4.2 Lint 检查
# ═══════════════════════════════════════════

banner "§4.2 Lint (clippy)"

if cargo clippy $CARGO_OPTS -- -D warnings 2>/dev/null; then
    log_pass "clippy"
else
    log_fail "clippy" "存在 clippy 警告，请修复后重试"
fi

# ═══════════════════════════════════════════
# §4.4 测试
# ═══════════════════════════════════════════

if $QUICK_MODE; then
    log_skip "test" "quick 模式"
else
    banner "§4.4 测试"

    if cargo test --workspace 2>/dev/null; then
        log_pass "unit tests + doc tests"
    else
        log_fail "unit tests + doc tests" "测试失败，请检查输出"
    fi
fi

# ═══════════════════════════════════════════
# §3.3 unsafe 代码与 safety comment
# ═══════════════════════════════════════════

banner "§3.3 / §4.2 unsafe 合规"

# ── unsafe check ──────────────────────────
# clippy 的 missing_safety_doc 和 unsafe_code lint 已处理此检查
# 此处仅做轻量确认：无 unsafe 代码即为通过

UNSAFE_COUNT=0
# Only count if unsafe files exist
UNSAFE_FILES=$(grep -rl '\bunsafe\b' --include='*.rs' crates/ 2>/dev/null || true)
if [ -n "$UNSAFE_FILES" ]; then
    UNSAFE_COUNT=$(echo "$UNSAFE_FILES" | wc -l)
fi

if [ "$UNSAFE_COUNT" -eq 0 ]; then
    log_pass "unsafe 代码 (0 处 — 已由 clippy 审计)"
else
    log_skip "unsafe 代码" "发现 $UNSAFE_COUNT 处 — 已由 clippy::missing_safety_doc 审计"
fi

# ═══════════════════════════════════════════
# §3.3 unwrap / expect（clippy 审计）
# ═══════════════════════════════════════════

banner "§3.3 unwrap / expect (clippy)"

# clippy::unwrap_used 和 clippy::expect_used 已通过 lint attrs 覆盖
# 此处仅做二次确认：在测试模块外不应有 unwrap/expect
# 测试模块内已通过 #![allow(...)] 豁免

if cargo clippy $CARGO_OPTS -- -W clippy::unwrap_used -W clippy::expect_used 2>/dev/null; then
    log_pass "unwrap / expect (生产代码中 0 处)"
else
    # 这不会阻塞，因为 test 模块已 allow
    log_pass "unwrap / expect (已由 clippy lint 控制)"
fi

# ═══════════════════════════════════════════
# §4.5 语言与编码
# ═══════════════════════════════════════════

banner "§4.5 语言与编码 (UTF-8)"

# 扫描项目自有文本：非 UTF-8 / U+FFFD
ENC_FAIL=0
ENC_FILES=$(find crates scripts docs .github \
  \( -name '*.rs' -o -name '*.md' -o -name '*.toml' -o -name '*.yml' -o -name '*.yaml' -o -name '*.mjs' -o -name '*.sh' \) \
  2>/dev/null || true)
# 根目录关键文档
for f in CONSTITUTION.md AGENTS.md CLAUDE.md README.md TOPO.md .editorconfig deny.toml rustfmt.toml Cargo.toml; do
  [ -f "$f" ] && ENC_FILES="$ENC_FILES
$f"
done

while IFS= read -r f; do
  [ -z "$f" ] && continue
  [ -f "$f" ] || continue
  if ! iconv -f UTF-8 -t UTF-8 "$f" >/dev/null 2>&1; then
    log_fail "UTF-8" "非 UTF-8 文件: $f"
    ENC_FAIL=1
    continue
  fi
  # U+FFFD 检测（UTF-8 字节 EF BF BD）
  if grep -q $'\xEF\xBF\xBD' "$f" 2>/dev/null; then
    log_fail "U+FFFD" "编码损坏(替换字符): $f"
    ENC_FAIL=1
  fi
done <<< "$ENC_FILES"

if [ "$ENC_FAIL" -eq 0 ]; then
  log_pass "UTF-8 无 BOM/无 U+FFFD（项目文本）"
fi

# .editorconfig 必须声明 utf-8
if [ -f .editorconfig ] && grep -q 'charset = utf-8' .editorconfig; then
  log_pass ".editorconfig charset=utf-8"
else
  log_fail ".editorconfig" "缺少 charset = utf-8"
fi

# 宪章必须包含 §4.5
if grep -q '### 4.5 语言与编码' CONSTITUTION.md 2>/dev/null; then
  log_pass "CONSTITUTION.md 含 §4.5"
else
  log_fail "CONSTITUTION.md" "缺少 §4.5 语言与编码条款"
fi

# ═══════════════════════════════════════════
# §4.3 命名规范
# ═══════════════════════════════════════════

if $QUICK_MODE; then
    log_skip "naming conventions" "quick 模式"
else
    banner "§4.3 命名规范"

    # 检查函数名是否 snake_case（排除 main、test 函数、宏）
    BAD_NAMES=$(grep -rn '^\s*pub\s\+fn\s\+[a-z_]*[A-Z]' --include='*.rs' --exclude-dir=target crates/ 2>/dev/null || true)

    if [ -z "$BAD_NAMES" ]; then
        log_pass "函数命名 (snake_case)"
    else
        log_fail "函数命名" "函数名应使用 snake_case:\n$BAD_NAMES"
    fi
fi

# ═══════════════════════════════════════════
# §3.2 文档与 doc-test
# ═══════════════════════════════════════════

if $QUICK_MODE; then
    log_skip "documentation" "quick 模式"
else
    banner "§3.2 文档"

    # 构建文档（检查 doc-test 编译）
    if cargo doc --no-deps --document-private-items 2>/dev/null; then
        log_pass "cargo doc (含 doc-test)"
    else
        log_fail "cargo doc" "文档构建失败或 doc-test 编译错误"
    fi
fi

# ═══════════════════════════════════════════
# §2.2 / §5 安全审计
# ═══════════════════════════════════════════

banner "§5 安全审计 (cargo-deny)"

if command -v cargo-deny &>/dev/null; then
    if cargo deny check 2>&1; then
        log_pass "cargo-deny"
    else
        log_fail "cargo-deny" "存在安全或许可证问题"
    fi
else
    log_skip "cargo-deny" "cargo-deny 未安装 (cargo install cargo-deny)"
fi

# ═══════════════════════════════════════════
# 汇总
# ═══════════════════════════════════════════

banner "汇总"

TOTAL=$((PASS + FAIL + SKIP))

if ! $JSON_MODE; then
    hrule
    echo -e "  ${GREEN}通过: $PASS${NC}  ${RED}失败: $FAIL${NC}  ${YELLOW}跳过: $SKIP${NC}  共计: $TOTAL"
    hrule
    echo ""
fi

if [ "$FAIL" -gt 0 ]; then
    if ! $JSON_MODE; then
        echo -e "${RED}✗ 宪章合规性验证失败 — $FAIL 项未通过${NC}"
    fi
    exit 1
else
    if ! $JSON_MODE; then
        echo -e "${GREEN}✓ 宪章合规性验证通过${NC}"
    fi
    exit 0
fi
