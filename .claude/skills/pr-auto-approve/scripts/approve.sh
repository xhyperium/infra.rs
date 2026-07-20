#!/usr/bin/env bash
# pr-auto-approve: 以 @liukongqiang5 对 PR 提交 APPROVE（非作者路径）。
#
# 环境变量（必须）:
#   LIUKONGQIANG5_APPROVE_TOKEN  — liukongqiang5 的 GitHub token（repo scope）
#
# 可选:
#   PR_AUTO_APPROVE_EXPECTED_LOGIN  默认 liukongqiang5
#   PR_AUTO_APPROVE_REPO           默认 xhyperium/xhyper.rs
#   PR_AUTO_APPROVE_API            默认 https://api.github.com
#
# 用法:
#   bash .agent/skills/pr-auto-approve/scripts/approve.sh <pr-number> [body]
#   bash .agent/skills/pr-auto-approve/scripts/approve.sh 799
#   bash .agent/skills/pr-auto-approve/scripts/approve.sh 799 "CI green recovery"
#
# 退出码:
#   0  已 APPROVED 或已存在有效 APPROVE（幂等）
#   1  用法/参数错误
#   2  token 缺失或身份不是期望账号
#   3  GitHub API 失败
#   4  禁止操作（如 token 用户即 PR 作者且 ruleset 要求他人批准）
set -euo pipefail

API="${PR_AUTO_APPROVE_API:-https://api.github.com}"
REPO="${PR_AUTO_APPROVE_REPO:-xhyperium/xhyper.rs}"
EXPECTED_LOGIN="${PR_AUTO_APPROVE_EXPECTED_LOGIN:-liukongqiang5}"
API_VERSION="2022-11-28"

usage() {
  echo "用法: $0 <pr-number> [review-body]" >&2
  echo "  需要环境变量 LIUKONGQIANG5_APPROVE_TOKEN（身份必须是 ${EXPECTED_LOGIN}）" >&2
  exit 1
}

PR="${1:-}"
[[ "$PR" =~ ^[0-9]+$ ]] || usage
BODY="${2:-Auto-approve via @${EXPECTED_LOGIN} (LIUKONGQIANG5_APPROVE_TOKEN). CI/recovery path; not author self-approve.}"

if [[ -z "${LIUKONGQIANG5_APPROVE_TOKEN:-}" ]]; then
  echo "FAIL: 未设置 LIUKONGQIANG5_APPROVE_TOKEN" >&2
  exit 2
fi
TOKEN="$LIUKONGQIANG5_APPROVE_TOKEN"

api() {
  local method="$1"
  local path="$2"
  shift 2
  curl -sS -X "$method" \
    -H "Authorization: Bearer ${TOKEN}" \
    -H "Accept: application/vnd.github+json" \
    -H "X-GitHub-Api-Version: ${API_VERSION}" \
    "${API}${path}" \
    "$@"
}

json_field() {
  python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get(sys.argv[1],"") if not isinstance(d.get(sys.argv[1]), dict) else d.get(sys.argv[1],{}).get(sys.argv[2],""))' "$@"
}

# 1) 校验 token 身份
USER_JSON="$(api GET /user)"
LOGIN="$(printf '%s' "$USER_JSON" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("login",""))')"
if [[ "$LOGIN" != "$EXPECTED_LOGIN" ]]; then
  echo "FAIL: token 身份是 '${LOGIN}'，期望 '@${EXPECTED_LOGIN}'" >&2
  exit 2
fi
echo "OK: token 身份 @${LOGIN}"

# 2) 读取 PR
PR_JSON="$(api GET "/repos/${REPO}/pulls/${PR}")"
PR_STATE="$(printf '%s' "$PR_JSON" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("state",""))')"
AUTHOR="$(printf '%s' "$PR_JSON" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("user",{}).get("login",""))')"
HEAD_SHA="$(printf '%s' "$PR_JSON" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("head",{}).get("sha",""))')"
HTML_URL="$(printf '%s' "$PR_JSON" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("html_url",""))')"

if [[ "$PR_STATE" != "open" ]]; then
  echo "FAIL: PR #${PR} state=${PR_STATE}（仅 open 可 approve） url=${HTML_URL}" >&2
  exit 4
fi

if [[ "$AUTHOR" == "$LOGIN" ]]; then
  echo "FAIL: @${LOGIN} 是 PR #${PR} 作者，无法（也不应）self-approve" >&2
  exit 4
fi
echo "OK: PR #${PR} author=@${AUTHOR} head=${HEAD_SHA:0:9}"
echo "    ${HTML_URL}"

# 3) 幂等：若已有该用户对当前 head 的 APPROVED，直接成功
REVIEWS="$(api GET "/repos/${REPO}/pulls/${PR}/reviews?per_page=100")"
ALREADY="$(printf '%s' "$REVIEWS" | python3 -c '
import json,sys
reviews=json.load(sys.stdin)
login=sys.argv[1]; head=sys.argv[2]
# 取该用户最后一条 review
mine=[r for r in reviews if (r.get("user") or {}).get("login")==login]
if not mine:
  print("no"); raise SystemExit
last=mine[-1]
# COMMIT 级：state APPROVED 且 commit_id 匹配 head（或 state 仍有效）
st=(last.get("state") or "").upper()
cid=(last.get("commit_id") or "")
if st=="APPROVED" and (not cid or cid==head or head.startswith(cid) or cid.startswith(head[:7])):
  print("yes")
elif st=="APPROVED":
  # 仍算有 approve；require_last_push_approval 可能要求重批
  print("stale" if cid and cid!=head else "yes")
else:
  print("no")
' "$LOGIN" "$HEAD_SHA")"

if [[ "$ALREADY" == "yes" ]]; then
  echo "OK: @${LOGIN} 已对 PR #${PR} 有 APPROVED（幂等跳过）"
  exit 0
fi

# 4) 提交 APPROVE
PAYLOAD="$(python3 -c 'import json,sys; print(json.dumps({"event":"APPROVE","body":sys.argv[1]}))' "$BODY")"
RESP="$(api POST "/repos/${REPO}/pulls/${PR}/reviews" -d "$PAYLOAD")"
# 错误形态：{"message":"..."}
ERR="$(printf '%s' "$RESP" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("message",""))')"
STATE="$(printf '%s' "$RESP" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("state",""))')"
RID="$(printf '%s' "$RESP" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("id",""))')"

if [[ -n "$ERR" && -z "$STATE" ]]; then
  echo "FAIL: GitHub API: ${ERR}" >&2
  printf '%s\n' "$RESP" >&2
  exit 3
fi

if [[ "${STATE^^}" != "APPROVED" ]]; then
  echo "FAIL: 期望 state=APPROVED，得到 state=${STATE}" >&2
  printf '%s\n' "$RESP" >&2
  exit 3
fi

echo "OK: APPROVED by @${LOGIN} review_id=${RID} pr=#${PR}"
echo "    commit=${HEAD_SHA}"
exit 0
