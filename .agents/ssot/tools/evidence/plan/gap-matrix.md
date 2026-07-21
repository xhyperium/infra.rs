# Gap Matrix — SPEC-EVIDENCE-002 vs 现状

| 字段 | 值 |
|------|-----|
| Matrix ID | `GAP-EVIDENCE-002-v1` |
| Source Spec | `.agents/ssot/tools/evidence/xhyper-evidence-complete-spec.md` |
| Spec ID | `SPEC-EVIDENCE-002` · Status **Proposed** |
| Baseline | `main@007ca7b5`（2026-07-14） |
| Current package | `evidence` @ `tools/evidence` · **0.1.0** |
| Target package | `evidence` @ `crates/evidence` · **0.1.1**（Stable 后） |
| Supersedes | `evidence-spec.md`；ADR-010 中 evidence 部分需修订 |

> 状态枚举：`ABSENT`（无实现）· `WRONG`（存在但与 002 冲突）· `PARTIAL`（部分满足）· `PASS`（已闭合）· `N/A`（本章无代码）· `GOVERNANCE`（需人审/ADR）

---

## 0. 总览：当前是什么 vs 目标

| 维度 | 当前（tools/evidence 0.1.0） | 目标（SPEC-EVIDENCE-002） |
|------|------------------------------|---------------------------|
| 定位措辞 | README「不可篡改」 | **tamper-evident only**；无签名检查点时禁「不可篡改」 |
| 路径 | `tools/evidence` | `crates/evidence` + `crates/adapters/evidence/*` + `tools/evidence-cli` |
| 记录模型 | 六字段公开 `EvidenceRecord` | 私有字段 `EvidenceRecordV1` + `EvidenceDraft` |
| 哈希 | `hash_bytes` 无 domain；`prev\|\|ts_le\|\|module\|\|op\|\|in\|\|out` | canonical V1 + domain-separated SHA-256 |
| Genesis | 全零 `[0;32]` | `SHA256("XHYPER:EVIDENCE:GENESIS:V1\0" \|\| chain_id)` |
| 链 | 单全局内存 Vec；无 chain_id/sequence | multi-chain + sequence 连续 + ChainHead |
| 幂等 | 无 | event_id 幂等 + IdempotencyConflict |
| 时间 | 单一 `ts` | `recorded_at`（Clock）+ 可选 `event_time` |
| Actor/Subject | 无 | EvidenceActor + subject_digest |
| Outcome | 无（input/output hash 对） | typed EvidenceOutcome |
| 持久化 | 仅 InMemory | memory（测）/ file（fsync）/ postgres+outbox |
| Checkpoint | 无 | CheckpointV1 + Signed + independent anchor |
| 依赖 | kernel + **sha2 + anyhow** | kernel + **sha2 + thiserror only** |
| mock | feature mock；verify 永远成功 | 禁止 core mock feature；memory 不得称 Durable |
| 错误映射 | chain broken → Invalid | 链损坏 → Invariant；fail-closed |
| 调用方 | domain_macro Debug 字符串 hash；gate hash name | 必须 canonical domain digest |

---

## 1. 章节级差距矩阵（§0–§34）

| 章 | 主题 | 代码 | 文档/政策 | 门禁 | 综合 | 关闭 Wave | 备注 |
|----|------|------|-----------|------|------|-----------|------|
| §0 | 文档定位 | N/A | PARTIAL | ABSENT | PARTIAL | W0/W6 | Spec Proposed；未 Approved；未 supersede 旧 spec |
| §1 | 安全模型 | WRONG | PARTIAL | ABSENT | WRONG | W0–W5 | 无威胁覆盖 T1–T18 证明；措辞「不可篡改」 |
| §2 | 职责边界 | WRONG | PARTIAL | ABSENT | WRONG | W1/W4 | runtime 在 tools/；memory 与 core 混放 |
| §3 | 目录结构 | ABSENT | ABSENT | ABSENT | ABSENT | W1/W4/W5 | 无 crates/evidence；无 adapters/evidence；无 evidence-cli |
| §4 | 依赖合同 | WRONG | PARTIAL | PARTIAL | WRONG | W1 | 有 anyhow；无 thiserror；依赖路径 tools |
| §5 | Crate 规则 | PARTIAL | PARTIAL | ABSENT | WRONG | W1 | 无 forbid/deny 全套；有 production unwrap 风险；公开可变字段 |
| §6 | 值对象 | ABSENT | ABSENT | ABSENT | ABSENT | W1 | 无 Digest32/ChainId/EventId/OperationId/EvidenceName |
| §7 | Actor/Subject | ABSENT | ABSENT | ABSENT | ABSENT | W1 | 无 |
| §8 | EvidenceDraft | ABSENT | ABSENT | ABSENT | ABSENT | W1 | 无 |
| §9 | EvidenceRecordV1 | WRONG | WRONG | ABSENT | WRONG | W1 | 六字段公开结构 ≠ V1 |
| §10 | 时间语义 | WRONG | WRONG | ABSENT | WRONG | W1 | 单一 ts |
| §11 | Canonical V1 | ABSENT | ABSENT | ABSENT | ABSENT | W1 | 无 preimage；无 domain tag；LE 混用 |
| §12 | Chain 语义 | ABSENT | ABSENT | ABSENT | ABSENT | W1/W2 | 无 ChainHead/sequence/fork 检测 |
| §13 | Append 合同 | WRONG | ABSENT | ABSENT | WRONG | W2 | EvidenceSink::record ≠ AppendRequest/Receipt/Durability |
| §14 | Reader 合同 | PARTIAL | ABSENT | ABSENT | WRONG | W2 | 仅 records() 克隆；无 head/get_by_event_id/read_range 合同 |
| §15 | 原子性 | ABSENT | ABSENT | ABSENT | ABSENT | W3/W4 | T-ATOM-001…006；模式 A/B/C/External/Memory/Rejected |
| §16 | Checkpoint | ABSENT | ABSENT | ABSENT | ABSENT | W5 | 全缺 |
| §17 | 纯验证 | PARTIAL | ABSENT | ABSENT | WRONG | W1/W2 | verify_chain 弱；映射 Invalid；无 VerificationReport |
| §18 | EvidenceError | ABSENT | ABSENT | ABSENT | ABSENT | W1 | 直接 XError；无专用错误枚举/映射表 |
| §19 | Memory adapter | WRONG | ABSENT | ABSENT | WRONG | W2 | 在 core 内；可声称内存即生产；lock poison 静默空 |
| §20 | File adapter | ABSENT | ABSENT | ABSENT | ABSENT | W4 | 全缺 |
| §21 | Postgres/outbox | ABSENT | ABSENT | ABSENT | ABSENT | W4 | 全缺 |
| §22 | 保留/隐私 | ABSENT | ABSENT | ABSENT | ABSENT | W3/W5 | T-PRIV-001…003；I-16 六类 retention |
| §23 | Schema 演进 | ABSENT | ABSENT | ABSENT | ABSENT | W1/W5 | T-SCH-001…003；V1 未冻结；无 golden |
| §24 | 测试合同 | PARTIAL | ABSENT | ABSENT | WRONG | W1–W5 | 有少量 unit；无 golden/fuzz/mutants/conformance |
| §25 | CLI | ABSENT | ABSENT | ABSENT | ABSENT | W5 | 无 evidence-cli |
| §26 | 政策清单 | ABSENT | ABSENT | ABSENT | ABSENT | W0/W3 | 无 `.architecture/evidence-policy.toml` |
| §27 | 机器门禁 | ABSENT | ABSENT | ABSENT | ABSENT | W6 | 无 EVIDENCE-* rules |
| §28 | CI 命令 | PARTIAL | PARTIAL | PARTIAL | WRONG | W6 | workspace CI 含 evidence 测试；无 002 专用门禁 |
| §29 | 性能 | ABSENT | ABSENT | ABSENT | ABSENT | W5 | 无 benchmark 合同 |
| §30 | Observability | ABSENT | ABSENT | ABSENT | ABSENT | W5 | core 不应依赖 observex；adapter 指标缺 |
| §31 | 迁移计划 | PARTIAL | PARTIAL | ABSENT | PARTIAL | W0–W7 | 002 已写 P0–P6；代码未开迁 |
| §32 | 自身 Evidence | ABSENT | ABSENT | ABSENT | ABSENT | 每波 | 无 evidence/system/<date>-… 包 |
| §33 | 完成定义 | ABSENT | ABSENT | ABSENT | ABSENT | W8 | 全部未勾 |
| §34 | 最终裁定 | N/A | PASS* | N/A | N/A | 持续 | *仅文档存在；代码未满足 |

---

## 2. 威胁模型 T1–T18 覆盖现状

| ID | 威胁 | 当前防御 | 目标防御 | Gap |
|----|------|----------|----------|-----|
| T1 | 改历史字段 | 弱哈希链 unit | canonical + digest 校验 | OPEN |
| T2 | 删中间 | unit 有 | sequence + previous | PARTIAL→需 V1 |
| T3 | 重排 | 弱 | sequence 连续 | OPEN |
| T4 | 重复插入 | 无 | event_id 幂等 | OPEN |
| T5 | 链尾截断 | 无 | signed checkpoint | OPEN |
| T6 | 整链替换 | 无 | independent anchor | OPEN |
| T7 | 并发分叉 | 单锁弱 | CAS + ForkDetected | OPEN |
| T8 | 重试重复审计 | 无 | event_id 幂等 | OPEN |
| T9 | 崩溃半条 | N/A 内存 | file frame+commit marker | OPEN |
| T10 | 短写/fsync 失败 | 无 | Durable 合同 | OPEN |
| T11 | 编码不一致 | 有 | golden V1 | OPEN |
| T12 | Debug/JSON 摘要 | **ACTIVE BUG** domain_macro Debug hash | 禁 + domain digest | OPEN |
| T13 | 伪造 actor | 无 actor | id_digest + 签名边界 | OPEN |
| T14 | 敏感原文 | 字段可塞 String | 仅 digest+受控名 | OPEN |
| T15 | mock 进生产 | mock feature 在 core | 阻断 memory 生产图 | OPEN |
| T16 | 旧 verifier | 无版本 | schema retention | OPEN |
| T17 | 时钟混淆 | 单一 ts | recorded_at/event_time | OPEN |
| T18 | 写失败仍业务成功 | 未 fail-closed 裁定 | required + fail-closed | OPEN |

---

## 3. 调用方 / 下游影响面

| 消费者 | 路径 | 当前用法 | 迁移动作 | Wave |
|--------|------|----------|----------|------|
| domain_macro | `crates/domain/macro` | `hash_bytes(format!("{point:?}"))`；`EvidenceSink` | Debug→canonical domain；typed outcome；EventId | W3 |
| domain_macro tests/fuzz | 同上 | InMemoryEvidenceSink | 改 memory adapter | W3 |
| gate | `crates/infra/gate` | EvidenceSink + hash_bytes(name) | Append 或桥接；canonical name digest | W3 |
| workspace member | `Cargo.toml` | `tools/evidence` | 改为 `crates/evidence` | W1 |
| lint-deps / R1 | `docs/architecture/spec.md` | L0 path tools/evidence | 改路径；R1 仍允许 domain→evidence | W0/W6 |
| ADR-010 | Proposed | 六字段+mock feature | 修订为 002 模型 | W0 GOVERNANCE |
| evidence-spec.md | 旧 | EvidenceSink 最小面 | Superseded by 002 | W0/W6 |
| INFRA-003 | 基础设施 evidence 协议 | 工作包验证协议 | **正交**：勿混；交叉引用即可 | 持续 |

---

## 4. 已知现有缺陷（必须进 residual，禁止粉饰）

| ID | 缺陷 | 规范引用 | 严重度 |
|----|------|----------|--------|
| DEF-001 | runtime 在 tools/ | §0, §3, EVIDENCE-PATH-001 | P0 |
| DEF-002 | 依赖 anyhow | §4.2, EVIDENCE-ANYHOW-001 | P0 |
| DEF-003 | 通用 hash_bytes | §11.5, EVIDENCE-DOMAIN-001 | P0 |
| DEF-004 | 全零 genesis | §11.4, EVIDENCE-GENESIS-001 | P0 |
| DEF-005 | 公开可变 EvidenceRecord 字段 | §9.1, EVIDENCE-PUBLIC-001 | P0 |
| DEF-006 | Debug 字符串参与 digest（domain_macro） | §11, EVIDENCE-DEBUG-HASH-001 | P0 |
| DEF-007 | Mock verify 永远成功 | §19.2, §31.1 | P0 |
| DEF-008 | lock poison 静默空/unwrap 路径 | §5, §19 | P1 |
| DEF-009 | chain corrupt → Invalid | §18.2 | P1 |
| DEF-010 | README「不可篡改」 | §1.1 | P1 |
| DEF-011 | 无 chain_id/sequence/event_id | §6–§13 | P0 |
| DEF-012 | 无 durable adapter | §20–§21 | P1（生产阻断） |
| DEF-013 | 无 checkpoint/anchor | §16 | P1（生产阻断） |
| DEF-014 | 无 golden vectors | §24.1 | P0（V1 冻结条件） |
| DEF-015 | 无 evidence-policy.toml | §26 | P1 |
| DEF-016 | ADR-010 / 旧 spec 与 002 冲突 | §0, §31 | GOVERNANCE |
| DEF-017 | fail-open/fail-closed 未在调用方落地 | §13.7, §15 | P0 for required ops |
| DEF-018 | LE 字节序 + 字段拼接歧义 module/op | §11 | P0 |
| DEF-019 | ADR-012 auditx 路径 vs 002 crates/evidence | ADR-012 / plan §1.3 | GOVERNANCE |
| DEF-020 | 迁移期双 package 同名 `evidence` | I-18 / Cargo | P0 |

枚举 SSOT：[`spec-inventory.md`](./spec-inventory.md) · residual：[`residual-open.md`](./residual-open.md)

---

## 5. 与 INFRA-003 / 工作包 Evidence 的边界

| 系统 | 用途 | 路径 | 不得 |
|------|------|------|------|
| **crate `evidence`（本战役）** | 运行时业务审计链 | crates/evidence + adapters | 用工作包 CI 记录冒充业务链 |
| **INFRA-003 / evidence/infrastructure** | 工作包验收证据 | evidence/infrastructure、schemas | 用业务链代替 CI artifact 权威 |

两者共享「可验证、可追溯、禁假 PASS」文化，但 **schema/API 独立**。本计划只闭合 SPEC-EVIDENCE-002。

---

## 6. 差距闭合优先级（执行序）

```text
P0 冻结扩散 + 规格/台账诚实
  → P1 Core V1 + golden（代码信任根）
    → P2 Legacy bridge（不伪造历史）
      → P3 Domain/gate 迁移（消除 Debug hash）
        → P4 Durable adapters + conformance
          → P5 Checkpoint + anchor
            → P6 Cutover + 门禁 + 删 tools/evidence
              → 十轮验收 ×10 → 人审 Approved → 0.1.1 / stable
```

**禁止**：在 Core V1 + golden 之前宣称 durable/production-ready。
**禁止**：把旧六字段静默 rehash 成 V1 并声称历史连续。
**禁止**：在 Spec 仍为 Proposed 时勾 §33 全绿或 registry stable。
