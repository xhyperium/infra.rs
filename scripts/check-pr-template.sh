#!/usr/bin/env bash
# check-pr-template.sh — 检查 PR 描述是否符合模板格式
#
# 由 CI 在 PR 事件中调用，读取 PR body 并验证必需字段已填写。
# 环境变量: PR_BODY (PR 描述文本)

set -euo pipefail

BODY="${PR_BODY:-}"

if [ -z "$BODY" ]; then
    echo "::error::未检测到 PR 描述"
    exit 1
fi

PASS=true

# ── 1. 检查「类型」选择 ────────────────────

if echo "$BODY" | grep -qE '^\s*-\s*\[\s*x\s*\]'; then
    echo "::notice::变更类型: 已勾选"
else
    echo "::error::变更类型: 请勾选至少一项 (- [x])"
    PASS=false
fi

# ── 2. 检查「关联 Issue」 ──────────────────

CLOSES=$(echo "$BODY" | sed -n '/^## 关联 Issue/,/^## /p' | grep -oP 'Closes #\d+' || true)
if [ -n "$CLOSES" ]; then
    echo "::notice::关联 Issue: $CLOSES"
else
    echo "::warning::关联 Issue: 未找到 'Closes #<num>' 链接，建议关联 Issue"
fi

# ── 3. 检查「变更摘要」不为空 ──────────────

SUMMARY=$(echo "$BODY" | sed -n '/^## 变更摘要/,/^## /p' | sed '1d;$d' | grep -v '^<!--' | grep -v '^$' | grep -v '^##' || true)

# 去除空行和注释行后检查是否有内容
SUMMARY_TRIMMED=$(echo "$SUMMARY" | tr -d '[:space:]')
if [ -z "$SUMMARY_TRIMMED" ]; then
    echo "::error::变更摘要: 请填写变更说明"
    PASS=false
else
    echo "::notice::变更摘要: 已填写"
fi

# ── 4. 检查「宪章合规性」勾选 ──────────────

CONSTITUTION=$(echo "$BODY" | sed -n '/^## 宪章合规性/,/^## /p' || true)
UNCHECKED=$(echo "$CONSTITUTION" | grep -E '^\s*-\s*\[\s*\]' | grep -v '<!--' || true)

if [ -n "$UNCHECKED" ]; then
    echo "::warning::宪章合规性: 以下项目未勾选，请确认或说明原因:"
    echo "$UNCHECKED" | while read -r line; do
        echo "::warning::  $line"
    done
else
    CHECKED_COUNT=$(echo "$CONSTITUTION" | grep -cE '^\s*-\s*\[\s*x\s*\]' || echo 0)
    echo "::notice::宪章合规性: $CHECKED_COUNT 项已勾选"
fi

# ── 5. 检查「验证方式」有实际内容 ───────────────

VERIFY_SECTION=$(echo "$BODY" | sed -n '/^## 验证方式/,/^## /p' || true)
# 检查代码块中是否有实际内容（非注释行）
VERIFY_CONTENT=$(echo "$VERIFY_SECTION" | sed -n '/```bash/,/```/p' | grep -v '```' | grep -v '^#' | grep -v '^$' | grep -v '^<!--' || true)

if [ -z "$VERIFY_CONTENT" ]; then
    echo "::warning::验证方式: 请在代码块中贴入验证命令和输出"
else
    echo "::notice::验证方式: 已填写"
fi

# ── 6. 检查「审查聚焦」有内容 ──────────────

FOCUS=$(echo "$BODY" | sed -n '/^## 审查聚焦/,/^$/p' | sed '1d' | grep -v '^<!--' | grep -v '^$' || true)
FOCUS_TRIMMED=$(echo "$FOCUS" | tr -d '[:space:]')
if [ -z "$FOCUS_TRIMMED" ]; then
    echo "::warning::审查聚焦: 请指出需要 reviewer 关注的部分"
else
    echo "::notice::审查聚焦: 已填写"
fi

# ── 结果 ────────────────────────────────────

if $PASS; then
    echo "::notice::PR 模板校验通过"
    exit 0
else
    echo "::error::PR 模板校验失败 — 请完善 PR 描述后重新推送"
    exit 1
fi
