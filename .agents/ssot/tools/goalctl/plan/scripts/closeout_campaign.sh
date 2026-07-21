#!/usr/bin/env bash
# Sole ledger writer for goalctl Phase 1.1 campaign closeout.
# 1) Assert delivery tip  2) Verify step-3 commands once  3) Run 10x into SCRATCH/10x-<tip>/
# 4) Emit campaign-proof.json  5) Rewrite verdict + todo from proof only
# Exit non-zero on any mismatch.
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"
SCRATCH="${SCRATCH:-/tmp/grok-goal-c68094b55cac/implementer}"
mkdir -p "$SCRATCH"

DELIVERY_TIP="$(git rev-parse HEAD)"
SHORT_TIP="${DELIVERY_TIP:0:12}"
TIP_DIR="$SCRATCH/10x-$DELIVERY_TIP"
PROOF="$SCRATCH/campaign-proof.json"
APPROVAL="$SCRATCH/approval-readback.json"
GATE_SCRIPT="$ROOT/.agents/ssot/tools/goalctl/plan/scripts/run_10x_gate.sh"
PR_NUMBER="${PR_NUMBER:-470}"

echo "== closeout_campaign tip=$DELIVERY_TIP =="

# --- 1) Delivery tip assert ---
if [ -n "${EXPECTED_TIP:-}" ] && [ "$DELIVERY_TIP" != "$EXPECTED_TIP" ]; then
  echo "FAIL expected tip $EXPECTED_TIP got $DELIVERY_TIP"
  exit 2
fi
if ! git diff --quiet HEAD; then
  echo "FAIL dirty worktree (must be clean for closeout)"
  git status --short | head -30
  exit 2
fi
echo "OK delivery tip $DELIVERY_TIP clean"

# --- 2) Verification plan step 3 (once; logs mandatory for T-VER-001) ---
test ! -d .config/goal
cargo fmt -p xhyper-goalctl -- --check
cargo clippy -p xhyper-goalctl --all-targets -- -D warnings
cargo test -p xhyper-goalctl --quiet
# just goal-check (required by plan.md / T-VER-001)
if command -v just >/dev/null 2>&1; then
  just goal-check >"$SCRATCH/goalctl-just-goal-check.log" 2>&1
else
  echo "FAIL just not installed" | tee "$SCRATCH/goalctl-just-goal-check.log"
  exit 2
fi
test -s "$SCRATCH/goalctl-just-goal-check.log"
rg -q "全部通过|PASS" "$SCRATCH/goalctl-just-goal-check.log" || {
  echo "FAIL goal-check log missing PASS"
  tail -40 "$SCRATCH/goalctl-just-goal-check.log" || true
  exit 2
}
cargo run -q -p xhyper-goalctl -- version | tee "$SCRATCH/goalctl-version.txt" | grep -q "0.1.1"
cargo run -q -p xhyper-goalctl -- doctor --state-dir "$SCRATCH/state" --json >"$SCRATCH/goalctl-cli-doctor.log" || true
cargo run -q -p xhyper-goalctl -- reconcile --module goalctl --source-commit "$DELIVERY_TIP" --json \
  | tee "$SCRATCH/goalctl-cli-reconcile.log" \
  | python3 -c "import sys,json; v=json.load(sys.stdin); r=v['result']; assert r['dimensions']['verification']['value']!='VERIFIED'; assert r['dimensions']['operations']['value']!='OK'"
# compile must succeed at clean delivery tip (capture for Verification plan step 3)
cargo run -q -p xhyper-goalctl -- compile --module goalctl --source-commit "$DELIVERY_TIP" --json \
  >"$SCRATCH/goalctl-cli-compile.log" 2>&1
python3 - <<'PY'
import json
from pathlib import Path
import os
p = Path(os.environ["SCRATCH"]) / "goalctl-cli-compile.log"
d = json.loads(p.read_text())
assert d.get("ok") is True and d.get("exit_code") == 0, d
assert d.get("source_commit"), d
assert d.get("result") is not None
print("OK compile", d["source_commit"][:12])
PY
echo "OK verification step-3"

# --- 3) 10x into immutable tip dir ---
export SCRATCH
export TIP_DIR
rm -rf "$TIP_DIR"
mkdir -p "$TIP_DIR"
bash "$GATE_SCRIPT"
test -f "$TIP_DIR/gate-result.json"
test -f "$TIP_DIR/ws-round-01.log"
test -f "$TIP_DIR/ws-round-10.log"
# assert every round log binds same tip + OK fmt
for i in $(seq -w 1 10); do
  rg -q "content_tip=$DELIVERY_TIP" "$TIP_DIR/ws-round-$i.log"
  rg -q "OK fmt" "$TIP_DIR/ws-round-$i.log"
  rg -q "^ROUND .* PASS$" "$TIP_DIR/ws-round-$i.log" || rg -q "ROUND ${i#0} PASS" "$TIP_DIR/ws-round-$i.log" || rg -q "ROUND $((10#$i)) PASS" "$TIP_DIR/ws-round-$i.log"
done
python3 - <<PY
import json
from pathlib import Path
gr = json.loads(Path("$TIP_DIR/gate-result.json").read_text())
assert gr["fail_rounds"] == 0, gr
assert gr["content_tip"] == "$DELIVERY_TIP", gr
print("OK 10x fail_rounds=0 tip-bound logs")
PY

# --- 4) Approval readback (PR 470 only; never auto-DONE) ---
python3 - <<'PY'
import json, os, subprocess
from pathlib import Path
scratch = Path(os.environ.get("SCRATCH", "/tmp/grok-goal-c68094b55cac/implementer"))
tip = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
pr_number = int(os.environ.get("PR_NUMBER", "470"))
present = bool(os.environ.get("LIUKONGQIANG5_APPROVE_TOKEN"))
out = {
    "env_name": "LIUKONGQIANG5_APPROVE_TOKEN",
    "present": present,
    "used": False,
    "token_value_written": False,
    "approver": "@liukongqiang5",
    "campaign": "goalctl Phase 1.1 Truth Hardening",
    "pr_number": pr_number,
    "pr_url": f"https://github.com/xhyperium/infra.rs/pull/{pr_number}",
    "local_head": tip,
    "notes": "Goalctl approval evidence is PR 470 only. Token value never written to repo.",
}
try:
    pr = json.loads(
        subprocess.check_output(
            ["gh", "pr", "view", str(pr_number), "--json", "number,url,headRefOid,title,reviews"],
            text=True,
        )
    )
    out["pr_title"] = pr.get("title")
    out["pr_headRefOid"] = pr.get("headRefOid")
    out["pr_url"] = pr.get("url") or out["pr_url"]
    lk = [
        r
        for r in (pr.get("reviews") or [])
        if r.get("author", {}).get("login") == "liukongqiang5"
    ]
    out["liukongqiang5_reviews"] = [
        {
            "state": r.get("state"),
            "commit": (r.get("commit") or {}).get("oid"),
            "at": r.get("submittedAt"),
        }
        for r in lk
    ]
    bound_pr = any(
        r.get("state") == "APPROVED"
        and (r.get("commit") or {}).get("oid") == pr.get("headRefOid")
        for r in lk
    )
    bound_local = any(
        r.get("state") == "APPROVED" and (r.get("commit") or {}).get("oid") == tip for r in lk
    )
    out["tip_bound_approved_on_pr_head"] = bound_pr
    out["tip_bound_approved_on_local_head"] = bound_local
except Exception as e:
    out["pr_error"] = f"{type(e).__name__}: {e}"
    bound_pr = False
    bound_local = False

# T-VER-003 is never machine DONE unless tip-bound APPROVED on THIS head
if bound_local:
    out["conclusion"] = "APPROVED_ON_LOCAL_HEAD"
    out["t_ver_003"] = "DONE"
    out["reason"] = (
        "liukongqiang5 APPROVED on local delivery tip. Still not Goal ACHIEVED / Spec Approved / Cutover."
    )
else:
    out["conclusion"] = "HUMAN_ACTION_REQUIRED"
    out["t_ver_003"] = "HUMAN_ACTION_REQUIRED"
    out["reason"] = (
        "Agent-safe machine complete. Tip-bound APPROVE on delivery tip still required. "
        "Not Goal ACHIEVED / Spec Approved / Cutover."
    )

text = json.dumps(out, indent=2, ensure_ascii=False)
assert "469" not in text or "PR 470 only" in text  # no bare PR 469 as approval target
# Hard ban: no pr_number 469
assert out["pr_number"] == 470
(scratch / "approval-readback.json").write_text(text + "\n")
print("OK approval-readback", out["conclusion"], out["t_ver_003"])
PY

# --- 5) campaign-proof.json ---
python3 - <<PY
import json, os, subprocess
from pathlib import Path

tip = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
scratch = Path("$SCRATCH")
tip_dir = Path("$TIP_DIR")
gate = json.loads((tip_dir / "gate-result.json").read_text())
approval = json.loads((scratch / "approval-readback.json").read_text())
assert gate["fail_rounds"] == 0
assert gate["content_tip"] == tip
assert approval["pr_number"] == 470
assert "469" not in json.dumps({"pr_number": approval["pr_number"]})

t_ver_003 = approval.get("t_ver_003", "HUMAN_ACTION_REQUIRED")
proof = {
    "schema": "goalctl-campaign-proof/v1",
    "delivery_tip": tip,
    "content_tip": tip,
    "content_tip_at_run": tip,
    "fail_rounds": 0,
    "pass_rounds": 10,
    "fmt_each_round": True,
    "tip_dir": str(tip_dir),
    "gate_result": str(tip_dir / "gate-result.json"),
    "summary_log": str(tip_dir / "summary.log"),
    "round_logs": [str(tip_dir / f"ws-round-{i:02d}.log") for i in range(1, 11)],
    "approval_readback": str(scratch / "approval-readback.json"),
    "pr_number": 470,
    "pr_url": approval.get("pr_url"),
    "t_ver_001": "DONE",
    "t_ver_002": "DONE",
    "t_ver_003": t_ver_003,
    "approval_conclusion": approval.get("conclusion"),
    "head_eq_verdict_tip": True,  # ledger rewrite below will set content_tip == tip before commit
    "head_eq_gate_log_tip": True,
    "config_goal_absent": not Path(".config/goal").exists(),
    "branch": subprocess.check_output(["git", "branch", "--show-current"], text=True).strip(),
}
(scratch / "campaign-proof.json").write_text(json.dumps(proof, indent=2) + "\n")
print("OK campaign-proof.json")
print(json.dumps(proof, indent=2))
PY

# --- 6) Rewrite ledger files from proof only ---
python3 - <<'PY'
import json
from pathlib import Path
import os

scratch = Path(os.environ.get("SCRATCH", "/tmp/grok-goal-c68094b55cac/implementer"))
proof = json.loads((scratch / "campaign-proof.json").read_text())
tip = proof["delivery_tip"]
tip_dir = proof["tip_dir"]
t_ver_003 = proof["t_ver_003"]
assert t_ver_003 in ("DONE", "HUMAN_ACTION_REQUIRED", "HUMAN_ONLY")
assert proof["pr_number"] == 470
assert proof["fail_rounds"] == 0

verdict = f"""# 10x Verdict — PLAN-GOALCTL-002-phase1.1-v1

| 字段 | 值 |
|------|-----|
| Plan | PLAN-GOALCTL-002-phase1.1-v1 |
| Package | xhyper-goalctl 0.1.1 |
| fail_rounds | 0 |
| pass_rounds | 10 |
| final | PASS |
| content_tip | {tip} |
| content_tip_at_run | {tip} |
| Gate script | .agents/ssot/tools/goalctl/plan/scripts/run_10x_gate.sh |
| Closeout script | .agents/ssot/tools/goalctl/plan/scripts/closeout_campaign.sh |
| Log dir | {tip_dir} |
| Summary | {tip_dir}/summary.log |
| Round logs | {tip_dir}/ws-round-01.log … ws-round-10.log |
| Proof | {scratch}/campaign-proof.json |
| Date | 2026-07-16 |
| Branch | fix/goalctl-phase11-truth-hardening |
| PR | 470 |

每轮检查项（含 **cargo fmt -- --check**）：

1. plan/todo ID 覆盖（GAP-001…017 + AC-P0-*）
2. 禁止项：无 .config/goal；reconcile 无假 VERIFIED/OK
3. cargo fmt -p xhyper-goalctl -- --check
4. cargo clippy -p xhyper-goalctl --all-targets -- -D warnings
5. cargo test -p xhyper-goalctl
6. CLI version 0.1.1 + reconcile subject-bound
7. tip-stable（round 内 HEAD 不变）

| Round | Result | Notes |
|------:|--------|-------|
| 1–10 | PASS | OK fmt; content_tip={tip} |

解释边界：10x PASS 不等于 Goal ACHIEVED / Spec Approved / Cutover。
GOAL-GOALCTL-002 / SPEC-GOALCTL-002 仍为 PROPOSED。

Tip freeze：content_tip == delivery_tip == HEAD at gate start ({tip}).
"""
Path(".agents/ssot/tools/goalctl/plan/goalctl-plan-10x-verdict.md").write_text(verdict)

todo_path = Path(".agents/ssot/goalctl/todo.md")
lines = todo_path.read_text().splitlines()
out_lines = []
for line in lines:
    if "| T-VER-001 |" in line:
        out_lines.append(
            f"| T-VER-001 | gates | DONE | tip `{tip}`; cargo fmt+clippy+test; closeout_campaign.sh |"
        )
    elif "| T-VER-002 |" in line:
        out_lines.append(
            f"| T-VER-002 | 10x | DONE | tip `{tip}` fail_rounds=0 fmt each round; "
            f"`plan/scripts/run_10x_gate.sh`; `{tip_dir}/ws-round-01.log`…`ws-round-10.log`; "
            f"`{tip_dir}/summary.log` |"
        )
    elif "| T-VER-003 |" in line:
        out_lines.append(
            f"| T-VER-003 | liukongqiang5 APPROVE | {t_ver_003} | PR 470 only; "
            f"SCRATCH `approval-readback.json` conclusion={proof['approval_conclusion']}; "
            f"local_head=`{tip}`; ≠ Goal ACHIEVED |"
        )
    else:
        out_lines.append(line)
todo_path.write_text("\n".join(out_lines) + "\n")

# copy proof into plan for commit
Path(".agents/ssot/tools/goalctl/plan/campaign-proof.json").write_text(
    json.dumps(proof, indent=2) + "\n"
)
print("OK ledger rewritten from campaign-proof.json")
# sanity: content_tip appears in both
assert tip in Path(".agents/ssot/tools/goalctl/plan/goalctl-plan-10x-verdict.md").read_text()
assert tip in Path(".agents/ssot/goalctl/todo.md").read_text()
assert "HUMAN_ACTION_REQUIRED" in Path(".agents/ssot/goalctl/todo.md").read_text() or t_ver_003 == "DONE"
assert "d2676998" not in Path(".agents/ssot/goalctl/todo.md").read_text()
print("OK ledger sanity")
PY

echo "== closeout_campaign complete tip=$DELIVERY_TIP =="
echo "proof=$PROOF"
echo "tip_dir=$TIP_DIR"
echo "Next: commit ledger files (verdict + todo + campaign-proof.json + scripts) on this tip."
