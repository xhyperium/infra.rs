> **历史 monorepo 记录**（infra.rs）：文中 archgate / `.architecture` 不构成本仓验收条件；本仓不移植 archgate。

# Round 02 Findings — Evidence Plan Completeness (Verifier)

```text
round: 2
result: FAIL
scope: systematic SPEC-EVIDENCE-002 §18–§34 + full R-* checklist + tasks↔§33 cross-link
baseline: main@007ca7b5
sources_checked:
  - .agents/ssot/tools/evidence/xhyper-evidence-complete-spec.md (§18–§34 + residual §0–§17 cross refs)
  - .agents/ssot/tools/evidence/plan/plan.md
  - .agents/ssot/tools/evidence/plan/gap-matrix.md
  - .agents/ssot/tools/evidence/plan/tasks.md
  - .agents/ssot/tools/evidence/plan/approval-packet.md
  - .worktrees/evidence-todo.md
  - round-01-findings.md (prior adversarial pass)
failed_checks:
  - R-SPEC-003
  - R-DEP-002
  - R-CANON-001
  - R-ATOM-001
  - R-ERR-001
  - R-GATE-001
  - R-TEST-001
  - R-POL-001
omissions: [see §Omissions]
false_pass_risks: [see §False-pass risks]
notes: |
  Round 2 re-evaluates the full 40-item checklist independently and deep-scans
  §18–§34. Confirms Round 1 FAILs and adds R-TEST-001 + R-POL-001 as FAIL under
  adversarial reading of §24/§26/§28 completeness. Cross-link tasks↔§33 has
  phantom IDs and DEFER-without-formal-record items. Metrics §30, gate §27,
  error §18, CLI exit meanings, migration P0–P6 vs Waves reviewed.
```

---

## 1. Full R-* checklist evaluation (Round 2 independent)

| Check ID | Result | Round-2 notes (§18–§34 emphasis) |
|----------|--------|----------------------------------|
| R-SPEC-001 | **PASS** | Unchanged. |
| R-SPEC-002 | **PASS** | §18–§34 present in gap-matrix. |
| R-SPEC-003 | **FAIL** | tasks↔§33 map incomplete (detail §3 below). |
| R-GAP-001 | **PASS** | DEF-001…018 present. |
| R-GAP-002 | **PASS** | T1–T18 present. |
| R-PATH-001 | **PASS** | |
| R-PATH-002 | **PASS** | |
| R-PATH-003 | **PASS** | |
| R-DEP-001 | **PASS** | |
| R-DEP-002 | **FAIL** | Full §4.2 list still missing from plan pack. |
| R-API-001 | **PASS** | |
| R-API-002 | **PASS** | |
| R-API-003 | **PASS** | |
| R-CANON-001 | **FAIL** | 25-step table still not in plan pack; golden AC is “§24.1 列表全覆盖” by ref only. |
| R-CANON-002 | **PASS*** | |
| R-CANON-003 | **PASS** | |
| R-CANON-004 | **PASS** | |
| R-TIME-001 | **PASS*** | |
| R-CHAIN-001 | **PASS*** | |
| R-APPEND-001 | **PASS** | |
| R-APPEND-002 | **PASS*** | |
| R-READ-001 | **PASS*** | |
| R-ATOM-001 | **FAIL** | Phantom T-ATOM; §15/§21 outbox incomplete vs full contract. |
| R-CP-001 | **PASS*** | T-CP covers signer/anchor/tail; §16.5 hard-deadline block write missing. |
| R-ERR-001 | **FAIL** | **Deep:** 24 variants + 6-way XError map not in plan pack (see §2.1). |
| R-MEM-001 | **PASS** | |
| R-FILE-001 | **PASS*** | Frame format details only “§20.3”; commit-marker/dual length/magic not listed. |
| R-PG-001 | **PASS*** | Tables named; head columns (sequence/head_digest/updated_at) not in AC. |
| R-TEST-001 | **FAIL** | Adversarial: §24 requires named suites (24.1 vector set, 24.4 checkpoint props, 24.5–24.11). Plan maps fuzz/branch/mutants/miri/historical almost entirely to **T-CI-002 规划**, not closeable ACs. Mutation “must not survive” list (§24.10) **absent**. Branch≥90% no dedicated W1 AC (only line≥95% T-CORE-033). |
| R-CLI-001 | **PASS*** | Commands present; §2.3 “vectors **generate**” vs §25.1 only verify — generate missing. Exit codes listed as numbers only (meanings omission). |
| R-POL-001 | **FAIL** | T-POL-001 skeleton only. Spec §26 requires per-required-op fields: producer, operation, subject/chain/actor strategy, input/output canonical domains, atomicity, durability, checkpoint, retention, owner — **not all in AC**. retention_days example 2555 not tasked. |
| R-GATE-001 | **FAIL** | See full gate inventory §2.2 — **9+ IDs missing** from tasks. |
| R-MIG-001 | **PASS*** | P0–P6↔Waves generally aligned; see §2.5 caveats. |
| R-EVID-001 | **PASS** | plan §8 ≡ §32 tree. |
| R-DOWN-001 | **PASS** | |
| R-GOV-001 | **PASS** | |
| R-FORBID-001 | **PASS** | |
| R-TODO-001 | **PASS*** | |
| R-CROSS-001 | **PASS** | INFRA-003 boundary intact. |
| R-HONEST-001 | **PASS** | No false Approved/stable. |

**FAIL count: 8** → **round result = FAIL**.

---

## 2. Deep-scan §18–§34

### 2.1 EvidenceError variants (§18.1) — required enumeration

Spec lists **24** variants. Plan/tasks: “§18.1 全 variant” only — **none listed**.

```text
InvalidName
InvalidDraft
InvalidEncoding
UnsupportedVersion
MissingChain
MissingRecord
IdempotencyConflict
HeadConflict
SequenceOverflow
SequenceGap
DuplicateSequence
DuplicateEventId
ChainIdMismatch
PreviousDigestMismatch
RecordDigestMismatch
ForkDetected
CheckpointMismatch
SignatureInvalid
TailTruncated
CorruptStorage
StorageUnavailable
DurabilityFailure
ClockUnavailable
SynchronizationFailure
```

### 2.2 XError mapping (§18.2) — incomplete in plan

| Spec mapping | In tasks? |
|--------------|-----------|
| InvalidName/Draft/Encoding → Invalid | **NO** |
| MissingChain/Record → Missing | **NO** |
| IdempotencyConflict/HeadConflict/Duplicate* → Conflict | **NO** |
| StorageUnavailable/DurabilityFailure/ClockUnavailable/UnsupportedVersion → Unavailable | **NO** |
| SequenceOverflow/Gap/ChainIdMismatch/Previous*/Record*/Fork/Checkpoint/Signature/Tail/Corrupt → **Invariant** | partial (“链损坏→Invariant”) |
| SynchronizationFailure → Internal | **NO** |
| “链损坏绝不映射为普通 Invalid” | spirit only |

### 2.3 All EVIDENCE-* gate IDs (§27)

| Gate ID | tasks coverage |
|---------|----------------|
| EVIDENCE-PATH-001 | T-ARCH-001 |
| EVIDENCE-DEP-001 | T-ARCH-002 (bundled) |
| EVIDENCE-DEP-002 | T-ARCH-002 (bundled) |
| EVIDENCE-ANYHOW-001 | T-ARCH-003 |
| **EVIDENCE-CANONICAL-001** | **MISSING** |
| EVIDENCE-DOMAIN-001 | T-ARCH-004 abbrev |
| EVIDENCE-DEBUG-HASH-001 | T-ARCH-004 abbrev |
| EVIDENCE-JSON-HASH-001 | T-ARCH-004 abbrev |
| EVIDENCE-GENESIS-001 | T-ARCH-004 abbrev |
| EVIDENCE-PUBLIC-001 | T-ARCH-004 abbrev |
| **EVIDENCE-DURABILITY-001** | **MISSING** |
| EVIDENCE-MEMORY-PROD-001 | T-ARCH-005 |
| **EVIDENCE-IDEMPOTENCY-001** | **MISSING** |
| **EVIDENCE-CONCURRENCY-001** | **MISSING** |
| **EVIDENCE-RECOVERY-001** | **MISSING** |
| **EVIDENCE-FSYNC-001** | **MISSING** |
| EVIDENCE-POLICY-001 | T-ARCH-006 abbrev |
| EVIDENCE-COVERAGE-001 | T-ARCH-006 abbrev |
| EVIDENCE-ATOMICITY-001 | T-ARCH-006 abbrev |
| EVIDENCE-CHECKPOINT-001 | T-ARCH-006 abbrev |
| **EVIDENCE-ANCHOR-001** | **MISSING** |
| **EVIDENCE-SCHEMA-001** | **MISSING** |
| **EVIDENCE-VECTOR-001** | **MISSING** |

**Score: 5 explicit + ~9 abbreviated/bundled + 9 fully missing ≈ incomplete for R-GATE-001.**

### 2.4 CLI exit codes (§25.3)

T-CLI-006: `0/2/3/4/5/6/7` — codes present; **meanings not restated**:

```text
0  valid / success
2  invalid arguments
3  chain invalid
4  checkpoint/signature invalid
5  storage unavailable
6  unsupported version
7  repair required
```

repair-tail constraints (only incomplete last frame; backup; no past trusted checkpoint) partially in T-CLI-005.

Missing vs §2.3: CLI **generate** golden vectors (only verify tasked).

### 2.5 Metrics list (§30)

T-OBS-001: “§30；无高基数敏感 label” — **names not listed** in plan pack:

```text
evidence_append_total
evidence_append_failures_total
evidence_append_latency_seconds
evidence_durability_failures_total
evidence_chain_head_sequence
evidence_checkpoint_age_seconds
evidence_checkpoint_failures_total
evidence_verify_failures_total
evidence_outbox_backlog
evidence_recovery_seconds
evidence_fork_detected_total
```

Also: core must not depend observex — stated in gap §30 row, not in T-CORE-002 AC text (deps whitelist implies it).

### 2.6 Migration P0–P6 vs Waves

| Spec phase | Plan Wave / Tasks | Alignment |
|------------|-------------------|-----------|
| P0 freeze | W0 T-FREEZE/T-DOC | OK (tasks still TODO) |
| P1 Core V1 | W1 T-CORE-* | OK |
| P2 Compatibility bridge | W3 T-LEG-001/002 (depends T-CORE-025 only) | **OK but Wave label “W3” mixes with domain**; gap §6 says P2 before P3 — dependency graph allows parallel; document could clarify P2≠domain |
| P3 Domain migration | W3 T-DOM/T-GATE | OK |
| P4 Durable adapters | W2 memory + W4 file/pg | OK (memory conformance first) |
| P5 Checkpoint | W5 T-CP | OK; CLI bundled same Wave |
| P6 Cutover | W6 T-CUT/T-ARCH | OK; registry stable deferred W8/W9 |

**R-MIG-001 PASS** with note: P2 naming under W3 is slightly confusing but not contradictory.

### 2.7 §19–§21 adapters

| Spec | Gap | Task depth |
|------|-----|------------|
| Memory real seal/idempotency/CAS; ban always-success verify | DEF-007 | T-MEM-002…006 OK |
| File segment header/frames/footer/commit marker | ABSENT | T-FILE-002 by §ref — magic/format_version/dual u32_len not listed |
| File group commit / directory fsync | | not explicit |
| File recovery 10-step startup | | T-FILE-005 partial |
| PG FOR UPDATE append | | T-PG-003 OK |
| PG unique (chain_id,sequence)/(chain_id,event_id) | | T-PG-002 “唯一约束” OK |
| PG ban delete outbox before durable | | T-PG-004 OK |
| PG concurrency suite list | | T-PG-006 partial |

### 2.8 §22 Retention / privacy

gap: W5/W7. tasks: only `T-POL-002 + docs` for artifacts; **no**:

- content-addressed artifact store contract (object key = digest)
- encryption / access control / Object Lock
- deletion/erasure evidence pattern
- verifier/schema/public key retention as first-class tasks  
→ system §33.6 “source artifacts retention” / “verifier/schema/keys” weakly mapped.

### 2.9 §23 Schema evolution

- V1 freeze rules partially via golden + T-CORE-026
- dual-read / V2 new domain tag policy: **no task**
- algorithm migration checkpoint dual-anchor: **no task**
- historical schema: T-LEG-002 + T-CI-002 only

### 2.10 §24 Test contract detail gaps

| § | Required | Plan |
|---|----------|------|
| 24.1 | Named golden set (empty/genesis, 6 outcomes, optionals, max name, pre-epoch event_time, boundaries, multi-record, checkpoint) | “§24.1 列表全覆盖” **by reference only** |
| 24.4 | Checkpoint property suite | T-CP-004/006 partial; no full suite task |
| 24.5 | Idempotency suite | folded into MEM/PG |
| 24.8 | Fuzz targets: decoder, verifier, segment parser, checkpoint parser, CLI import | not listed |
| 24.9 | branch ≥90%; adapter line≥90%; recovery path 100% scenarios | line≥95% only (core) |
| 24.10 | mutants ≥90% + “must not survive” mutation list | T-CI-002 规划 only; list **ABSENT** |
| 24.11 | Miri core+memory | T-CI-002 规划 only |

### 2.11 §28 CI commands

plan §4.2 lists a subset. Missing explicit tasks for:

- `cargo llvm-cov -p evidence --fail-under-lines 95` (T-CORE-033 soft)
- `cargo mutants -p evidence`
- `cargo miri test -p evidence`
- `cargo run -p archgate -- --json`
- `cargo run -p xtask -- crate-standard --check`
- adapter crash_recovery / concurrency test packages
- Nightly: full fuzz/mutants/chaos/key rotation/historical schema

### 2.12 §29 Performance / backpressure

T-PERF-001/002 skeleton exist. **Missing §29.3 backpressure**:

- required ops fail-closed on lag
- StorageUnavailable/DurabilityFailure explicit
- no unbounded memory cache
- no drop old evidence
- no silent Volatile fallback

### 2.13 §31 current problems list

gap DEF + §0 overview covers essentially all §31.1 bullets — **PASS** for registration.

### 2.14 §32 / §33 / §34

- §32 template: plan §8 complete (**PASS** R-EVID-001).
- §33 checkboxes: mapping table exists but **cross-link defects** (§3).
- §34 final framing: plan §0.1 / §11 echoes correctly; not falsely claimed done (**PASS** R-HONEST-001).

### 2.15 Current code bugs + INFRA-003 (reconfirm)

Same as Round 1: tools/evidence hash_bytes, zero genesis, anyhow, public fields, mock always-ok, domain_macro Debug hash, gate name hash — all DEF-registered. INFRA-003 boundary explicit. No plan step confuses WP CI evidence with runtime chain.

### 2.16 Outcome tags / preimage (cross from §8/§11)

Still omitted from plan pack (see Round 1 §2.1–2.2) — affects golden freeze readiness before W1.

---

## 3. Cross-link: tasks.md ↔ §33 (adversarial)

### 33.1 规格闭合

| Item | Task | Verdict |
|------|------|---------|
| Approved | T-HUM-001 | OK (human) |
| old superseded | T-DOC-002 T-HUM-002 | OK |
| ADR | T-DOC-004 | OK |
| path/package | T-CUT-002/003 | OK |
| registry | T-REG-001/002 | OK |
| policy.toml | T-POL-001/002 | OK but POL content thin |
| no Unknown residual | T-RES-001 T-SKEP-001 | T-RES-001 still TODO |

### 33.2 Core

Mostly OK (T-CORE-*). Gaps: canonical freeze depends on unlisted 25-step/golden names; no explicit task for “no JSON hash” beyond T-ARCH-004 abbrev.

### 33.3 Adapter

OK structure; disk full/short write/fsync via T-FILE-008 only (acceptable if AC expands).

### 33.4 Checkpoint

| Item | Map | Issue |
|------|-----|-------|
| signed | T-CP-002 | OK |
| key rotation | T-CP-006 | OK |
| independent anchor | T-CP-005 | interface only; production anchor DEFER in approval A8 |
| tail truncation | T-CP-004 | OK |
| **full chain replacement** | `T-CP-005 + verify` | **vague** — no dedicated detection AC |
| startup verify | T-FILE-005 T-CP-004 | partial |

### 33.5 测试

| Item | Map | Issue |
|------|-----|-------|
| golden | T-CORE-026/027 T-CLI-004 | OK structure |
| property | T-CORE-028/030 | OK |
| fuzz | **T-CI-002** | planning only |
| line≥95% | T-CORE-033 | OK |
| branch≥90% | **T-CI-002** | planning only |
| mutants≥90% | **T-CI-002** | planning only |
| Miri | **T-CI-002** | planning only |
| adapter chaos | T-FILE-008 T-PG-006 | OK-ish |
| historical schema | T-LEG-002 T-CI-002 | weak |

### 33.6 系统

| Item | Map | Issue |
|------|-----|-------|
| required ops | T-POL-002 | OK |
| fail-closed | T-DOM-005 | only domain_macro, not all Tier-A |
| Tier-A atomicity | **`T-ATOM via design`** | **PHANTOM TASK ID — FAIL** |
| external Attempted+terminal | **（订单域后续；policy 预留）** | **NO TASK — FAIL** |
| source artifacts retention | T-POL-002 + docs | weak |
| verifier/schema/keys retention | T-CP-006 + docs | weak |
| CI Evidence | T-EVID-SYS T-CI-001 | OK structure |

**Cross-link verdict: FAIL** (phantom T-ATOM + missing external path task + over-reliance on T-CI-002 planning).

---

## 4. Omissions (machine-list)

```text
omissions:
  # from Round 1 still open
  - §11.2 25-step preimage order not embedded
  - Outcome tag map 0x00..0x05 not listed
  - §11.5 CONTENT digest framing absent
  - §4.2 full forbid list incomplete
  - §15.1/15.3/15.4 atomic patterns incomplete / phantom T-ATOM
  - §12.2 overflow / §12.4 fork protocol under-specified
  # Round 2 primary
  - §18.1 24 EvidenceError variants not listed in plan/tasks
  - §18.2 full XError mapping table not listed
  - EVIDENCE-CANONICAL-001 missing task
  - EVIDENCE-DURABILITY-001 missing task
  - EVIDENCE-IDEMPOTENCY-001 missing task
  - EVIDENCE-CONCURRENCY-001 missing task
  - EVIDENCE-RECOVERY-001 missing task
  - EVIDENCE-FSYNC-001 missing task
  - EVIDENCE-ANCHOR-001 missing task
  - EVIDENCE-SCHEMA-001 missing task
  - EVIDENCE-VECTOR-001 missing task
  - §25.3 exit code meanings not restated
  - §2.3/CLI golden vector generate command not tasked
  - §30 eleven metric names not listed in tasks
  - §24.1 named golden vector inventory not copied into tasks
  - §24.10 mutation “must not survive” list absent
  - §24.8 fuzz target list absent
  - §24.9 branch/adapter coverage ACs incomplete
  - §26 full required-operation field set not in T-POL AC
  - §22 artifact store / erasure / multi-retention dimensions under-tasked
  - §23 V2/dual-read/algorithm migration policy no task
  - §29.3 backpressure fail-closed rules no task
  - §16.5 checkpoint hard-deadline blocks required writes no task
  - §16.1 checkpoint preimage byte layout not enumerated
  - SignatureAlgorithm / KMS adapter contract under-specified beyond Ed25519
  - §28 archgate/crate-standard/llvm-cov/mutants/miri commands not fully tasked
  - §33.4 full chain replacement detection vague
  - §33.6 external Attempted+terminal no Task ID
  - residual-open (T-RES-001) file not created
```

---

## 5. False-pass risks

```text
false_pass_risks:
  - T-ARCH-004/006 slash-abbreviation DONE after partial gate set
  - T-CI-002 "规划" used to check off §33.5 fuzz/miri/mutants/branch
  - T-CORE-020/021 DONE without implementing full XError matrix
  - T-OBS-001 DONE with empty metrics stub "see §30"
  - T-POL-001 skeleton mistaken for full §26 policy completeness
  - T-CP-005 "anchor 合同接口" mistaken for EVIDENCE-ANCHOR-001 production proof
  - Phantom T-ATOM makes §33.6 look mapped when no work item exists
  - "external … policy 预留" silent drop of T18-related system requirement
  - W7 T-V10-R0N could pass R-GATE-001 by seeing "EVIDENCE-*" string without ID inventory
  - Dual SSOT for golden paths (approval A10 open) → vector drift false PASS
  - P2 under W3 label → skip legacy bridge before domain cutover
  - Claiming plan 10x complete while R1+R2 already FAIL
```

---

## 6. Consistency with Round 1

| Item | R1 | R2 |
|------|----|----|
| Overall | FAIL | FAIL |
| Shared FAILs | R-SPEC-003, R-DEP-002, R-CANON-001, R-ATOM-001, R-ERR-001, R-GATE-001 | same |
| Added FAIL | — | **R-TEST-001**, **R-POL-001** |
| Honesty | PASS | PASS |
| Implementation complete? | No | No |

No contradiction: Round 2 is strictly harsher on test/policy depth for §24/§26.

---

## 7. Minimal remediation for Round 2 PASS (additive to Round 1)

1. Attach **error-matrix.md**: 24 variants + §18.2 map; bind T-CORE-020/021 AC to that file.
2. Attach **gates.md**: all 23 EVIDENCE-* IDs, one AC each under T-ARCH-*.
3. Attach **metrics.md**: 11 names + cardinality ban; bind T-OBS-001.
4. Expand T-CORE-026 AC with **named** §24.1 vector list; T-CORE-014 with 25-step table.
5. Split T-CI-002 into real tasks: T-FUZZ-001, T-MIRI-001, T-MUT-001 (with §24.10 kill-list), T-COV-BRANCH-001 — or formally **DEFER** with residual IDs (not PASS).
6. Replace phantom `T-ATOM via design` → `T-ATOM-001`; add `T-EXT-001` or formal DEFER record for external Attempted+terminal.
7. Expand T-POL-001/002 AC to full §26 field set + retention dimensions.
8. Add T-BP-001 backpressure (§29.3) and T-CP-007 hard-deadline write-stop (§16.5).
9. Re-run plan 10x only after above land; keep fail_rounds honest.

---

## 8. Round 2 verdict

```text
result: FAIL
failed_checks:
  - R-SPEC-003
  - R-DEP-002
  - R-CANON-001
  - R-ATOM-001
  - R-ERR-001
  - R-GATE-001
  - R-TEST-001
  - R-POL-001
fail_rounds_contribution: 1
cross_link_tasks_section_33: FAIL
adversarial_posture: prefer FAIL — gate ID holes, error matrix hole, phantom §33 tasks, test/policy under-specification
do_not_claim: fail_rounds=0 | plan complete | Spec Approved | §33 closed
```
