# Spec Inventory — 防遗漏附录（SPEC-EVIDENCE-002）

| 字段 | 值 |
|------|-----|
| Inventory ID | `INV-EVIDENCE-002-v1` |
| 用途 | 将规范中不可缩写的枚举项显式落盘，供 tasks/gates/tests 引用 |
| 配套 | plan.md · tasks.md · gap-matrix.md |

> 本文件是计划包的一部分。实现 Task 的 AC 可引用此处 ID，禁止仅写「见 §xx 全覆盖」而无枚举。

---

## I-1. Canonical V1 Record preimage 字段序（§11.2）— 25 步

| Step | 字段 | 编码 |
|------|------|------|
| 1 | domain_tag | fixed ASCII `"XHYPER:EVIDENCE:RECORD:V1\0"` |
| 2 | schema_version | u16_be = 1 |
| 3 | chain_id | 32 bytes |
| 4 | sequence | u64_be |
| 5 | recorded_at_unix_nanos | i64_be |
| 6 | event_id | 32 bytes |
| 7 | operation_id | 32 bytes |
| 8 | producer_len | u16_be |
| 9 | producer | producer_len bytes |
| 10 | actor_namespace_len | u16_be |
| 11 | actor_namespace | actor_namespace_len bytes |
| 12 | actor_id_digest | 32 bytes |
| 13 | subject_digest | 32 bytes |
| 14 | operation_len | u16_be |
| 15 | operation | operation_len bytes |
| 16 | event_time_present | u8 (0x00/0x01) |
| 17 | event_time_unix_nanos | i64_be when present |
| 18 | input_digest | 32 bytes |
| 19 | outcome_tag | u8 |
| 20 | outcome_digest | 32 bytes except Attempted |
| 21 | metadata_digest_present | u8 |
| 22 | metadata_digest | 32 bytes when present |
| 23 | correlation_id_present | u8 |
| 24 | correlation_id | 32 bytes when present |
| 25 | previous_digest | 32 bytes |

`record_digest = SHA256(steps 1..25)`；持久化 `preimage || record_digest`。

**Task**: `T-CORE-014` AC 必须逐 step 断言顺序与宽度。

---

## I-2. Outcome tags（§11.2）

| Tag | Variant | outcome_digest |
|-----|---------|----------------|
| 0x00 | Attempted | 无 |
| 0x01 | Committed | output_digest |
| 0x02 | Rejected | reason_digest |
| 0x03 | Failed | error_digest |
| 0x04 | Cancelled | reason_digest |
| 0x05 | Compensated | result_digest |

**Task**: `T-CORE-009` + golden per tag.

---

## I-3. Domain tags

| 用途 | Tag 字符串 |
|------|------------|
| Record | `XHYPER:EVIDENCE:RECORD:V1\0` |
| Genesis | `XHYPER:EVIDENCE:GENESIS:V1\0` |
| Content | `XHYPER:EVIDENCE:CONTENT:V1\0` |
| ChainId derive | `XHYPER:EVIDENCE:CHAIN-ID:V1\0` |
| Checkpoint | `XHYPER:EVIDENCE:CHECKPOINT:V1\0` |

---

## I-4. EvidenceError 全集（§18.1）— 24 variants

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

### XError 映射（§18.2）

| XError | EvidenceError |
|--------|---------------|
| Invalid | InvalidName, InvalidDraft, InvalidEncoding |
| Missing | MissingChain, MissingRecord |
| Conflict | IdempotencyConflict, HeadConflict, DuplicateSequence, DuplicateEventId |
| Unavailable | StorageUnavailable, DurabilityFailure, ClockUnavailable, UnsupportedVersion |
| Invariant | SequenceOverflow, SequenceGap, ChainIdMismatch, PreviousDigestMismatch, RecordDigestMismatch, ForkDetected, CheckpointMismatch, SignatureInvalid, TailTruncated, CorruptStorage |
| Internal | SynchronizationFailure |

**禁止**：链损坏 → Invalid。

**Task**: `T-CORE-020`, `T-CORE-021`（逐 variant 单测映射）。

---

## I-5. Durability（§13.1）

| 值 | 语义 | 生产 required 默认 |
|----|------|-------------------|
| Volatile | 仅内存 | 否；memory 最高 |
| Process | OS page cache，未承诺掉电 | 否 |
| Durable | 事务提交或 fsync 后返回 | **是** |

**Task**: `T-CORE-022` 三态枚举；`T-MEM-004` 拒 Durable；`T-FILE-004`/`T-PG-003` 实现 Durable；`T-BOOT-001` 生产禁 Volatile/Process for required。

---

## I-6. Golden vectors 名录（§24.1）

| Vector ID | 内容 |
|-----------|------|
| GV-empty-genesis | empty/genesis |
| GV-attempted | Attempted |
| GV-committed | Committed |
| GV-rejected | Rejected |
| GV-failed | Failed |
| GV-cancelled | Cancelled |
| GV-compensated | Compensated |
| GV-opt-all | all optional fields present |
| GV-opt-none | all optional fields absent |
| GV-max-name | max name length |
| GV-event-time-pre-epoch | event_time before epoch |
| GV-ts-boundaries | timestamp boundaries |
| GV-multi-chain | multi-record chain |
| GV-checkpoint | checkpoint |

每 vector 四元组：`semantic input` · `canonical hex` · `record digest` · `expected chain head`。

路径 SSOT（A10 决议前默认）：`crates/evidence/tests/vectors/evidence-v1/` + CLI 只读引用。

**Task**: `T-CORE-026` 必须创建上表 14 项；`T-CLI-004` 复算。

---

## I-7. Mutation kill list（§24.10）— 不得存活

```text
M-KILL-01 字段顺序变化
M-KILL-02 长度前缀删除
M-KILL-03 字节序反转
M-KILL-04 domain tag 删除
M-KILL-05 previous digest 不参与 hash
M-KILL-06 sequence 不参与 hash
M-KILL-07 event_id 幂等检查删除
M-KILL-08 verify 比较反转
M-KILL-09 checkpoint 对比删除
M-KILL-10 错误映射降级（Invariant→Invalid）
```

**Task**: `T-MUT-001`（从 T-CI-002 拆出）。

---

## I-8. Fuzz 目标（§24.8）

| ID | 目标 |
|----|------|
| FUZZ-01 | canonical decoder |
| FUZZ-02 | record verifier |
| FUZZ-03 | segment parser |
| FUZZ-04 | checkpoint parser |
| FUZZ-05 | CLI import |

**Task**: `T-FUZZ-001`…`T-FUZZ-005` 或 `T-FUZZ-001` 覆盖表 + corpus。

---

## I-9. EVIDENCE-* 门禁全表（§27）— 23 IDs

### Core (10)

| ID | 规则摘要 | Task |
|----|----------|------|
| EVIDENCE-PATH-001 | runtime 在 tools/ → fail | T-ARCH-001 |
| EVIDENCE-DEP-001 | core 内依仅 kernel | T-ARCH-002 |
| EVIDENCE-DEP-002 | 外依仅 sha2,thiserror | T-ARCH-002 |
| EVIDENCE-ANYHOW-001 | 无 anyhow | T-ARCH-003 |
| EVIDENCE-CANONICAL-001 | 必须 canonical V1 | T-ARCH-010 |
| EVIDENCE-DOMAIN-001 | 禁无 domain hash_bytes | T-ARCH-004 |
| EVIDENCE-DEBUG-HASH-001 | 禁 Debug/Display 进 digest | T-ARCH-004 |
| EVIDENCE-JSON-HASH-001 | 禁裸 JSON digest | T-ARCH-004 |
| EVIDENCE-GENESIS-001 | 禁全零 genesis | T-ARCH-004 |
| EVIDENCE-PUBLIC-001 | 禁公开可写 record 字段 | T-ARCH-004 |

### Adapter (6)

| ID | 规则摘要 | Task |
|----|----------|------|
| EVIDENCE-DURABILITY-001 | required 链禁 Volatile/Process | T-ARCH-011 |
| EVIDENCE-MEMORY-PROD-001 | release 图无 evidence_memory | T-ARCH-005 |
| EVIDENCE-IDEMPOTENCY-001 | 必须过 event_id conformance | T-ARCH-012 |
| EVIDENCE-CONCURRENCY-001 | 必须过 concurrent suite | T-ARCH-013 |
| EVIDENCE-RECOVERY-001 | 生产 adapter 有 crash recovery evidence | T-ARCH-014 |
| EVIDENCE-FSYNC-001 | file Durable 必须有 fsync 合同 | T-ARCH-015 |

### System (7)

| ID | 规则摘要 | Task |
|----|----------|------|
| EVIDENCE-POLICY-001 | required op 未登记 → fail | T-ARCH-006 |
| EVIDENCE-COVERAGE-001 | required op 无成功/失败测 → fail | T-ARCH-006 |
| EVIDENCE-ATOMICITY-001 | Tier-A 无事务/outbox/SoT 证明 → fail | T-ARCH-016 |
| EVIDENCE-CHECKPOINT-001 | production chain 无 checkpoint policy → fail | T-ARCH-006 |
| EVIDENCE-ANCHOR-001 | signed checkpoint 无独立 anchor → fail | T-ARCH-017 |
| EVIDENCE-SCHEMA-001 | schema 变化未新建版本 → fail | T-ARCH-018 |
| EVIDENCE-VECTOR-001 | golden 漂移无 RFC → fail | T-ARCH-019 |

---

## I-10. CLI（§25）

### 命令

| 命令 | Task |
|------|------|
| verify | T-CLI-002 |
| inspect | T-CLI-002 |
| head | T-CLI-002 |
| export | T-CLI-002 |
| checkpoint verify | T-CLI-003 |
| vectors verify | T-CLI-004 |
| vectors generate | T-CLI-007（新增） |
| repair-tail | T-CLI-005 |

### 退出码

| Code | 含义 |
|------|------|
| 0 | valid / success |
| 2 | invalid arguments |
| 3 | chain invalid |
| 4 | checkpoint/signature invalid |
| 5 | storage unavailable |
| 6 | unsupported version |
| 7 | repair required |

### 默认行为 AC（T-CLI-002）

```text
- 只读默认
- human + --json（JSON 非 canonical）
- 错误 stderr
- 不输出敏感信息
- 支持 chain + sequence range
```

### repair-tail AC（T-CLI-005）

```text
- 仅未提交无 commit marker 尾帧
- 只读验证 → repair plan → 显式确认 → 备份 → repair evidence
- 不得跨越可信 checkpoint
```

---

## I-11. Metrics（§30）— adapter/bootstrap 暴露

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

禁止高基数敏感 label。**Task**: `T-OBS-001` AC = 上表 11 名全注册清单。

---

## I-12. Core 禁止外部依赖（§4.2）

```text
anyhow, serde, serde_json, bincode, postcard,
tokio, async-std, futures, tracing, log,
chrono, time, uuid, rand, sqlx, reqwest
```

允许：`kernel`, `sha2`, `thiserror`。  
Dev 允许：`proptest`, `trybuild`, `static_assertions`, cargo-fuzz 相关。

---

## I-13. evidence-policy required operation 字段（§26）

```text
producer, operation, subject strategy, chain strategy, actor strategy,
input canonical domain, output/error canonical domain,
atomicity, durability, checkpoint policy, retention, owner
```

Chain 登记：`namespace, subject derivation, expected write rate, writer ownership, retention, checkpoint frequency, criticality`。

**Task**: `T-POL-002` schema 校验上述键。

---

## I-14. P0–P6 ↔ Wave 映射

| Spec 阶段 | Wave | 主 Task 前缀 |
|-----------|------|--------------|
| P0 冻结错误扩散 | W0 | T-FREEZE / T-DOC |
| P1 Core V1 | W1 | T-CORE |
| P2 Compatibility bridge | W3（依赖 W1） | T-LEG |
| P3 Domain migration | W3 | T-DOM / T-GATE |
| P4 Durable adapters | W4 | T-FILE / T-PG |
| P5 Checkpoint | W5 | T-CP / T-CLI |
| P6 Cutover | W6 | T-CUT / T-ARCH / T-BOOT |
| 十轮验收 | W7 | T-V10 |
| 人审/版本 | W8–W9 | T-HUM / T-33 |

---

## I-15. §15 原子性模式 → Task

| 模式 | 说明 | Task |
|------|------|------|
| A | 业务+evidence 同事务 | T-ATOM-001, T-PG-003 |
| B | 业务+outbox 同事务 | T-ATOM-002, T-PG-004/005 |
| C | evidence log 即 SoT | T-ATOM-003 |
| External | Attempted 前 durable + terminal 结果 | T-ATOM-004（订单/交易所域；可先 DEFER 但必须登记） |
| Memory-only | compute→Committed evidence→append→swap | T-ATOM-005 |
| Rejected path | Rejected outcome + fail-closed | T-ATOM-006, T-DOM-005 |

---

## I-16. §22 保留与隐私 → Task

| 项 | Task |
|----|------|
| record 仅摘要 | T-CORE-012 模型保证 |
| artifact store content-addressed | T-PRIV-001 |
| retention 六类 | T-PRIV-002 |
| 删除/erasure 不改历史 | T-PRIV-003 |
| 禁物理删 committed（保留期内） | T-PRIV-003 |

六类 retention：`record, checkpoint, signing public key, canonical schema, source artifact, verifier binary/source`。

---

## I-17. File recovery 10 步（§20.5）— T-FILE-005 AC

```text
1 writer lock → 2 可信 checkpoint → 3 扫 segment → 4 验 header
→ 5 验完整 frame → 6 不完整尾帧 → 7 sequence/digest
→ 8 对照 checkpoint → 9 建 head → 10 才允许写
```

Frame：`u32_be(len) || canonical_record_bytes || u32_be(len) || commit_marker`；前后 len 必须相同。

---

## I-18. 双包共存命名（致命同名修复）

| 阶段 | package name | path |
|------|--------------|------|
| 迁移期旧 | `evidence_legacy`（rename from evidence）或 feature 隔离 | `tools/evidence` 临时 |
| 新 core | `evidence` | `crates/evidence` |
| 切后 | 仅 `evidence` | `crates/evidence` |

**Task**: `T-LEG-003` rename/隔离策略落地，禁止两个 `name = "evidence"` 同时在 workspace。

---

## I-19. Bootstrap 生产强制

| 项 | Task |
|----|------|
| 生产 feature 图禁止 evidence_memory | T-BOOT-001 / T-ARCH-005 |
| required chain 默认 Durable | T-BOOT-001 / T-ARCH-011 |
| 未达 checkpoint policy hard deadline 阻写 | T-BOOT-002 / T-CP-003 |

---

## I-20. CI 命令清单（§28）→ Task

| 命令 | Task |
|------|------|
| cargo fmt -- --check | T-CI-001 |
| cargo clippy -p evidence --all-targets -- -D warnings | T-CI-001 |
| cargo test -p evidence | T-CI-001 |
| cargo llvm-cov -p evidence --fail-under-lines 95 | T-CORE-033 / T-CI-003 |
| cargo mutants -p evidence | T-MUT-001 |
| cargo miri test -p evidence | T-MIRI-001 |
| cargo run -p archgate -- --json | T-ARCH-* / T-CI-001 |
| cargo run -p xtask -- lint-deps | T-CI-001 |
| cargo run -p xtask -- crate-standard --check | T-CI-004 |
| cargo test -p evidence_memory | T-CI-001 |
| cargo test -p evidence_file | T-CI-001 |
| cargo test -p evidence_file --test crash_recovery | T-FILE-008 |
| cargo test -p evidence_postgres --test concurrency | T-PG-006 |
| cargo run -p evidence-cli -- vectors verify | T-CLI-004 |
| cargo run -p evidence-cli -- verify \<fixture\> | T-CLI-002 |

Nightly：full mutation · full fuzz · Miri · adapter chaos · key rotation · historical schema → `T-CI-NIGHTLY-001`（正式任务，非「草案」；未接线前状态 BLOCKED 不算 PASS）。

---

## I-21. §23 Schema 演进

| 项 | Task |
|----|------|
| V1 冻结规则 | T-CORE-026 后文档声明 T-SCH-001 |
| Reader 多版本 | T-SCH-002 |
| 算法迁移须 V2/新 tag | T-SCH-003 |
| 迁移 checkpoint 双锚定 | T-SCH-003 / T-CP |

---

## I-22. §29.3 背压

```text
durable 跟不上 → required fail-closed
→ StorageUnavailable / DurabilityFailure
→ 不无限内存缓存、不丢旧 evidence、不静默 Volatile
```

**Task**: `T-BP-001`。

---

## I-23. 信任边界组件（§1.3）

kernel · evidence core · evidence adapters · checkpoint signer · external anchor · domain/service · bootstrap

---

## I-24. 非职责清单（§2.4）— 不得实现进 core

```text
tracing/日志/指标；业务状态机；订单/资金规则；自动生成 operation 名；
保存敏感 payload；密钥托管；KMS/Vault 具体实现；网络传输；业务事务编排；
任意 JSON 作 canonical；替代源数据备份；用摘要证明调用方未说谎
```

---

## I-25. ADR / 规范冲突登记

| 冲突 | 处理 Task |
|------|-----------|
| ADR-010 六字段+mock feature | T-DOC-004 |
| 旧 evidence-spec.md | T-DOC-002 T-HUM-002 |
| architecture/spec.md 路径 tools/evidence | T-DOC-003 T-CUT-003 |
| ADR-012 / auditx / 路径投影（若保留 tools/evidence 叙述） | T-DOC-005 显式对账 |
| Article IX mock-in-crate vs 002 禁 core mock | A2 in approval-packet |

---

## I-26. Forbidden 统一清单（三处必须一致）

```text
1. 假 §33 Done / registry stable / 无 Evidence 勾 PASS
2. 旧链静默 rehash 声称 V1 连续
3. core 引入 anyhow/serde/tokio/uuid/chrono
4. mock / evidence_memory 进生产
5. 通用 hash_bytes / Debug→digest
6. 链损坏映射 Invalid
7. SKIP 计 PASS / 手写 digest
8. 私钥进 core/仓库
9. AI 独断 Spec Approved
10. 两个 package 同名 evidence 并存无隔离策略
```
