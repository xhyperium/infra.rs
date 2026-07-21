# Plan — SPEC-EVIDENCE-002 完整执行计划（v1）

| 字段 | 值 |
|------|-----|
| Plan ID | `PLAN-EVIDENCE-002-v1-complete` |
| Source Spec | [`.agents/ssot/tools/evidence/xhyper-evidence-complete-spec.md`](../xhyper-evidence-complete-spec.md) |
| Spec ID | `SPEC-EVIDENCE-002` · Status **Approved**（2026-07-14 ZoneCNH）→ Target **Stable** |
| Goal | `GOAL-EVIDENCE-AUDIT-TRUST` |
| Package | `evidence` @ `crates/evidence` **0.1.0** → 目标 **0.1.1**（发布/A9 Defer） |
| Layer | L0 Audit Core（runtime 在 `crates/evidence`；CLI `tools/evidence-cli`） |
| Baseline | `main@007ca7b5`（开战役）· 实现 PR #253 |
| Gap Matrix | [`gap-matrix.md`](./gap-matrix.md) |
| Spec Inventory | [`spec-inventory.md`](./spec-inventory.md)（I-1…I-26 防遗漏枚举） |
| Tasks | [`tasks.md`](./tasks.md) |
| Residual | [`residual-open.md`](./residual-open.md) |
| Approval Packet | [`approval-packet.md`](./approval-packet.md) · **人审签字 DONE** |
| Work Todo | [`.worktrees/evidence-todo.md`](../../../../.worktrees/evidence-todo.md) |
| 10x Verdict | [`evidence-plan-10x-verdict.md`](./evidence-plan-10x-verdict.md) |
| Strategy | **诚实台账 → 冻结错误扩散 → Core V1 冻结 → 桥接 → 领域迁移 → 持久化 → 检查点 → 切换 → 十轮验收 → 人审** |
| Campaign status | **IMPLEMENTATION + Spec Approved** · cutover DONE · **≠ package stable** · **≠ §33 全闭合** |
| Alignment | [`evidence/evidence-002/alignment-2026-07-14.md`](../../../../../evidence/evidence-002/alignment-2026-07-14.md) |
| Forbidden | 见 **I-26** 统一十条（假 Done / stable / 静默 rehash / 禁依赖 / mock 生产 / hash_bytes / 损坏→Invalid / SKIP=PASS / 私钥进库 / 双包同名 / AI 独断 Approved） |

---

## 0. 深度分析结论（对照完整规范 §0–§34）

### 0.1 Evidence 是什么

`evidence` 的生产终态不是「写一条日志」，也不是当前的：

```text
Vec<Record> + SHA256(prev || fields)
```

而是可独立验证的 **tamper-evident audit evidence** 链：

```text
typed EvidenceDraft
→ canonical V1 preimage
→ domain-separated SHA-256
→ contiguous sequence + ChainHead
→ idempotent linearizable append
→ durable persistence + crash recovery
→ signed checkpoint + independent anchor
→ reproducible verifier / CLI
```

在无可信签名检查点与独立外部锚点时，**禁止**对外使用「不可篡改 / 绝对可信 / 无法删除 / 永久证明」。

### 0.2 准入四问（任一否决则不得进 core）

1. 是否为审计事实链的 **模型、编码、摘要、封链或纯验证**？
2. 是否可在 **无 IO / 无运行时 / 无 serde** 的 core 中表达？
3. 是否引入 **不稳定摘要**（Debug/JSON/未域分离 hash）？
4. 是否把 **测试替身** 装进生产依赖图？

只有 1=是、2=是、3=否、4=否 才能进入 `crates/evidence`。

### 0.3 现状一句话

当前 `tools/evidence` 是 **incubating prototype**：六字段内存哈希链 + `hash_bytes` + 全零 genesis + mock 恒成功；被 `domain_macro`（Debug 字符串摘要）与 `gate` 使用。与 SPEC-EVIDENCE-002 **全面不对齐**，**不得**标记 production/stable。

### 0.4 章节级现状（摘要；明细见 gap-matrix）

| 区间 | 主题 | 综合状态 |
|------|------|----------|
| §0–§5 | 定位/职责/目录/依赖/crate 规则 | **WRONG / ABSENT** |
| §6–§12 | 值对象 / Draft / RecordV1 / 时间 / canonical / chain | **ABSENT / WRONG** |
| §13–§18 | Append/Reader/原子性/Checkpoint/Verify/Error | **ABSENT / WRONG** |
| §19–§21 | memory/file/postgres adapters | **WRONG / ABSENT** |
| §22–§26 | 保留/演进/测试/CLI/政策 | **ABSENT** |
| §27–§30 | 门禁/CI/性能/可观测 | **ABSENT** |
| §31–§34 | 迁移/自身 Evidence/完成定义/裁定 | **文档有；实现无** |

### 0.5 硬阻断（未关则禁止 stable）

1. **DEF-001** runtime 仍在 `tools/`
2. **DEF-003/006/018** 无 domain 的 hash / Debug hash / 字段拼接歧义
3. **DEF-004** 全零 genesis
4. **DEF-014** 无 golden vectors → V1 不可冻结
5. **DEF-012/013** 无 durable + 无 checkpoint（生产审计不可用）
6. **DEF-016** Spec 仍 Proposed；ADR-010 冲突未修订
7. **DEF-017** required 操作 fail-closed 未落地

---

## 1. 执行策略与原则

```text
1. 证据优先：任何 PASS 必须绑定命令输出或 evidence/system/<date>-… 文件
2. 外科手术：只改 evidence 闭合所需路径；不顺手重构无关 crate
3. 单 writer：同一文件路径不并行写（agent team 按路径分片）
4. residual 纪律：OPEN / CLOSED / DEFER(accepted) only；禁止静默 Unknown
5. 禁止：registry stable、§33 全勾、手写 PASS 代替命令
6. 禁止：旧六字段 rehash 后声称与历史 V1 连续
7. 禁止：core 引入 anyhow/serde/tokio/uuid/chrono
8. 人审闸门：Spec Approved / 0.1.1 / stable 不由 AI 独断关闭
9. 十轮验收：fail_rounds 必须为 0 才可宣称「十轮通过」
10. 分支纪律：禁止 main 直接开发；worktree `.worktrees/workspaces/<branch>`
```

### 1.1 分支与 worktree

```text
branch:  feat/evidence-002-core-v1   （后续按波次可拆 PR 栈）
worktree: .worktrees/workspaces/feat-evidence-002-core-v1
base:    origin/main HEAD
```

大战役可拆 PR 栈：

| PR 栈 | 内容 |
|-------|------|
| PR1 | W0 文档/冻结/措辞 + policy 骨架 |
| PR2 | W1 crates/evidence Core V1 + golden |
| PR3 | W2 memory adapter + traits 拆出 core |
| PR4 | W3 domain_macro + gate 迁移 |
| PR5 | W4 file adapter |
| PR6 | W4 postgres/outbox |
| PR7 | W5 checkpoint + CLI |
| PR8 | W6 门禁 + cutover 删除 tools/evidence |

### 1.2 与 INFRA-003 边界

- **本计划** = 运行时审计 crate `evidence`（业务事实链）
- **INFRA-003** = 工作包 CI/验收 Evidence 协议（`evidence/infrastructure`）
- 禁止混用 schema；允许交叉引用「禁假 PASS」原则

### 1.3 与 ADR-012（auditx）路径冲突

ADR-012 曾写：`tools/evidence` 运行时 → `crates/infra/auditx`，CI 工具仍留 `tools/evidence`。  
SPEC-EVIDENCE-002 裁定目标为 **`crates/evidence`（L0）** + `crates/adapters/evidence/*` + `tools/evidence-cli`。

| 决议状态 | 说明 |
|----------|------|
| **未人审二选一** | 见 approval **A11**；`T-DOC-005` |
| 本计划默认实现目标 | **002 路径**（L0 `crates/evidence`），因 R1 允许 domain→evidence 且 002 为完整生产合同 |
| 禁止 | 实现期静默同时创建 auditx 与 evidence 两套 runtime |

### 1.4 Spec Inventory 强制引用

实现 Task 的 AC **不得**仅写「见 §xx 全覆盖」。必须引用 `spec-inventory.md` 中的 **I-*** 表项（preimage 25 步、24 错误、23 门禁、14 golden、11 metrics 等）。

### 1.5 P0–P6 ↔ Wave（I-14）

| Spec | Wave | 前缀 |
|------|------|------|
| P0 冻结 | W0 | T-FREEZE / T-DOC |
| P1 Core V1 | W1 | T-CORE |
| P2 bridge | W3 | T-LEG |
| P3 domain | W3 | T-DOM / T-GATE |
| P4 durable | W4 | T-FILE / T-PG |
| P5 checkpoint | W5 | T-CP / T-CLI |
| P6 cutover | W6 | T-CUT / T-ARCH / T-BOOT |
| 验收 | W7 | T-V10 |
| 人审 | W8–W9 | T-HUM / T-33 |

---

## 2. 波次（Waves）与依赖 DAG

```text
W0  台账/冻结/措辞/政策骨架     ──┐
W1  Core V1 + golden + 错误映射  ──┼──→ W2  Memory adapter + Append/Reader
W3  Domain/gate 迁移 + policy 登记 ──┤
W4  File + Postgres adapters     ──┤
W5  Checkpoint + CLI + 观测/基准 ──┼──→ W6 门禁/CI/cutover 准备
W7  十轮验收 ×10                 ──┤
W8  人审 Approved + version      ──┘
W9  §33 闭合声明（仅人审后）
```

| Wave | 名称 | 可并行 | Owner | 退出条件 |
|------|------|--------|-------|----------|
| **W0** | 冻结错误扩散 + 台账 | 是（docs） | Doc/Policy | 措辞修正；禁新增 hash_bytes/Debug-hash；quality=incubating；plan/todo 一致 |
| **W1** | Core V1 | 否（新 crate） | Core Agent | crates/evidence 编译；canonical+golden；无 anyhow；字段私有 |
| **W2** | Memory adapter + contracts | 依赖 W1 | Adapter-Mem | Append/Reader/幂等/CAS；Volatile only；conformance 子集 |
| **W3** | Domain/gate + policy | 依赖 W2 | Domain Agents | 删除 Debug hash；required ops 登记；失败路径 evidence |
| **W4** | File + Postgres | 可互为并行（不同包） | Adapter-File / Adapter-Pg | fsync/crash；outbox 原子；conformance |
| **W5** | Checkpoint + CLI | 依赖 W1+W2 | Crypto/CLI | SignedCheckpoint；verify/inspect/export；anchor 合同 |
| **W6** | 门禁 + cutover | 依赖 W3–W5 | Gate/Release | EVIDENCE-* 机控；删 tools/evidence；路径 SSOT |
| **W7** | 十轮全量检查 | 串行 10 轮 | Verify Team | fail_rounds=0 |
| **W8** | 人审 + 0.1.1 | 人 | Owner | Spec Approved；version bump 策略 |
| **W9** | §33 全勾 | 人+机 | Owner | 33.1–33.6；registry stable 单独决策 |

### 2.1 本会话 AI 默认可执行范围

| 可立即执行 | 需人决策 / 外部 |
|------------|-----------------|
| W0 plan/todo/gap/approval 包 | Spec Status → Approved |
| W0 文档措辞（README 去「不可篡改」） | ADR-010 修订合并 |
| W1 在 feature 分支 scaffold crates/evidence | production anchor 真实 KMS/OSS |
| W1 golden vectors + core 实现 | postgres 真实联调环境 |
| W2 memory adapter | registry stable |
| 十轮 **计划完备性** 检查 | 0.1.1 publish |
| 局部 domain_macro 迁移草案 | 法律 retention 天数最终值 |

**本轮默认目标**：落盘完整计划 + todo + **十轮无遗漏检查** + 启动 W0/W1 scaffold（若分支就绪）。**不**宣称 §33 闭合。

---

## 3. Agent Team 编制与路径互斥

### 3.1 角色

| Role | 职责 | 写入路径（互斥） |
|------|------|------------------|
| **Planner** | plan / gap / residual / todo | `.agents/ssot/tools/evidence/plan/*`、`.worktrees/evidence-todo.md` |
| **Executor-Core** | crates/evidence 全模块 | `crates/evidence/**` |
| **Executor-Mem** | memory adapter | `crates/adapters/evidence/memory/**` |
| **Executor-File** | file adapter | `crates/adapters/evidence/file/**` |
| **Executor-Pg** | postgres/outbox | `crates/adapters/evidence/postgres/**` |
| **Executor-CLI** | evidence-cli | `tools/evidence-cli/**` |
| **Executor-Domain** | domain_macro 迁移 | `crates/domain/macro/**` |
| **Executor-Gate** | gate 迁移 | `crates/infra/gate/**` |
| **Executor-Policy** | evidence-policy + arch | `.architecture/evidence-policy.toml`、policies |
| **Executor-Archgate** | EVIDENCE-* 规则 | `tools/archgate/**`、`tools/xtask/**` |
| **Doc-Sync** | README/AGENTS/CHANGELOG/spec 状态 | 各 crate 文档；**不**改 core 逻辑 |
| **Verifier** | 十轮清单 + verdict | `evidence/system/<date>-*/` |
| **Skeptic** | 反证假 PASS / 遗漏 residual | 只读 + review 批注 |

### 3.2 并行规则

```text
同时可写：Core vs File vs Pg vs CLI（不同树）
不可同时写：Core 与 workspace Cargo.toml 成员表（串行合并）
W3 domain 与 gate 可并行（不同 crate）
Verifier 只读；Skeptic 只读
任何 Wave 结束 → 单点集成 → 再开下一波并行
```

### 3.3 Team 流水线（每波）

```text
team-plan → team-prd（原子任务）→ team-exec（按路径）→ team-verify → team-fix（≤N 次）
```

---

## 4. 十轮验收协议（W7；计划检查复用同一清单）

每轮 **独立** 从下列清单执行；任一项 FAIL → 该轮 FAIL。  
宣称「十轮通过」当且仅当 `fail_rounds == 0`。

### Round checklist R1–R10（完整，禁止缩水）

| # | Check ID | 检查项 | 通过判据 |
|---|----------|--------|----------|
| 1 | R-SPEC-001 | Spec 文件存在且 ID=SPEC-EVIDENCE-002 | 路径+页眉 |
| 2 | R-SPEC-002 | §0–§34 无章节在 gap-matrix 缺失 | 34 行齐全 |
| 3 | R-SPEC-003 | 完成定义 §33.1–33.6 全部映射到 Task | tasks.md 可追踪 |
| 4 | R-GAP-001 | DEF-001…018 全部登记 | residual/todo 有 ID |
| 5 | R-GAP-002 | T1–T18 全部有覆盖策略 | gap-matrix §2 |
| 6 | R-PATH-001 | 目标路径 crates/evidence 写明 | plan §5 |
| 7 | R-PATH-002 | adapters memory/file/postgres 写明 | plan §5 |
| 8 | R-PATH-003 | tools/evidence-cli + 删除 tools/evidence | plan W6 |
| 9 | R-DEP-001 | core 白名单 kernel+sha2+thiserror | plan/tasks |
| 10 | R-DEP-002 | 禁止 anyhow/serde/tokio 列表完整 | gap+plan |
| 11 | R-API-001 | Digest32/ChainId/EventId/OperationId/EvidenceName | tasks W1 |
| 12 | R-API-002 | EvidenceDraft + Outcome 六态 | tasks W1 |
| 13 | R-API-003 | EvidenceRecordV1 字段私有 + seal_record_v1 | tasks W1 |
| 14 | R-CANON-001 | Record preimage 25 步顺序完整 | tasks/golden |
| 15 | R-CANON-002 | genesis domain tag 非全零 | tasks |
| 16 | R-CANON-003 | 边界 ("ab","c")≠("a","bc") | 测试任务 |
| 17 | R-CANON-004 | digest_canonical 域分离；禁 hash_bytes | tasks |
| 18 | R-TIME-001 | recorded_at / event_time 分离 | tasks |
| 19 | R-CHAIN-001 | sequence 从 1；ChainHead；禁 0+零摘要空链 | tasks |
| 20 | R-APPEND-001 | Durability 三态；Durable 生产默认 | tasks |
| 21 | R-APPEND-002 | 幂等 + HeadConflict + fail-closed | tasks |
| 22 | R-READ-001 | head/get_by_event_id/read_range 合同 | tasks |
| 23 | R-ATOM-001 | 同事务 / outbox / SoT 三选一 | tasks W4 |
| 24 | R-CP-001 | CheckpointV1 + Signed + TailTruncated | tasks W5 |
| 25 | R-ERR-001 | EvidenceError 全集 + XError 映射表 | tasks W1 |
| 26 | R-MEM-001 | production_allowed=false；禁伪 Durable | tasks W2 |
| 27 | R-FILE-001 | segment frame + fsync + recovery | tasks W4 |
| 28 | R-PG-001 | heads/records/outbox/checkpoints 表不变量 | tasks W4 |
| 29 | R-TEST-001 | golden/property/fuzz/coverage/mutants/miri | tasks |
| 30 | R-CLI-001 | verify/inspect/head/export/checkpoint/vectors/repair-tail | tasks W5 |
| 31 | R-POL-001 | evidence-policy.toml 结构 | tasks W0/W3 |
| 32 | R-GATE-001 | EVIDENCE-PATH/DEP/ANYHOW/… 全列表 | tasks W6 |
| 33 | R-MIG-001 | P0–P6 与 Wave 对齐；旧链不静默重编码 | plan §2/§6 |
| 34 | R-EVID-001 | §32 自身 Evidence 目录模板 | plan §8 |
| 35 | R-DOWN-001 | domain_macro + gate 迁移任务存在 | tasks W3 |
| 36 | R-GOV-001 | ADR-010 修订 / Spec Approved 为人审闸 | approval-packet |
| 37 | R-FORBID-001 | Forbidden 清单无矛盾执行步骤 | plan 页眉 |
| 38 | R-TODO-001 | evidence-todo 覆盖全部 Wave 与 DEF | todo 文件 |
| 39 | R-CROSS-001 | 与 INFRA-003 边界声明 | plan §1.2 |
| 40 | R-HONEST-001 | 无假 PASS / 未把 Proposed 写成 Approved | 全文检索 |

### 4.1 计划完备性十轮（本阶段强制）

在写代码前，对 **plan + gap + tasks + todo + approval** 跑 **10 轮** 上表检查（可并行由 Verifier/Skeptic 交叉）。  
每轮输出：

```text
round: N
result: PASS|FAIL
failed_checks: [...]
omissions: [...]
```

汇总：`evidence-plan-10x-verdict.md` → `fail_rounds` 必须为 0。

### 4.2 实现后十轮（W7）

同一清单 + 机器命令：

```bash
cargo fmt -- --check
cargo clippy -p evidence --all-targets -- -D warnings
cargo test -p evidence
cargo test -p evidence_memory
cargo test -p evidence_file
cargo test -p evidence_postgres
cargo run -p evidence-cli -- vectors verify
cargo xtl lint-deps
# + coverage / mutants / miri per §28
```

---

## 5. 目标目录与模块落地顺序（W1）

```text
crates/evidence/
  Cargo.toml          # name=evidence; deps: kernel, sha2, thiserror
  README.md / AGENTS.md / CHANGELOG.md
  src/
    lib.rs            # forbid unsafe; deny missing_docs/unreachable_pub
    error.rs          # EvidenceError + Into<XError>
    ids.rs            # Digest32, ChainId, EventId, OperationId
    name.rs           # EvidenceName
    digest.rs         # genesis, digest_canonical, domain tags
    draft.rs          # EvidenceDraft, EvidenceOutcome, EvidenceActor
    record.rs         # EvidenceRecordV1, seal_record_v1
    canonical/mod.rs
    canonical/v1.rs   # encode/decode preimage
    chain.rs          # ChainHead
    checkpoint.rs     # CheckpointV1 preimage（签名在 adapter）
    verify.rs         # pure verify + VerificationReport
    contracts.rs      # EvidenceAppender, EvidenceReader, Durability, Append*
  tests/
    golden_vectors.rs
    canonical_properties.rs
    chain_properties.rs
    compile_fail.rs   # 或 static_assertions 过渡
    fuzz_regressions.rs
tests/vectors/evidence-v1/   # 仓库级 golden（路径以落地为准，须 SSOT）

crates/adapters/evidence/memory/
crates/adapters/evidence/file/
crates/adapters/evidence/postgres/

tools/evidence-cli/
```

**删除（仅 W6 cutover 后）**：`tools/evidence`。

迁移期允许 **双包短暂共存**（legacy feature 或 `evidence_legacy`），但：

- 新代码不得新增 `hash_bytes` / `InMemoryEvidenceSink` 生产用法；
- domain/gate 迁完后删除旧包。

---

## 6. 迁移语义（强制）

### 6.1 旧链不可无损升级

旧六字段 **不能** 补出：`chain_id, sequence, event_id, operation_id, actor, subject, recorded_at/event_time`。

正确：

```text
V1 新链 genesis(chain_id)
→ 首条 migration record 引用「旧链最终 digest + migration manifest digest」
→ 旧链保留只读 verifier（legacy）
→ 禁止声称字节级历史连续
```

### 6.2 P0 冻结清单（W0 立即）

```text
- 禁止新增 hash_bytes 调用
- 禁止新增 Debug → hash
- 禁止新增 InMemoryEvidenceSink 生产使用
- quality 降为 incubating
- 「不可篡改」→ tamper-evident prototype
```

---

## 7. 任务索引（详见 tasks.md）

| 批次 | Task 前缀 | 数量级 | Wave |
|------|-----------|--------|------|
| 台账 | T-PLAN / T-TODO / T-DOC / T-RES | ~15 | W0 |
| Core | T-CORE-* | ~40 | W1 |
| Memory | T-MEM-* | ~15 | W2 |
| Domain/Gate | T-DOM-* / T-GATE-* | ~20 | W3 |
| File | T-FILE-* | ~20 | W4 |
| Postgres | T-PG-* | ~20 | W4 |
| Checkpoint/CLI | T-CP-* / T-CLI-* | ~25 | W5 |
| 门禁/CI/Cutover | T-ARCH-* / T-CI-* / T-CUT-* | ~20 | W6 |
| 验收 | T-V10-* / T-EVID-* | ~15 | W7–W9 |

---

## 8. Evidence 自身包（§32）

每次系统变更：

```text
evidence/system/<date>-<change-id>/
├── manifest.json
├── spec-version.txt
├── commit.txt
├── toolchain.txt
├── commands.log
├── fmt.log / clippy.log / tests.log
├── coverage.json / mutants.json / fuzz-summary.json
├── golden-vector-diff.txt
├── adapter-conformance.json
├── recovery-tests.json
├── public-api.diff
├── schema-compatibility.md
├── threat-model-review.md
└── verdict.md
```

禁止：被测系统自证唯一可信；SKIP=PASS；旧 commit 冒充当前；手写 digest。

---

## 9. §33 完成定义映射（勾选纪律）

| §33 节 | 闭合条件 | 最早 Wave | AI 可独断？ |
|--------|----------|-----------|-------------|
| 33.1 规格 | Approved + superseded + ADR 修订 + policy + registry | W8 | **否** |
| 33.2 Core | crates/evidence + V1 全项 | W1+W7 | 实现可；stable 否 |
| 33.3 Adapter | mem/file/pg + conformance | W4+W7 | 实现可；生产锚否 |
| 33.4 Checkpoint | signed+anchor+tail | W5+W7 | 部分需外部 |
| 33.5 测试 | golden/fuzz/cov/mutants/miri | W7 | 环境依赖 |
| 33.6 系统 | required ops + fail-closed + CI Evidence | W3–W8 | 否（全系统） |

**任何未满足项保持 OPEN；禁止为赶工降级判据。**

---

## 10. 风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| Spec 长期 Proposed | 实现与治理脱节 | approval-packet；实现可在 feature 分支推进但不 stable |
| 双包共存过久 | 调用方继续用旧 API | W0 冻结；W3 优先迁 domain_macro |
| Debug hash 已在历史数据 | 无法重算 | migration manifest；不重写历史 |
| File fsync 语义平台差 | 假 Durable | 合同测试 + 文档平台假设 |
| Postgres 无环境 | W4 阻塞 | 先 mem+file；pg 标 BLOCKED 但不假 PASS |
| 门禁未机控 | 回归 | W6 EVIDENCE-* 进 archgate/xtask |
| Agent 并行写冲突 | 损坏 | 路径互斥表 §3.1 |
| 与 INFRA-003 混淆 | 错误关闭 WP | §1.2 边界 |

---

## 11. 成功判据（战役级，非本会话）

仅当下列 **全部** 为真：

1. §33.1–33.6 有证据勾选（人审 + 机器）
2. `fail_rounds == 0`（实现十轮）
3. `tools/evidence` 已删除且 workspace 无回归
4. 无 generic `hash_bytes`；无生产 memory；无 anyhow in core
5. golden vectors 独立 verifier 复算一致
6. Spec Status = Approved；package quality 达约定档（stable 另决策）

**本计划落盘与十轮计划检查通过 ≠ 战役完成。**

---

## 12. 变更日志（本 plan）

| 日期 | 版本 | 说明 |
|------|------|------|
| 2026-07-14 | v1 | 首版完整执行计划；对照 SPEC-EVIDENCE-002 §0–§34；baseline 007ca7b5 |
| 2026-07-14 | v1.1 | 十轮 FAIL 后补：spec-inventory I-1…I-26；消灭幽灵 T-ATOM；全量 EVIDENCE-*；双包隔离；ADR-012 A11；原子性/隐私/背压/schema 任务；拆 CI 草案桶 |
| 2026-07-14 | v1.1.1 | 计划 10x pass3 PASS；W0 文档/policy；架构 SSOT + CLAUDE/AGENTS/TECH/CHANGELOG 对齐；alignment-2026-07-14 |
