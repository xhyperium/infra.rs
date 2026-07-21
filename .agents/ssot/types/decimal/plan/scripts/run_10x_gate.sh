#!/usr/bin/env bash
# 10x gate for PLAN-TYPES-DECIMALX-002-agent-safe-v1
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../../../../.." && pwd)"
cd "$ROOT"
SCRATCH="${DECIMALX_SCRATCH:-/tmp/grok-goal-99a109d2452b/implementer}"
LOGDIR="${SCRATCH}/10x"
mkdir -p "$LOGDIR"
SUMMARY="${LOGDIR}/decimal-10x-summary.log"
: >"$SUMMARY"
fail_rounds=0
pass_rounds=0
TIP0="$(git rev-parse HEAD)"
echo "content_tip_start=$TIP0" | tee -a "$SUMMARY"

check_round() {
  local r="$1"
  local log="${LOGDIR}/round-$(printf '%02d' "$r").log"
  {
    echo "=== round $r ==="
    echo "HEAD=$(git rev-parse HEAD)"
    test -f .agents/ssot/types/decimal/decimalx-spec.md
    test -f .agents/ssot/types/decimal/20260717/xhyper-decimalx-complete-goal.md
    test -f .agents/ssot/types/decimal/20260717/xhyper-decimalx-complete-spec.md
    grep -q 'Draft' .agents/ssot/types/decimal/20260717/xhyper-decimalx-complete-spec.md
    grep -q 'SPEC-TYPES-DECIMALX-002' .agents/ssot/types/decimal/plan/plan.md
    grep -q 'GOAL-TYPES-DECIMALX-002' .agents/ssot/types/decimal/plan/plan.md
    grep -q 'decimalx-spec.md' .agents/ssot/types/decimal/plan/plan.md
    grep -q 'REJECTED' .agents/ssot/types/decimal/plan/plan.md
    grep -q 'crates/types/numeric' .agents/ssot/types/decimal/plan/residual-open.md
    test -f .agents/ssot/types/decimal/todo.md
    grep -q 'DONE' .agents/ssot/types/decimal/todo.md
    # no bare agent-safe OPEN lines claiming completion wrongly — HUMAN_ONLY ok
    ! grep -E '^\| T-.*\| AGENT_SAFE \| OPEN' .agents/ssot/types/decimal/todo.md
    test -f .agents/ssot/types/decimal/plan/evidence/m0-consumer-inventory-2026-07-17.txt
    grep -q 'decimalx' .agents/ssot/types/decimal/plan/evidence/m0-consumer-inventory-2026-07-17.txt
    grep -q '# Panics' crates/types/decimal/src/lib.rs
    cargo test -p xhyper-decimalx --quiet
    cargo check -p xhyper-decimalx --all-targets --quiet
    cargo clippy -p xhyper-decimalx --all-targets -- -D warnings
    cargo fmt -- --check
    test -f .agents/ssot/types/decimal/plan/alignment-decimalx-2026-07-17.md
    ! grep -Eiq 'package stable|wire stable|Goal Achieved|Status:.*Approved' \
      .agents/ssot/types/decimal/plan/alignment-decimalx-2026-07-17.md
    test "$(git rev-parse HEAD)" = "$TIP0"
    echo "ROUND $r PASS"
  } >"$log" 2>&1
}

for i in $(seq 1 10); do
  if check_round "$i"; then
    pass_rounds=$((pass_rounds + 1))
    echo "round=$i result=PASS" | tee -a "$SUMMARY"
  else
    fail_rounds=$((fail_rounds + 1))
    echo "round=$i result=FAIL log=${LOGDIR}/round-$(printf '%02d' "$i").log" | tee -a "$SUMMARY"
  fi
done

{
  echo "pass_rounds=$pass_rounds"
  echo "fail_rounds=$fail_rounds"
  echo "content_tip_end=$(git rev-parse HEAD)"
  if [[ "$fail_rounds" -eq 0 ]]; then
    echo "final=PASS"
  else
    echo "final=FAIL"
  fi
} | tee -a "$SUMMARY"

test "$fail_rounds" -eq 0
