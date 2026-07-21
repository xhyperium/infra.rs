# Round 01 Findings — Evidence Plan Completeness (Verifier)

```text
round: 1
result: FAIL
scope: systematic top-down SPEC-EVIDENCE-002 §0–§17 + full R-* checklist
baseline: main@007ca7b5
sources_checked:
  - .agents/ssot/tools/evidence/xhyper-evidence-complete-spec.md
  - .agents/ssot/tools/evidence/plan/plan.md
  - .agents/ssot/tools/evidence/plan/gap-matrix.md
  - .agents/ssot/tools/evidence/plan/tasks.md
  - .agents/ssot/tools/evidence/plan/approval-packet.md
  - .worktree/evidence-todo.md
  - tools/evidence (spot-check code bugs)
  - crates/domain/macro, crates/infra/gate (downstream)
failed_checks:
  - R-SPEC-003
  - R-DEP-002
  - R-CANON-001
  - R-ATOM-001
  - R-ERR-001
  - R-GATE-001
omissions: [see §Omissions below]
false_pass_risks: [see §False-pass risks below]
notes: |
  Plan pack is a strong scaffold (honest Proposed status, DEF/T coverage,
  Wave DAG, path layout, INFRA-003 boundary). It is NOT yet complete enough
  for fail_rounds=0: several checklist ACs rely on bare "§N" references
  without embedding mandatory enumerations (25-step preimage, full forbid list,
  EvidenceError×XError map, full EVIDENCE-* gate IDs, §15 atomic modes).
  Prefer FAIL over soft-pass. Implementation code not claimed complete.
```

---

## 1. Full R-* checklist evaluation (Round 1)

| Check ID | Result | Evidence / reason |
|----------|--------|-------------------|
| R-SPEC-001 | **PASS** | Spec path + header `SPEC-EVIDENCE-002` present; plan/gap/todo cite same ID. |
| R-SPEC-002 | **PASS** | gap-matrix §1 has rows §0–§34 (35 rows incl. §0). |
| R-SPEC-003 | **FAIL** | tasks “§33 勾选→Task 映射” incomplete: (a) `external Attempted+terminal` = “订单域后续；policy 预留” — **no Task ID**; (b) Tier-A 原子性 cites phantom **`T-ATOM via design`** (no such task); (c) several 33.5 items only map to `T-CI-002` “规划/草案”, not executable close-out ACs. |
| R-GAP-001 | **PASS** | DEF-001…018 in gap-matrix §4 + evidence-todo §1. (`T-RES-001` residual file still TODO, but IDs are registered.) |
| R-GAP-002 | **PASS** | T1–T18 in gap-matrix §2 + todo §3 with Wave coverage. |
| R-PATH-001 | **PASS** | plan §5 `crates/evidence/`. |
| R-PATH-002 | **PASS** | plan §5 memory/file/postgres under `crates/adapters/evidence/`. |
| R-PATH-003 | **PASS** | plan §5 `tools/evidence-cli/`; W6/T-CUT-002 delete `tools/evidence`. |
| R-DEP-001 | **PASS** | plan §5 + T-CORE-002: kernel+sha2+thiserror. |
| R-DEP-002 | **FAIL** | Spec §4.2 forbids: anyhow, serde, serde_json, bincode, postcard, tokio, async-std, futures, tracing, log, chrono, time, uuid, rand, sqlx, reqwest (+ §4.1 workspace forbid list). Plan only repeats subset: “anyhow/serde/tokio/uuid/chrono”. tasks only “无 anyhow/serde/tokio”. **列表不完整**. |
| R-API-001 | **PASS** | T-CORE-004…007 Digest32/ChainId/EventId/OperationId/EvidenceName. |
| R-API-002 | **PASS** | T-CORE-009/010 Outcome 六态 + Draft; tags “0x00–0x05” named (mapping detail still omission). |
| R-API-003 | **PASS** | T-CORE-012/013 private fields + seal_record_v1. |
| R-CANON-001 | **FAIL** | Spec §11.2 enumerates **25 ordered preimage steps**. Plan/tasks only say “25 字段序；BE；presence” — **order not listed**, domain_tag / presence bytes / outcome_digest-except-Attempted not restated as AC. Golden task references §24.1 cases, not the 25-step order table. |
| R-CANON-002 | **PASS*** | T-CORE-017 “非全零”; gap DEF-004 + genesis formula in gap overview. *Tag string `XHYPER:EVIDENCE:GENESIS:V1\0` not in task AC (omission, not full fail). |
| R-CANON-003 | **PASS** | T-CORE-029 `("ab","c")≠("a","bc")`. |
| R-CANON-004 | **PASS** | T-CORE-018 digest_canonical; ban hash_bytes; DEF-003. |
| R-TIME-001 | **PASS*** | T-CORE-011 recorded_at/event_time. *Clock inject, fail-on-clock, non-zero, caller must not supply recorded_at — not in task AC. |
| R-CHAIN-001 | **PASS*** | T-CORE-019 ChainHead + ban 0+zero empty; sequence from 1 implied via MEM. *Sequence overflow freeze, ForkDetected isolation protocol (§12.4), multi-chain partition policy fields (§12.5) under-specified in tasks. |
| R-APPEND-001 | **PASS** | T-CORE-022 Durability/Append*; production Durable in plan/approval A6. |
| R-APPEND-002 | **PASS*** | T-MEM-002 seal+sequence+idempotent+CAS; T-DOM-005 fail-closed. *IdempotencyConflict vs same-content receipt, HeadConflict no-side-effect rules not named in AC. |
| R-READ-001 | **PASS*** | T-CORE-023 + T-MEM-003 head/get/range. *limit∈[1,10000], start≥1, no silent skip corrupt — not in AC text. |
| R-ATOM-001 | **FAIL** | Spec §15.1 requires A same-txn / B outbox / C SoT. tasks: T-PG-004 outbox only; §33.6 invents **`T-ATOM via design`**; §15.3 external Attempted+terminal deferred with no task; §15.4 pure-memory stage pattern **absent**; §15.5 Rejected-path only partially via T-DOM-005/006. |
| R-CP-001 | **PASS*** | T-CORE-024 + T-CP-001…006 Checkpoint/Signed/TailTruncated. *Checkpoint preimage byte layout not enumerated (see ROUND 2). |
| R-ERR-001 | **FAIL** | T-CORE-020 “§18.1 全 variant” + T-CORE-021 only “链损坏→Invariant”. Spec §18.1 has **24 named variants**; §18.2 full XError map (Invalid/Missing/Conflict/Unavailable/Invariant/Internal) **not restated**. Plan pack does not list variants or mapping table. |
| R-MEM-001 | **PASS** | T-MEM-004/007 production_allowed=false; ban pseudo-Durable. |
| R-FILE-001 | **PASS** | T-FILE-002…008 segment/fsync/recovery (by §20 ref). |
| R-PG-001 | **PASS** | T-PG-002 heads/records/outbox/checkpoints + uniqueness. |
| R-TEST-001 | **PASS*** | golden/property/fuzz/coverage/mutants/miri referenced across W1/W5/W6. *Heavy reliance on T-CI-002 “规划” for fuzz/miri/mutants/branch. |
| R-CLI-001 | **PASS** | T-CLI-002…005 cover verify/inspect/head/export/checkpoint/vectors/repair-tail. |
| R-POL-001 | **PASS*** | T-POL-001 skeleton schema_version+chain/operation. *§26 required-operation field list (13 fields) not fully in AC. |
| R-GATE-001 | **FAIL** | Spec §27 lists **23** EVIDENCE-* IDs. tasks T-ARCH-001…006 only cover subset + slash-abbreviations; **missing**: CANONICAL-001, DURABILITY-001, IDEMPOTENCY-001, CONCURRENCY-001, RECOVERY-001, FSYNC-001, ANCHOR-001, SCHEMA-001, VECTOR-001 (and abbreviated DOMAIN/DEBUG/JSON/GENESIS/PUBLIC not one-task-per-ID). |
| R-MIG-001 | **PASS** | gap §6 P0→P6; plan Waves; §6.1 no silent re-encode; T-LEG-001/002. (P2 bridge scheduled under W3 tasks — acceptable dependency-wise.) |
| R-EVID-001 | **PASS** | plan §8 full `evidence/system/<date>-…` tree matches §32. |
| R-DOWN-001 | **PASS** | T-DOM-001…006 + T-GATE-001…002; gap §3. Spot-check confirms current bugs: domain_macro `hash_bytes(format!("{point:?}"))`; gate `hash_bytes(name)`. |
| R-GOV-001 | **PASS** | approval-packet A1–A10; human-only Approved/stable. |
| R-FORBID-001 | **PASS** | plan header Forbidden consistent with migration/no fake PASS/no silent rehash. |
| R-TODO-001 | **PASS*** | todo covers Waves + DEF-001…018. *W1+ listed as ranges not per-task checkboxes (acceptable at planning stage). |
| R-CROSS-001 | **PASS** | plan §1.2 + gap-matrix §5 INFRA-003 orthogonal boundary. |
| R-HONEST-001 | **PASS** | Spec/plan/todo consistently **Proposed**; campaign ≠ Stable; §33 not closed; no false Approved. |

**Summary counts:** PASS 28 · PASS* (with residual risk) 9 counted in PASS · **FAIL 6**  
→ **round result = FAIL** (`failed_checks` non-empty).

---

## 2. Deep-scan omissions (§0–§17 focus)

### 2.1 §11.2 Record preimage — all 25 steps

Plan pack does **not** embed the ordered list. Spec requires:

| # | Field | In plan/tasks? |
|---|-------|----------------|
| 1 | domain_tag `XHYPER:EVIDENCE:RECORD:V1\0` | **NO** (only “25 字段序”) |
| 2 | schema_version u16_be = 1 | NO explicit |
| 3 | chain_id 32 | NO explicit |
| 4 | sequence u64_be | NO explicit |
| 5 | recorded_at_unix_nanos i64_be | NO explicit |
| 6 | event_id 32 | NO explicit |
| 7 | operation_id 32 | NO explicit |
| 8–9 | producer_len u16_be + producer | NO explicit |
| 10–12 | actor_namespace_len + namespace + actor_id_digest | NO explicit |
| 13 | subject_digest 32 | NO explicit |
| 14–15 | operation_len + operation | NO explicit |
| 16–17 | event_time_present + event_time i64_be | NO explicit |
| 18 | input_digest 32 | NO explicit |
| 19–20 | outcome_tag + outcome_digest (**except Attempted**) | tags range only |
| 21–22 | metadata_digest_present + digest | NO explicit |
| 23–24 | correlation_id_present + correlation_id | NO explicit |
| 25 | previous_digest 32 | NO explicit |

Also missing explicit AC for: all integers **big-endian**; presence only `0x00|0x01`; length-prefixed var fields; `record_digest = SHA256(preimage)`; persist `preimage||digest`.

### 2.2 Outcome tags 0x00–0x05

T-CORE-009 states “tag 0x00–0x05” but **does not map**:

```text
0x00 Attempted
0x01 Committed
0x02 Rejected
0x03 Failed
0x04 Cancelled
0x05 Compensated
```

### 2.3 §11.5 content digest preimage

Missing from tasks: domain tag `XHYPER:EVIDENCE:CONTENT:V1\0` + `u16_be(domain_len)||domain||u64_be(content_len)||bytes`; domain examples (`domain_macro.point.v1` …).

### 2.4 §11.7 Decoder contract

T-CORE-015 covers illegal presence/tag/trailing only. Missing AC: overlong name, invalid ASCII, truncated input, unsupported schema, **no panic on arbitrary input**, allocation caps.

### 2.5 §12 Chain

- §12.2 sequence overflow → reject + freeze chain: no task.
- §12.4 ForkDetected: stop writes, quarantine, security event, no automatic longest-chain: no task.
- §12.5 default ban global single chain + per-domain partition examples: only via policy skeleton, not AC.

### 2.6 §13 Append nuances

- Durability definitions (Volatile/Process/Durable semantics) not restated.
- Same event_id + same content → original receipt; different content → IdempotencyConflict + freeze: not explicit.
- HeadConflict: no append/no sequence/no partial: not explicit.
- Appender algorithm steps (validate→idempotency→Clock→CAS head→sequence→seal→persist→durability): compressed into one MEM task.

### 2.7 §15 Atomicity (major)

| Spec | Plan coverage |
|------|----------------|
| 15.1 A/B/C triad | Incomplete; phantom T-ATOM |
| 15.2 outbox rules | T-PG-004/005 partial |
| 15.3 external Attempted+terminal | **Deferred, no task** |
| 15.4 pure memory state machine | **ABSENT** |
| 15.5 Rejected only after evidence success | Partial T-DOM-005 |

### 2.8 §16–§17 (preview; full ROUND 2)

Checkpoint preimage layout, frequency hard-deadline write-stop, VerificationReport fields, §17.2 full detection list — mostly “§16/§17” references only.

### 2.9 §0–§5 structural

| Item | Coverage |
|------|----------|
| Tamper-evident wording ban | T-DOC-001 TODO (tracked) |
| Non-responsibilities §2.4 | Not restated in plan |
| Dev-deps allowlist §4.3 | Not in tasks |
| Features default=[] / no mock | T-CORE-034 PASS |
| Crate forbid unsafe/todo/unwrap/poison-empty | T-CORE-003/031 + T-MEM-006 |

### 2.10 Current code bugs (must stay in residual)

Verified on tree (not invented):

| DEF | Confirmed |
|-----|-----------|
| DEF-001 | package at `tools/evidence` |
| DEF-002 | `anyhow` in Cargo.toml + source |
| DEF-003 | `pub fn hash_bytes` |
| DEF-004 | empty chain zero digest / zero genesis usage |
| DEF-005 | public `EvidenceRecord` fields |
| DEF-006 | domain_macro Debug string hash |
| DEF-007 | mock `verify_chain` always Ok |
| DEF-009 | corrupt → Invalid-style path (via XError, not Invariant) |
| gate | `hash_bytes(name.as_bytes())` |

These are registered (PASS for R-GAP-001) but none closed — correct honesty.

### 2.11 INFRA-003

Boundary declared (PASS R-CROSS-001). No conflation with runtime audit chain in plan steps.

---

## 3. Omissions (machine-list)

```text
omissions:
  - §11.2 25-step preimage order not embedded in plan/tasks/todo/golden AC
  - §11.2 RECORD domain tag string absent from tasks
  - §11.2 outcome_digest omitted for Attempted not stated
  - Outcome tag→variant mapping 0x00..0x05 not listed (only range)
  - §11.5 CONTENT domain tag + length framing absent
  - §11.7 full decoder refusal/no-panic contract incomplete
  - §4.2 full external forbid list incomplete in plan/tasks
  - §4.1 workspace internal forbid list not copied to tasks
  - §12.2 sequence overflow freeze: no task
  - §12.4 ForkDetected operational protocol: no task
  - §12.5 chain partition strategy table: not tasked beyond policy skeleton
  - §13.5 dual idempotency outcomes not explicit in AC
  - §15.1 A/B/C triad no real Task (phantom T-ATOM)
  - §15.3 external Attempted+terminal: no Task ID
  - §15.4 pure-memory append-then-swap pattern: ABSENT
  - §10 Clock failure / non-zero recorded_at / caller ban: incomplete AC
  - §14 read_range limit bounds + no silent skip: incomplete AC
  - §17.2 full verifier detection checklist not restated
  - §2.4 non-responsibilities not in plan body
  - residual-open file (T-RES-001) not yet created
```

---

## 4. False-pass risks

```text
false_pass_risks:
  - T-PLAN-003 marked DONE while §33 mapping has phantom T-ATOM and deferred external path
  - T-CORE-014 AC "25 字段序" can be checked as "count=25" without correct order unless golden freezes bytes
  - T-CORE-020 "§18.1 全 variant" DONE by reading enum file without plan-level variant inventory
  - T-CORE-021 only asserts 链损坏→Invariant; other XError rows could remain wrong
  - Slash-abbreviated T-ARCH-004/006 can be marked DONE after implementing 1 of N gates
  - T-CI-002 "规划" could fake-close §33.5 fuzz/miri/mutants/branch
  - "READY TO EXECUTE W0–W1" misread as Spec Approved / production-ready
  - Dual-package coexistence without hard freeze enforcement (T-FREEZE-001 still TODO)
  - Golden path dual SSOT (crate tests vs tests/vectors/evidence-v1) still open (approval A10)
```

---

## 5. What is solid (do not rework needlessly)

- Honest status: Proposed / incubating / §33 open.
- DEF-001…018 + T1–T18 registered.
- Target tree layout matches §3.
- Migration: no silent V1 rehash of legacy six-field chain.
- INFRA-003 orthogonality.
- Downstream domain_macro + gate migration tasks exist.
- Human gates in approval-packet clear.
- Ten-round protocol defined; this round correctly cannot claim fail_rounds=0.

---

## 6. Minimal remediation to re-attempt Round 1 PASS

1. **Embed** §11.2 25-step table + outcome tag map + BE/presence rules into tasks (or `plan/canonical-v1-checklist.md` linked from T-CORE-014/026).
2. Expand R-DEP-002 forbid list to full §4.1+§4.2.
3. Replace phantom `T-ATOM` with real `T-ATOM-001` (document A/B/C proof per Tier-A) + `T-EXT-001` for Attempted+terminal policy/tests (even if domain_order DEFER with formal DEFER record).
4. List EvidenceError 24 variants + full §18.2 XError table under T-CORE-020/021.
5. Expand T-ARCH-* to **one row per EVIDENCE-* ID** (or explicit checklist attachment).
6. Do **not** mark T-V10-PLAN DONE until fail_rounds=0 across 10 rounds.

---

## 7. Round 1 verdict

```text
result: FAIL
failed_checks: [R-SPEC-003, R-DEP-002, R-CANON-001, R-ATOM-001, R-ERR-001, R-GATE-001]
fail_rounds_contribution: 1
adversarial_posture: prefer FAIL — real plan-pack omissions, not nitpicks
code_implementation_status: not evaluated as complete (correctly still prototype)
```
