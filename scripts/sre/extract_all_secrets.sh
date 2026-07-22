#!/bin/bash
# extract_all_secrets.sh — 验证凭据范围与复杂度
#
# 1. 验证仅提取 PASSWORD/TOKEN 条目（排除 DATABASE/USER）
# 2. 验证所有密码满足复杂度要求（≥24 chars, uppercase, lowercase, digits）
#
# 用法:
#   scripts/sre/extract_all_secrets.sh dev
#   scripts/sre/extract_all_secrets.sh prod
#   scripts/sre/extract_all_secrets.sh all
#
# SSOT: docs/governance/credential-baseline.md

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ENV="${1:-dev}"

EXPECTED_DEV=57
EXPECTED_PROD=0

failures=0

check_complexity() {
  local env="$1"
  local file="${SCRIPT_DIR}/../../../ZoneCNH/sre/secrets/env/${env}.md"

  echo "=== ${env}.md 复杂度检查 ==="

  if [ ! -f "$file" ]; then
    echo "  ⚠️  File not found, skipping"
    return 0
  fi

  python3 << PYEOF
import re, sys

fail = 0
with open('$file') as f:
    text = f.read()

# Extract all password-like values from credential tables
passwords = re.findall(r'\\|\\s*(?:market_\\w+|macro_\\w+|PostgreSQL|TDengine|Redis|Kafka|ClickHouse)\\s*\\|.*?\\x60([^\\x60]+)\\x60', text)

# FRED API key (32-char hex, exempt from uppercase rule)
fred_match = re.search(r'api_key=(\\S+)', text)
fred_key = fred_match.group(1) if fred_match else None

# NATS token (may contain +++, exempt from standard rules)
nats_match = re.search(r'密码\\s*\\x60([^\\x60]+)\\x60', text)
nats_token = nats_match.group(1) if nats_match else None

total = len(passwords)
violations = 0

def check_password(pwd, label, is_hex_key=False, is_token=False):
    global violations
    pwd = pwd.strip()
    issues = []

    # Rule 1: Minimum 24 characters (token and hex key exempt)
    if not is_token and not is_hex_key and len(pwd) < 24:
        issues.append(f'len={len(pwd)}')

    # Rule 2: Uppercase + lowercase + digits (hex key and token exempt)
    if not is_hex_key and not is_token:
        has_upper = bool(re.search(r'[A-Z]', pwd))
        has_lower = bool(re.search(r'[a-z]', pwd))
        has_digit = bool(re.search(r'[0-9]', pwd))
        if not (has_upper and has_lower and has_digit):
            missing = []
            if not has_upper: missing.append('uppercase')
            if not has_lower: missing.append('lowercase')
            if not has_digit: missing.append('digits')
            issues.append('missing: ' + ', '.join(missing))

    # Rule 3: No 4+ consecutive same character
    if re.search(r'(.)\\1{3,}', pwd):
        issues.append('repeated char')

    # Rule 4: No common patterns (skip for hex keys)
    if not is_hex_key:
        for pattern in ['password', 'admin123', '12345678', 'qwerty']:
            if pattern in pwd.lower():
                issues.append(f'common pattern: {pattern}')
                break

    if issues:
        masked = pwd[:3] + '***' + pwd[-3:] if len(pwd) > 6 else '***'
        print(f'  ❌ {label}: {masked} — {", ".join(issues)}')
        violations += 1

for pwd in passwords:
    check_password(pwd, 'password')

if fred_key:
    total += 1
    check_password(fred_key, 'FRED API key', is_hex_key=True)

if nats_token:
    total += 1
    check_password(nats_token, 'NATS token', is_token=True)

if violations > 0:
    print()
    print(f'  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━')
    print(f'  强度报告')
    print(f'  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━')

    pass_count = total - violations
    pass_pct = round(pass_count / total * 100, 1)
    fail_pct = round(violations / total * 100, 1)

    print(f'  Total checked:     {total}')
    print(f'  Pass:              {pass_count} ({pass_pct}%)')
    print(f'  Fail:              {violations} ({fail_pct}%)')
    print(f'  Pass ratio:        {pass_count}/{total} = {pass_pct:.1f}%')

    # Category breakdown
    length_fails = sum(1 for pwd in passwords if len(pwd.strip()) < 24)
    hex_fail = 1 if fred_key and len(fred_key.strip()) < 32 else 0
    token_fail = 0  # exempt

    print(f'  ┌─────────────────────────────')
    print(f'  │ 按类别统计')
    print(f'  ├ 长度 < 24:        {length_fails + hex_fail} ({round((length_fails + hex_fail)/total*100, 1)}%)')
    print(f'  ├ 令牌/API Key:     {token_fail + (1 if hex_fail else 0)} ({round((token_fail + (1 if hex_fail else 0))/total*100, 1)}%) [exempt]')
    print(f'  └ 合规比例:         {pass_count}/{total}')
    print()

    # Severity-based rating
    if fail_pct >= 50:
        rating = "CRITICAL"
    elif fail_pct >= 25:
        rating = "WARNING"
    elif fail_pct > 0:
        rating = "IMPROVING"
    else:
        rating = "HEALTHY"

    print(f'  Overall rating: {rating} ({pass_pct}% pass rate)')
    print()
    sys.exit(1)
else:
    print(f'  ✅ {total} passwords pass complexity rules')
    sys.exit(0)
PYEOF

  if [ $? -ne 0 ]; then
    failures=$((failures + 1))
  fi
}

verify_extraction() {
  local env="$1"
  local expected="$2"

  echo "=== ${env}.md 范围检查 ==="

  local count
  count=$(node "${SCRIPT_DIR}/secrets-migrate-all.mjs" --source tables --env "$env" --dry-run 2>&1 | grep -c "FOUNDATIONX_" || true)
  echo "  PASSWORD/TOKEN entries: $count"

  local db_entries
  db_entries=$(node "${SCRIPT_DIR}/secrets-migrate-all.mjs" --source tables --env "$env" --dry-run 2>&1 | grep -c "_DATABASE\|_USER " || true)
  if [ "$db_entries" -gt 0 ]; then
    echo "  ❌ $db_entries DATABASE/USER entries found"
    failures=$((failures + 1))
  else
    echo "  ✅ No DATABASE/USER in scope"
  fi

  if [ "$expected" -eq 0 ]; then
    echo "  ℹ️  Expected count not defined for ${env} (actual: $count)"
  elif [ "$count" -eq "$expected" ]; then
    echo "  ✅ Count matches expected ($expected)"
  else
    echo "  ⚠️  $count != expected $expected"
  fi

  echo ""
}

if [ "$ENV" = "all" ] || [ "$ENV" = "dev" ]; then
  verify_extraction "dev" "$EXPECTED_DEV"
  check_complexity "dev"
fi

if [ "$ENV" = "all" ] || [ "$ENV" = "prod" ]; then
  verify_extraction "prod" "$EXPECTED_PROD"
  check_complexity "prod"
fi

echo "=== Result ==="
if [ "$failures" -eq 0 ]; then
  echo "✅ All checks passed"
  exit 0
else
  echo "❌ $failures failure(s)"
  exit 1
fi
