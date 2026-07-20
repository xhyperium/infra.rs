#!/usr/bin/env bash
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"
BRANCH="$(git rev-parse --abbrev-ref HEAD)"
SCRATCH="${SCRATCH:-/tmp/grok-goal-847be3632467/implementer}"
mkdir -p "$SCRATCH"
SSOT=".agents/ssot/types/canonical"
CRATE="crates/types/canonical"
fail() { echo "FAIL: $*" >&2; exit 1; }
ok() { echo "OK: $*"; }
echo "=== check-canonical-align branch=$BRANCH ==="
cargo metadata --no-deps --format-version 1 \
  | python3 -c 'import sys,json; n=[p["name"] for p in json.load(sys.stdin)["packages"]];
assert "xhyper-canonical" in n and "xhyper-decimalx" in n, n; print(n)'
test -f "$CRATE/src/lib.rs" || fail "no crate"
test -f crates/types/decimal/src/lib.rs || fail "no decimal"
test -f fixtures/market/order_cancel_okx.json || fail "no fixture"
test -s "$SSOT/plan/alignment-matrix-infra-2026-07-21.md" || fail "no matrix"
grep -q '\*\*Approved\*\*' "$SSOT/spec/spec.md" || fail "spec not Approved"
grep -qE '类型已删|类型已删除' "$SSOT/spec/spec.md" || fail "OrderId not deleted in spec"
grep -qE '纳秒|Unix \*\*ns\*\*' "$SSOT/spec/spec.md" || fail "ts ns missing"
! grep -q 'deprecated `OrderId`' "$SSOT/spec/spec.md" || fail "spec deprecated OrderId"
! grep -q 'deprecated `OrderId`' "$SSOT/plan/plan.md" || fail "plan deprecated OrderId"
! grep -E 'OPEN-TIME \|.*\| OPEN \|' "$SSOT/plan/plan.md" >/dev/null || fail "OPEN-TIME OPEN"
! grep -q 'not started' "$SSOT/goal/goal.md" || fail "goal placeholder"
s15=$(grep '| SAFE-15' "$SSOT/todo.md" || true)
echo "$s15" | grep -q DEFERRED || fail "SAFE-15 not DEFERRED: $s15"
s16=$(grep '| SAFE-16' "$SSOT/todo.md" || true)
echo "$s16" | grep -qE 'HUMAN_ONLY|DEFERRED' || fail "SAFE-16 not HUMAN: $s16"
t10=$(grep '| T-10X-001' "$SSOT/plan/tasks.md" || true)
echo "$t10" | grep -q DEFERRED || fail "T-10X not DEFERRED: $t10"
! rg -n 'type OrderId' "$CRATE/src" >/dev/null || fail "type OrderId in crate"
! rg -n '\bf32\b|\bf64\b' "$CRATE/src" --glob '*.rs' >/dev/null || fail "f32/f64 in crate"
cmp -s "$SSOT/spec/spec.md" "$SSOT/spec/xhyper-canonical-complete-spec.md" || fail "dual-mirror"
ok "authority facts"
cargo test -p xhyper-canonical -p xhyper-decimalx 2>&1 | tee "$SCRATCH/canonical-test.log" | tail -15
cargo clippy -p xhyper-canonical -p xhyper-decimalx --all-targets -- -D warnings 2>&1 | tee "$SCRATCH/canonical-clippy.log" | tail -5
cargo fmt -p xhyper-canonical -p xhyper-decimalx -- --check 2>&1 | tee "$SCRATCH/canonical-fmt.log"
{
  echo branch=$BRANCH
  rg -n 'type OrderId' "$CRATE/src" || echo no_OrderId
  rg -n 'contracts|domain' "$CRATE/Cargo.toml" || echo no_reverse
  rg -n 'checked_mul\(1_000_000\)' "$CRATE/src/proposed_time.rs"
  wc -c "$SSOT/plan/alignment-matrix-infra-2026-07-21.md"
} | tee "$SCRATCH/canonical-bounds.log"
echo "=== ALL CHECKS PASSED ==="
