#!/usr/bin/env bash
# testkit 交叉引用完整性验证脚本
# 验证 testkit SSOT 域内所有文件引用一致，无破链、无漂移
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BASE="$ROOT/.agents/ssot/testkit"
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

PASS=0
FAIL=0
WARN=0

pass() { echo -e "${GREEN}PASS${NC}  $*"; PASS=$((PASS + 1)); }
fail() { echo -e "${RED}FAIL${NC}  $*"; FAIL=$((FAIL + 1)); }
warn() { echo -e "${YELLOW}WARN${NC}  $*"; WARN=$((WARN + 1)); }

echo "=== testkit 交叉引用完整性验证 ==="
echo ""

# -----------------------------------------------------------
# §1 文件存在性
# -----------------------------------------------------------
echo "--- §1 关键文件存在性 ---"

check_exists() {
    local path="$BASE/$1"
    if [[ -f "$path" ]]; then
        pass "exists: $1"
    else
        fail "missing: $1"
    fi
}

check_exists "testkit-spec.md"
check_exists "spec/spec.md"
check_exists "spec/xhyper-testkit-complete-spec.md"
check_exists "spec/TESTKIT-SPEC-001.superseded.md"
check_exists "README.md"
check_exists "plan/plan.md"
check_exists "plan/gap-matrix.md"

# 旧文件必须不存在
if [[ ! -f "$BASE/xhyper-testkit-complete-spec.md" ]]; then
    pass "absent: xhyper-testkit-complete-spec.md (renamed to testkit-spec.md)"
else
    fail "stale: xhyper-testkit-complete-spec.md still exists at root"
fi

echo ""

# -----------------------------------------------------------
# §2 双镜像字节一致
# -----------------------------------------------------------
echo "--- §2 双镜像字节一致 ---"

if cmp -s "$BASE/spec/spec.md" "$BASE/spec/xhyper-testkit-complete-spec.md"; then
    pass "spec/spec.md ≡ spec/xhyper-testkit-complete-spec.md (byte-level)"
else
    fail "spec/spec.md ≠ spec/xhyper-testkit-complete-spec.md — DRIFT"
fi

echo ""

# -----------------------------------------------------------
# §3 redirect stub 指向正确
# -----------------------------------------------------------
echo "--- §3 重定向桩指向 ---"

# testkit-spec.md must reference spec/spec.md
if grep -q 'spec/spec\.md' "$BASE/testkit-spec.md"; then
    pass "testkit-spec.md → spec/spec.md"
else
    fail "testkit-spec.md missing ref to spec/spec.md"
fi

if grep -q 'spec/xhyper-testkit-complete-spec\.md' "$BASE/testkit-spec.md"; then
    pass "testkit-spec.md → spec/xhyper-testkit-complete-spec.md"
else
    fail "testkit-spec.md missing ref to spec/xhyper-testkit-complete-spec.md"
fi

echo ""

# -----------------------------------------------------------
# §4 README.md 交叉引用
# -----------------------------------------------------------
echo "--- §4 README.md 交叉引用 ---"

# L6: must link to spec/spec.md and spec/xhyper-testkit-complete-spec.md
if grep -q '\[spec/spec\.md\](spec/spec\.md)' "$BASE/README.md"; then
    pass "README: spec/spec.md link correct"
else
    fail "README: missing or wrong spec/spec.md link"
fi

if grep -q '\[spec/xhyper-testkit-complete-spec\.md\](spec/xhyper-testkit-complete-spec\.md)' "$BASE/README.md"; then
    pass "README: spec/xhyper-testkit-complete-spec.md link correct"
else
    fail "README: missing or wrong complete-spec link"
fi

# L55: cmp path must reference spec/ level
if grep -q 'testkit/spec/xhyper-testkit-complete-spec\.md' "$BASE/README.md"; then
    pass "README: cmp path references spec/xhyper-testkit-complete-spec.md"
else
    fail "README: cmp path does not reference spec/xhyper-testkit-complete-spec.md"
fi

echo ""

# -----------------------------------------------------------
# §5 plan/plan.md 交叉引用
# -----------------------------------------------------------
echo "--- §5 plan/plan.md 交叉引用 ---"

if grep -qE '\.\./spec/spec\.md\)' "$BASE/plan/plan.md"; then
    pass "plan/plan.md: Source Spec link → ../spec/spec.md"
else
    fail "plan/plan.md: missing or wrong Source Spec link"
fi

echo ""

# -----------------------------------------------------------
# §6 plan/gap-matrix.md 交叉引用
# -----------------------------------------------------------
echo "--- §6 plan/gap-matrix.md 交叉引用 ---"

if grep -q 'Source.*spec/spec\.md' "$BASE/plan/gap-matrix.md"; then
    pass "plan/gap-matrix.md: Source → spec/spec.md"
else
    fail "plan/gap-matrix.md: missing or wrong Source ref"
fi

echo ""

# -----------------------------------------------------------
# §7 superseded spec 交叉引用
# -----------------------------------------------------------
echo "--- §7 superseded spec 交叉引用 ---"

# TESTKIT-SPEC-001.superseded.md must link to ../spec/spec.md or ./xhyper-testkit-complete-spec.md
if grep -qE 'xhyper-testkit-complete-spec\.md.*spec.*testkit-complete|./xhyper-testkit-complete-spec\.md' "$BASE/spec/TESTKIT-SPEC-001.superseded.md"; then
    pass "TESTKIT-SPEC-001.superseded.md: link → xhyper-testkit-complete-spec.md"
else
    fail "TESTKIT-SPEC-001.superseded.md: broken or missing link"
fi

echo ""

# -----------------------------------------------------------
# §8 spec 文件内 cmp 自引用
# -----------------------------------------------------------
echo "--- §8 spec 文件 cmp 自引用 ---"

for spec in "$BASE/spec/spec.md" "$BASE/spec/xhyper-testkit-complete-spec.md"; do
    fname=$(basename "$spec")
    if grep -q 'testkit/spec/xhyper-testkit-complete-spec\.md' "$spec"; then
        pass "$fname: cmp path → spec/xhyper-testkit-complete-spec.md"
    else
        fail "$fname: cmp path does not reference spec/xhyper-testkit-complete-spec.md"
    fi
done

echo ""

# -----------------------------------------------------------
# §9 无 stale 引用残留（active 文件）
# -----------------------------------------------------------
echo "--- §9 stale 引用扫描 ---"

# 在 active 文件（非 plan/archive/）中搜索指向旧 root-level 文件名的引用
STALE_REF="testkit/xhyper-testkit-complete-spec\.md"

while IFS= read -r -d '' file; do
    # 跳过 plan/archive/（历史归档）
    if [[ "$file" == *"/plan/archive/"* ]]; then
        continue
    fi
    if grep -q "$STALE_REF" "$file"; then
        warn "$file: stale ref to root-level xhyper-testkit-complete-spec.md"
    fi
done < <(find "$BASE" -name "*.md" -print0)

# 确认无 stale warning（plan/archive/ 不计数）
echo "  (plan/archive/ 历史文件不在扫描范围)"
echo ""

# -----------------------------------------------------------
# §10 kernel 交叉引用
# -----------------------------------------------------------
echo "--- §10 kernel 交叉引用 ---"

KBASE="$ROOT/.agents/ssot/kernel"

if [[ -f "$KBASE/kernel-spec.md" ]]; then
    pass "kernel-spec.md exists"
else
    fail "kernel-spec.md missing"
fi

if grep -q 'spec/spec\.md' "$KBASE/kernel-spec.md"; then
    pass "kernel-spec.md → spec/spec.md"
else
    fail "kernel-spec.md missing ref to spec/spec.md"
fi

if grep -q 'spec/xhyper-kernel-complete-spec\.md' "$KBASE/kernel-spec.md"; then
    pass "kernel-spec.md → spec/xhyper-kernel-complete-spec.md"
else
    fail "kernel-spec.md missing ref to spec/xhyper-kernel-complete-spec.md"
fi

# 确认现有 kernel 引用未受影响
for check in "kernel/plan/plan.md" "kernel/design/DESIGN-KERNEL-002.md" "kernel/README.md"; do
    if [[ -f "$ROOT/.agents/ssot/$check" ]] && grep -q 'spec/spec\.md' "$ROOT/.agents/ssot/$check"; then
        pass "$check: references spec/spec.md (unchanged)"
    else
        warn "$check: check manually"
    fi
done

echo ""

# -----------------------------------------------------------
# §11 完整性检查：所有 spec 路径可 resolve
# -----------------------------------------------------------
echo "--- §11 链接可 resolve 性 ---"

# 从 testkit-spec.md 重定向桩追踪到最终目标
echo "  redirect chain: testkit-spec.md → spec/spec.md"
echo "                   testkit-spec.md → spec/xhyper-testkit-complete-spec.md"

if [[ -f "$BASE/spec/spec.md" ]] && [[ -f "$BASE/spec/xhyper-testkit-complete-spec.md" ]]; then
    pass "all redirect targets resolve"
else
    fail "redirect target missing"
fi

echo ""

# -----------------------------------------------------------
# 结果
# -----------------------------------------------------------
echo "========================================"
echo -e "PASS: ${GREEN}$PASS${NC}  FAIL: ${RED}$FAIL${NC}  WARN: ${YELLOW}$WARN${NC}"
echo "========================================"

if [[ $FAIL -gt 0 ]]; then
    echo -e "${RED}存在 $FAIL 项失败，请修复后重试${NC}"
    exit 1
else
    echo -e "${GREEN}所有交叉引用验证通过${NC}"
    exit 0
fi
