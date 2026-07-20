#!/usr/bin/env bash
# Production money-path hygiene for decimalx consumers (P0).
# Prefer rg when available; fall back to grep -R so CI runners without ripgrep still FAIL-closed (no SKIP).
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../../../../../.." && pwd)"
cd "$ROOT"
echo "== decimalx prod money-path check =="
fail=0

# search helper: first arg is regex; remaining are paths
# excludes common test/bench paths via grep -v when using grep fallback
search_prod() {
  local pattern="$1"
  shift
  local paths=("$@")
  if command -v rg >/dev/null 2>&1 && rg --version >/dev/null 2>&1; then
    rg -n --type rust "$pattern" "${paths[@]}" \
      -g '!**/*test*' -g '!**/tests/**' -g '!**/benches/**' 2>/dev/null || true
    return 0
  fi
  # grep fallback (portable enough for GHA self-hosted)
  local p out=""
  for p in "${paths[@]}"; do
    if [ -d "$p" ] || [ -f "$p" ]; then
      out+=$(grep -RInE --include='*.rs' "$pattern" "$p" 2>/dev/null \
        | grep -vE '/tests/|_test\.rs|/benches/|/#\[cfg\(test\)\]' \
        || true)
      out+=$'\n'
    fi
  done
  # filter empty lines
  printf '%s' "$out" | grep -vE '^$' || true
}

has_match() {
  local out
  out=$(search_prod "$@")
  [ -n "$(printf '%s' "$out" | tr -d '[:space:]')" ]
}

# 1) Forbidden: panicking rescale in production trees
if matches=$(search_prod '\.rescale\s*\(' crates/domain crates/adapters/exchange crates/adapters/storage crates/types/canonical); then
  if [ -n "$(printf '%s' "$matches" | tr -d '[:space:]')" ]; then
    echo "$matches"
    echo "FAIL: production .rescale(" >&2
    fail=1
  else
    echo "OK: no production .rescale("
  fi
fi

# 2) Forbidden: float coercion in focused money paths
if matches=$(search_prod 'as f64|as f32' \
  crates/domain/ledger crates/domain/exchange \
  crates/adapters/exchange/binance/src/parser.rs \
  crates/adapters/exchange/okx/src/parser.rs); then
  if [ -n "$(printf '%s' "$matches" | tr -d '[:space:]')" ]; then
    echo "$matches"
    echo "FAIL: float coercion" >&2
    fail=1
  else
    echo "OK: no float coercion"
  fi
fi

# 3) API surface
if grep -q 'pub const MAX_SCALE' crates/types/decimal/src/lib.rs \
  && grep -q 'pub fn try_new' crates/types/decimal/src/lib.rs; then
  echo "OK: MAX_SCALE/try_new"
else
  echo "FAIL: API missing" >&2
  fail=1
fi

# 4) ledger fail-closed
if awk '/fn balance_checked/,/^    pub fn |^    /// 分录|^}$/' crates/domain/ledger/src/ledger.rs 2>/dev/null \
  | grep -q 'checked_add'; then
  echo "OK: balance_checked checked_add"
elif grep -A40 'fn balance_checked' crates/domain/ledger/src/ledger.rs | grep -q 'checked_add'; then
  echo "OK: balance_checked checked_add"
else
  echo "FAIL: balance_checked" >&2
  fail=1
fi
if grep -q 'AmountOverflow' crates/domain/ledger/src/error.rs; then
  echo "OK: AmountOverflow"
else
  echo "FAIL: AmountOverflow" >&2
  fail=1
fi

# 5) Forbid panicking sum accumulation in production money crates
if matches=$(search_prod 'sum\s*=\s*sum\s*\+|sum\s*=\s*sum\s*-' \
  crates/domain/ledger/src \
  crates/domain/exchange/src \
  crates/adapters/exchange/binance/src \
  crates/adapters/exchange/okx/src \
  crates/adapters/storage/taos/src); then
  if [ -n "$(printf '%s' "$matches" | tr -d '[:space:]')" ]; then
    echo "$matches"
    echo "FAIL: panicking sum accumulation (use checked_add/sub)" >&2
    fail=1
  else
    echo "OK: no panicking sum accumulation"
  fi
fi

# 6) Production parse entrypoints must use FromStr
if grep -A8 'fn parse_decimal' crates/adapters/exchange/binance/src/parser.rs | grep -q '\.parse('; then
  echo "OK: binance parse_decimal uses FromStr"
else
  echo "FAIL: binance parse_decimal not using FromStr" >&2
  fail=1
fi
if grep -A8 'fn parse_decimal' crates/adapters/exchange/okx/src/parser.rs | grep -q '\.parse('; then
  echo "OK: okx parse_decimal uses FromStr"
else
  echo "FAIL: okx parse_decimal not using FromStr" >&2
  fail=1
fi

if [[ $fail -eq 0 ]]; then
  echo "RESULT: PASS"
  exit 0
fi
echo "RESULT: FAIL" >&2
exit 1
