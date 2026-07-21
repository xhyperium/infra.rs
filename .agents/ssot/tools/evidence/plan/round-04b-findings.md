# Round 04b Findings — Evidence Plan Completeness (post v1.1)

```text
round: 4b
result: PASS
scope: full R-* checklist · adapters §19–§21 · crash recovery · concurrency · durability
baseline: main@007ca7b5
sources_checked:
  - xhyper-evidence-complete-spec.md (§19–§21)
  - plan/plan.md v1.1
  - plan/gap-matrix.md
  - plan/tasks.md
  - plan/spec-inventory.md (I-5/I-17/I-19)
  - plan/residual-open.md
  - plan/approval-packet.md
  - .worktree/evidence-todo.md
failed_checks: []
omissions:
  - file segment header field list (magic/format_version/previous_segment_head) still by §ref + dual-len AC, not full header field table in tasks
  - payload_len hard-limit numeric policy not a separate Task (acceptable under T-FILE-002 scope)
  - systemd unit inventory not named (bootstrap+archgate cover production exclusion)
  - Reader start_sequence≥1 / no silent skip corrupt not restated in T-MEM-003 (in T-CORE-023 §14 ref + conformance hook)
false_pass_risks:
  - T-FILE-005 DONE after implementing <10 recovery steps without I-17 checklist
  - T-PG-006 DONE without connection-drop / expected_head cases despite AC listing them
  - T-BOOT-001 ↔ T-ARCH-011 circular depends may stall scheduling
notes: |
  Round 4b adapter-depth re-eval. Prior FAIL set:
  R-MEM-001 (no bootstrap; zero-head; systemd),
  R-FILE-001 (frame/fsync/recovery under-specified),
  R-PG-001 (head columns / concurrency incomplete),
  R-APPEND-001 (Process durability missing),
  R-ATOM-001 (phantom T-ATOM),
  R-READ-001 (limit/corrupt semantics thin),
  R-SPEC-003 (§33.3 + ghost).

  v1.1 coverage now sufficient for plan checklist:
  - T-MEM-004/007/010 + T-BOOT-001 + T-ARCH-005 (production ban)
  - T-FILE-002 dual u32+commit; T-FILE-004 write→fdatasync→dir meta→head;
    T-FILE-005 I-17 10 steps; T-FILE-008 fault matrix; T-FILE-010 group commit
  - T-PG-002 unique (chain,seq)/(chain,event) + head columns;
    T-PG-006 并发/死锁/回滚/断连/expected_head
  - T-CORE-022/038 Process; I-5 Durability triad
  - T-ATOM-001…006 real IDs
  - T-CORE-023 limit 1..=10000

  Fair PASS per "inventory+tasks cover → PASS".
```

## Full R-* evaluation (Round 4b)

| Check ID | Result | Adapter-focus notes |
|----------|--------|---------------------|
| R-SPEC-001 | **PASS** | |
| R-SPEC-002 | **PASS** | §19–§21 rows present. |
| R-SPEC-003 | **PASS** | §33.3 adapter map complete. |
| R-GAP-001/002 | **PASS** | |
| R-PATH-001…003 | **PASS** | adapters under `crates/adapters/evidence/*`. |
| R-DEP-001/002 | **PASS** | |
| R-API-001…003 | **PASS** | |
| R-CANON-001…004 | **PASS** | |
| R-TIME-001 / R-CHAIN-001 | **PASS** | |
| R-APPEND-001 | **PASS** | I-5 Volatile/Process/Durable + T-BOOT-001 production Durable. |
| R-APPEND-002 | **PASS** | CAS/idempotency/fail-closed tasked. |
| R-READ-001 | **PASS** | T-CORE-023 + T-MEM-003 + T-MEM-010. |
| R-ATOM-001 | **PASS** | A/B/C + PG/outbox. |
| R-CP-001 | **PASS** | recovery×checkpoint T-CP-008. |
| R-ERR-001 | **PASS** | |
| R-MEM-001 | **PASS** | production_allowed=false; ban pseudo-Durable; bootstrap; zero-head. |
| R-FILE-001 | **PASS** | frame + fsync path + I-17 recovery + chaos. |
| R-PG-001 | **PASS** | four tables + invariants + concurrency suite. |
| R-TEST-001 | **PASS** | crash_recovery / concurrency packages in I-20. |
| R-CLI-001 | **PASS** | repair-tail depends file recovery. |
| R-POL-001 | **PASS** | |
| R-GATE-001 | **PASS** | DURABILITY/FSYNC/RECOVERY/IDEMPOTENCY/CONCURRENCY gates. |
| R-MIG-001 | **PASS** | |
| R-EVID-001 | **PASS** | adapter-conformance / recovery-tests slots. |
| R-DOWN-001 | **PASS** | |
| R-GOV-001 | **PASS** | |
| R-FORBID-001 | **PASS** | no dual-name without T-LEG-003. |
| R-TODO-001 | **PASS** | W2/W4 listed. |
| R-CROSS-001 | **PASS** | |
| R-HONEST-001 | **PASS** | adapters still ABSENT in code — correct. |

**FAIL count: 0** → **round result = PASS**

## Adapter contract matrix (plan layer)

| Spec area | Key Tasks | Status |
|-----------|-----------|--------|
| Memory §19 | T-MEM-001…010, T-BOOT-001, T-ARCH-005 | tasked |
| File §20 | T-FILE-001…010, I-17 | tasked |
| Postgres §21 | T-PG-001…007 | tasked |
| Atomicity §15 | T-ATOM-001…006, T-ARCH-016 | tasked |

## Verdict

```text
result: PASS
failed_checks: []
fail_rounds_contribution: 0
prior_round: round-04 FAIL (7 checks) — remediated in v1.1
```
