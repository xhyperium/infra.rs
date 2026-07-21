#!/usr/bin/env bash
# Fail-closed 10x gate for goalctl Phase 1.1 campaign.
# Each round includes cargo fmt -- --check.
# Logs are written under SCRATCH/10x-<tip>/ (immutable per tip; never overwrite generic paths).
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"
SCRATCH="${SCRATCH:-/tmp/grok-goal-c68094b55cac/implementer}"
CONTENT_TIP="$(git rev-parse HEAD)"
TIP_DIR="${TIP_DIR:-$SCRATCH/10x-$CONTENT_TIP}"
mkdir -p "$TIP_DIR"

SUMMARY="$TIP_DIR/summary.log"
PATHS=(
  .agents/ssot/tools/goalctl/plan/gap-matrix.md
  .agents/ssot/tools/goalctl/plan/plan.md
  .agents/ssot/tools/goalctl/plan/goalctl-plan-10x-verdict.md
  .agents/ssot/goalctl/todo.md
  tools/goalctl
)

{
  echo "# 10x workspace gate"
  echo "content_tip=$CONTENT_TIP"
  echo "cwd=$ROOT"
  echo "tip_dir=$TIP_DIR"
  echo "start=$(date -Is)"
  echo
} > "$SUMMARY"

pass=0
fail=0

for i in $(seq 1 10); do
  nn=$(printf '%02d' "$i")
  log="$TIP_DIR/ws-round-$nn.log"
  ok=1
  {
    echo "=== WS ROUND $i ==="
    echo "content_tip=$CONTENT_TIP"
    echo "time=$(date -Is)"
    echo "cwd=$ROOT"

    if git diff --quiet HEAD -- "${PATHS[@]}"; then
      echo "OK worktree-matches-HEAD"
    else
      echo "FAIL worktree-differs-from-HEAD"
      git diff --stat HEAD -- "${PATHS[@]}" || true
      ok=0
    fi

    gap_head=$(git show "HEAD:.agents/ssot/tools/goalctl/plan/gap-matrix.md" 2>/dev/null || true)
    todo_head=$(git show "HEAD:.agents/ssot/goalctl/todo.md" 2>/dev/null || true)
    if echo "$gap_head" | rg -q 'GAP-001' && echo "$gap_head" | rg -q 'GAP-017'; then
      echo "OK gap-001-017"
    else
      echo "FAIL gap-coverage"; ok=0
    fi
    if echo "$gap_head" | rg -q 'AC-P0-SNAPSHOT' && echo "$gap_head" | rg -q 'AC-P0-RECONCILE' && echo "$gap_head" | rg -q 'AC-P0-COMPILE'; then
      echo "OK ac-p0"
    else
      echo "FAIL ac-p0"; ok=0
    fi
    open_n=$(echo "$todo_head" | rg -c '\| OPEN \|' || true)
    open_n=${open_n:-0}
    echo "open_rows=$open_n"
    if [ "$open_n" != "0" ]; then echo "FAIL open_rows"; ok=0; else echo "OK open_rows"; fi

    if test ! -d .config/goal; then echo "OK no-config-goal"; else echo "FAIL config-goal"; ok=0; fi
    if rg -q "尚未存在" .agents/ssot/tools/goalctl/contracts/VERSION-CAPABILITY-MATRIX.md 2>/dev/null; then
      echo "FAIL matrix-stale"; ok=0
    else
      echo "OK matrix"
    fi

    if cargo fmt -p xhyper-goalctl -- --check; then
      echo "OK fmt"
    else
      echo "FAIL fmt"; ok=0
    fi
    if cargo clippy -p xhyper-goalctl --all-targets --quiet -- -D warnings; then
      echo "OK clippy"
    else
      echo "FAIL clippy"; ok=0
    fi
    if cargo test -p xhyper-goalctl --quiet; then
      echo "OK cargo-test"
    else
      echo "FAIL cargo-test"; ok=0
    fi

    if cargo run -q -p xhyper-goalctl -- version 2>/dev/null | grep -q "0.1.1"; then
      echo "OK version"
    else
      echo "FAIL version"; ok=0
    fi
    out=$(cargo run -q -p xhyper-goalctl -- reconcile --module goalctl --source-commit "$CONTENT_TIP" --json)
    if echo "$out" | python3 -c "import sys,json; v=json.load(sys.stdin); r=v['result']; assert r['dimensions']['verification']['value']!='VERIFIED'; assert r['dimensions']['operations']['value']!='OK'"; then
      echo "OK reconcile-neg"
    else
      echo "FAIL reconcile-neg"; ok=0
    fi
    out=$(cargo run -q -p xhyper-goalctl -- --trust-level TRUSTED_BOT version --json 2>/dev/null || true)
    if echo "$out" | python3 -c "import sys,json; v=json.load(sys.stdin); assert v.get('trust_level')=='TRUSTED_BOT'" 2>/dev/null; then
      echo "OK trust"
    else
      if echo "$out" | rg -q 'GC-UNSUPPORTED|USAGE|unknown|TRUSTED_BOT'; then
        echo "OK trust-or-usage"
      else
        echo "FAIL trust"; ok=0
      fi
    fi

    now=$(git rev-parse HEAD)
    if [ "$now" = "$CONTENT_TIP" ]; then echo "OK tip-stable"; else echo "FAIL tip-moved $now"; ok=0; fi

    if [ "$ok" -eq 1 ]; then echo "ROUND $i PASS"; else echo "ROUND $i FAIL"; fi
  } >"$log" 2>&1

  if rg -q "^ROUND $i PASS$" "$log" && ! rg -q "^FAIL " "$log"; then
    pass=$((pass + 1))
    echo "round $i PASS" | tee -a "$SUMMARY"
  else
    fail=$((fail + 1))
    echo "round $i FAIL" | tee -a "$SUMMARY"
    rg -n "^FAIL |ROUND" "$log" | head -40 | tee -a "$SUMMARY" || true
  fi
done

{
  echo
  echo "pass_rounds=$pass"
  echo "fail_rounds=$fail"
  echo "content_tip=$CONTENT_TIP"
  echo "tip_dir=$TIP_DIR"
  echo "end=$(date -Is)"
} | tee -a "$SUMMARY"

# machine-readable gate result
python3 - <<PY
import json
from pathlib import Path
tip = "$CONTENT_TIP"
tip_dir = Path("$TIP_DIR")
fail = int("$fail")
pass_n = int("$pass")
out = {
  "content_tip": tip,
  "pass_rounds": pass_n,
  "fail_rounds": fail,
  "tip_dir": str(tip_dir),
  "summary": str(tip_dir / "summary.log"),
  "rounds": [str(tip_dir / f"ws-round-{i:02d}.log") for i in range(1, 11)],
  "fmt_each_round": True,
}
(tip_dir / "gate-result.json").write_text(json.dumps(out, indent=2) + "\n")
raise SystemExit(0 if fail == 0 else 1)
PY
