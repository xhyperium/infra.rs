# Tasks — SPEC-EVIDENCE-002 原子任务表

| 字段 | 值 |
|------|-----|
| Plan | `PLAN-EVIDENCE-002-v1-complete` |
| Status enum | `TODO` · `IN_PROGRESS` · `DONE` · `BLOCKED` · `DEFER` · `CANCELLED` |
| Baseline | `main@007ca7b5` |

> 完成定义：每条 Task 的 AC 必须可机器或文件证据验证；`DONE` 禁止无输出。  
> 依赖列使用 Task ID。路径互斥见 plan §3.1。

---

## W0 — 台账 / 冻结 / 政策骨架

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-PLAN-001 | 落盘 plan.md | 含 §0–§12；Forbidden 清单 | — | Planner | **DONE** |
| T-PLAN-002 | 落盘 gap-matrix.md | §0–§34 + T1–T18 + DEF-001…018 | — | Planner | **DONE** |
| T-PLAN-003 | 落盘 tasks.md | 覆盖 §33 全部勾选项 | — | Planner | **DONE** |
| T-PLAN-004 | 落盘 approval-packet.md | 人审闸门清晰 | — | Planner | **DONE** |
| T-TODO-001 | 更新 `.worktree/evidence-todo.md` | 全 Wave/DEF 可追踪 | T-PLAN-* | Planner | **DONE** |
| T-DOC-001 | tools/evidence README 去「不可篡改」 | 改为 tamper-evident prototype | — | Doc | **DONE** |
| T-DOC-002 | 标记旧 evidence-spec superseded 指向 | 页眉/链接到 002 | — | Doc | **DONE** |
| T-DOC-003 | architecture/spec.md 路径备注「迁移中」 | 不假装已迁 crates/ | — | Doc | **DONE** |
| T-POL-001 | 骨架 `.architecture/evidence-policy.toml` | schema_version + 示例 chain/operation | — | Policy | **DONE** |
| T-FREEZE-001 | 登记 P0 冻结：禁新增 hash_bytes/Debug-hash/生产 InMemory | residual + README 冻结节 | — | Doc | **DONE** |
| T-RES-001 | residual-open 清单初始化 | DEF 全登记 OPEN | T-PLAN-002 | Planner | **DONE** |
| T-INV-001 | 落盘 spec-inventory.md | I-1…I-26 枚举齐全 | T-PLAN-002 | Planner | **DONE** |
| T-V10-PLAN | 计划完备性十轮检查 | fail_rounds=0；verdict 文件 | T-PLAN-* T-TODO-001 T-INV-001 | Verifier | **DONE** |
| T-BRANCH-001 | 创建 feat 分支/worktree | 非 main 开发 | — | Lead | **DONE**（`docs/evidence-002-plan`；实现波另开 feat） |
| T-DOC-005 | ADR-012 auditx 路径 vs 002 crates/evidence 对账 | 冲突写入 approval A11 + alignment + spec §0；不静默选边 | — | Doc | **DONE**（登记；人审仍 OPEN） |
| T-ALIGN-001 | 执行后文档对齐包 | architecture/CLAUDE/AGENTS/TECH/CHANGELOG + evidence/evidence-002/alignment | T-DOC-003 | Doc | **DONE** |

---

## W1 — Core V1（`crates/evidence`）

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-CORE-001 | workspace member `crates/evidence` | cargo metadata 可见 | T-BRANCH-001 | Core | TODO |
| T-CORE-002 | Cargo.toml：kernel+sha2+thiserror only | 无 anyhow/serde/tokio | T-CORE-001 | Core | TODO |
| T-CORE-003 | lib.rs 属性 forbid/deny | 与 §5 一致 | T-CORE-001 | Core | TODO |
| T-CORE-004 | Digest32 API | from/as/into_bytes；无 Default | T-CORE-002 | Core | TODO |
| T-CORE-005 | ChainId + derive 规则 | domain tag CHAIN-ID:V1 | T-CORE-004 | Core | TODO |
| T-CORE-006 | EventId / OperationId | 32-byte newtype | T-CORE-004 | Core | TODO |
| T-CORE-007 | EvidenceName 校验 | regex/长度/ASCII；InvalidName | T-CORE-002 | Core | TODO |
| T-CORE-008 | EvidenceActor | namespace+id_digest 私有字段 | T-CORE-007 | Core | TODO |
| T-CORE-009 | EvidenceOutcome 六态 | non_exhaustive；tag 0x00–0x05 | T-CORE-004 | Core | TODO |
| T-CORE-010 | EvidenceDraft + builder | 无私有 seal 字段可写 | T-CORE-008 T-CORE-009 | Core | TODO |
| T-CORE-011 | Timestamp 使用 kernel::Timestamp | recorded_at/event_time | T-CORE-002 | Core | TODO |
| T-CORE-012 | EvidenceRecordV1 全私有 + getters | 无公开可变字段 | T-CORE-010 | Core | TODO |
| T-CORE-013 | seal_record_v1 | 不接受外部 record_digest | T-CORE-012 | Core | TODO |
| T-CORE-014 | canonical v1 encode | **I-1 逐步 1..25** 顺序+宽度+BE+presence；单测逐步 | T-CORE-013 | Core | TODO |
| T-CORE-015 | canonical v1 decode | 拒非法 presence/tag/trailing/truncation/超长；任意输入不 panic | T-CORE-014 | Core | TODO |
| T-CORE-016 | record_digest = SHA256(preimage) | 持久化 preimage\|\|digest；digest 不入自身 preimage | T-CORE-014 | Core | TODO |
| T-CORE-017 | genesis_digest(chain_id) | I-3 GENESIS tag；非全零 | T-CORE-005 | Core | TODO |
| T-CORE-018 | digest_canonical(domain, bytes) | I-3 CONTENT；**禁**公开 hash_bytes | T-CORE-007 | Core | TODO |
| T-CORE-019 | ChainHead + Option 空链 | 禁 0+零摘要 | T-CORE-005 | Core | TODO |
| T-CORE-020 | EvidenceError 全集 | **I-4 全部 24 variant** 存在且 non_exhaustive | T-CORE-002 | Core | TODO |
| T-CORE-021 | EvidenceError → XError 映射 | **I-4 映射表**逐 variant 测；链损坏≠Invalid | T-CORE-020 | Core | TODO |
| T-CORE-022 | Durability/AppendRequest/Receipt | **I-5 三态** Volatile/Process/Durable | T-CORE-012 T-CORE-019 | Core | TODO |
| T-CORE-023 | trait EvidenceAppender / EvidenceReader | §13–§14；limit 1..=10000 | T-CORE-022 | Core | TODO |
| T-CORE-024 | CheckpointV1 + preimage/digest | §16.1（无私钥） | T-CORE-019 | Core | TODO |
| T-CORE-025 | pure verify + VerificationReport | §17.2 检测项全覆盖测试矩阵 | T-CORE-015 T-CORE-016 | Core | TODO |
| T-CORE-026 | golden vectors 目录落地 | **I-6 全部 14 GV-*** 四元组 | T-CORE-016 | Core | TODO |
| T-CORE-027 | golden 测试 | 跨运行稳定；独立复算 | T-CORE-026 | Core | TODO |
| T-CORE-028 | property: encode/decode 稳定 | proptest；invalid length/tag/truncation/no-panic | T-CORE-015 | Core | TODO |
| T-CORE-029 | 边界 ("ab","c")≠("a","bc") | 强制回归 | T-CORE-014 | Core | TODO |
| T-CORE-030 | chain properties 篡改检测 | §24.3 九类失败各一测 | T-CORE-025 | Core | TODO |
| T-CORE-031 | 无 unsafe/todo/unwrap 生产路径 | clippy -D warnings | T-CORE-025 | Core | TODO |
| T-CORE-032 | README/AGENTS/CHANGELOG core | 中文；tamper-evident；禁「不可篡改」 | T-CORE-001 | Doc | TODO |
| T-CORE-033 | 覆盖率 line≥95% | llvm-cov fail-under-lines 95 | T-CORE-027 | Quality | TODO |
| T-CORE-034 | public API 无 mock feature | default=[] | T-CORE-002 | Core | TODO |
| T-CORE-035 | compile_fail 或 static_assertions | 防公开字段 | T-CORE-012 | Core | TODO |
| T-CORE-036 | Outcome tag 表 I-2 | 0x00–0x05 编解码往返 | T-CORE-009 | Core | TODO |
| T-CORE-037 | IdempotencyConflict 语义测 | 同 event 不同内容 → 该错误 | T-CORE-020 T-MEM-002 | Core | TODO |
| T-CORE-038 | Process durability 语义文档+枚举测 | 与 Volatile/Durable 可区分 | T-CORE-022 | Core | TODO |

---

## W2 — Memory adapter

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-MEM-001 | package evidence_memory 路径 | crates/adapters/evidence/memory | T-CORE-023 | Mem | TODO |
| T-MEM-002 | 实现 Appender：seal+sequence+idempotent+CAS | 合同测试 | T-MEM-001 | Mem | TODO |
| T-MEM-003 | 实现 Reader | head/get/range 限制 | T-MEM-001 | Mem | TODO |
| T-MEM-004 | Durable 请求 → DurabilityFailure | 不得伪 Durable | T-MEM-002 | Mem | TODO |
| T-MEM-005 | 并发线性化测试 | 无 fork/dup sequence | T-MEM-002 | Mem | TODO |
| T-MEM-006 | lock poison 不返回空集合 | 返回错误 | T-MEM-002 | Mem | TODO |
| T-MEM-007 | production_allowed 文档+标记 | README 明确 false | T-MEM-001 | Mem | TODO |
| T-MEM-008 | conformance suite 钩子 | 可被 file/pg 复用 | T-MEM-002 | Mem | TODO |
| T-MEM-009 | 并发 1000+ append 子集 | 无 fork/dup/lost | T-MEM-005 | Mem | TODO |
| T-MEM-010 | head 永不返回零摘要哨兵 | 空链 None | T-MEM-003 | Mem | TODO |

---

## W3 — Domain / Gate / Policy / Atomicity / Legacy

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-DOM-001 | domain_macro 去掉 Debug hash | 使用 digest_canonical | T-MEM-002 | Domain | TODO |
| T-DOM-002 | advance 成功/拒绝/失败 typed outcome | EvidenceOutcome | T-DOM-001 | Domain | TODO |
| T-DOM-003 | 稳定 EventId/OperationId | 重试幂等 | T-DOM-001 | Domain | TODO |
| T-DOM-004 | chain policy 登记 macro | evidence-policy.toml | T-POL-001 T-DOM-001 | Policy | TODO |
| T-DOM-005 | fail-closed：evidence 失败不伪装业务拒绝 | 测试 | T-DOM-002 | Domain | TODO |
| T-DOM-006 | 成功+失败路径合同测试 | §24 / §33.6 | T-DOM-002 | Domain | TODO |
| T-GATE-001 | gate 迁移 EvidenceAppender 或桥接 | 编译+测 | T-MEM-002 | Gate | TODO |
| T-GATE-002 | 去掉 hash_bytes(name) 裸用 | domain digest | T-GATE-001 | Gate | TODO |
| T-POL-002 | required operation 登记 | **I-13 全部键** schema 校验 | T-DOM-004 | Policy | TODO |
| T-LEG-001 | Legacy bridge 设计落地 | 不伪造 V1 连续；migration record | T-CORE-025 | Core | TODO |
| T-LEG-002 | 旧六字段只读 verifier 保留策略 | 文档+代码路径 | T-LEG-001 | Core | TODO |
| T-LEG-003 | 双包同名隔离 | rename `evidence_legacy` 或等价；workspace 无双 `name=evidence` | T-CORE-001 | Lead | TODO |
| T-ATOM-001 | 模式 A：业务+evidence 同事务证明 | 文档+pg 测 | T-PG-003 | Atom | TODO |
| T-ATOM-002 | 模式 B：业务+outbox 同事务 | 禁 evidence durable 前删 outbox | T-PG-004 | Atom | TODO |
| T-ATOM-003 | 模式 C：evidence 为 SoT 登记 | policy 可选 | T-POL-002 | Atom | TODO |
| T-ATOM-004 | 外部不可逆：Attempted+terminal | 共享 OperationId；不同 EventId；**订单域可 DEFER(accepted) 但须登记** | T-CORE-010 | Atom | TODO |
| T-ATOM-005 | 纯内存状态四步模式 | 第4步可失败则禁用 | T-MEM-002 | Atom | TODO |
| T-ATOM-006 | Rejected 路径合同 | 仅 append 成功后返回业务拒绝 | T-DOM-005 | Atom | TODO |
| T-PRIV-001 | artifact store 合同（content-addressed） | key=digest；非 core 实现 | T-POL-001 | Privacy | TODO |
| T-PRIV-002 | retention 六类策略字段 | I-16 | T-POL-002 | Privacy | TODO |
| T-PRIV-003 | erasure：保留摘要+deletion evidence；不改历史 | 测+文档 | T-PRIV-002 | Privacy | TODO |

---

## W4 — File + Postgres adapters

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-FILE-001 | package evidence_file | 路径 `crates/adapters/evidence/file`（新建 adapters/evidence 树，非 storage） | T-CORE-023 | File | TODO |
| T-FILE-002 | segment header/frames/footer | 双 u32 len + commit marker；footer seal | T-FILE-001 | File | TODO |
| T-FILE-003 | 单写者 lock（非仅 Mutex） | 跨进程；第二 writer 失败 | T-FILE-001 | File | TODO |
| T-FILE-004 | Durable fsync 合同 | write→fdatasync→目录元数据→更新 head | T-FILE-002 | File | TODO |
| T-FILE-005 | 崩溃恢复 10 步 | **I-17 全步**；仅截断未提交尾帧 | T-FILE-002 | File | TODO |
| T-FILE-006 | 已提交损坏 → quarantine | 禁写 | T-FILE-005 | File | TODO |
| T-FILE-007 | segment rotation 64MiB/1e6 | 跨 segment 连续 | T-FILE-002 | File | TODO |
| T-FILE-008 | crash/fault 注入 | kill-9 四时机+disk full+short write+fsync err+corrupt | T-FILE-005 | File | TODO |
| T-FILE-009 | conformance suite PASS | 与 mem 同套 | T-FILE-004 T-MEM-008 | File | TODO |
| T-FILE-010 | group commit 语义 | Durable 调用方等待 group fsync | T-FILE-004 | File | TODO |
| T-PG-001 | package evidence_postgres | `crates/adapters/evidence/postgres` | T-CORE-023 | Pg | TODO |
| T-PG-002 | 表 heads/records/outbox/checkpoints | 唯一 (chain,seq)(chain,event)；head 列齐全 | T-PG-001 | Pg | TODO |
| T-PG-003 | 直接 append 事务 | FOR UPDATE；失败整体回滚 | T-PG-002 | Pg | TODO |
| T-PG-004 | outbox 同事务业务插入 | 禁提前删 outbox | T-PG-002 | Pg | TODO |
| T-PG-005 | dispatcher 幂等 | 崩溃重试不重复 | T-PG-004 | Pg | TODO |
| T-PG-006 | 并发/死锁/回滚/断连/expected_head 竞争 | §21.5 / §24.7 | T-PG-003 | Pg | TODO |
| T-PG-007 | conformance suite PASS | | T-PG-003 T-MEM-008 | Pg | TODO |

---

## W5 — Checkpoint + CLI + 观测/性能

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-CP-001 | CheckpointSigner/Verifier traits | adapter 合同；私钥不进 core | T-CORE-024 | CP | TODO |
| T-CP-002 | Ed25519 adapter（或测试密钥） | 私钥不进仓库/镜像 | T-CP-001 | CP | TODO |
| T-CP-003 | 频率策略钩子 | 10k 或 60s；超 hard deadline 阻写 | T-CP-001 | CP | TODO |
| T-CP-004 | TailTruncated 检测 | 本地短于可信 checkpoint | T-CP-001 T-CORE-025 | CP | TODO |
| T-CP-005 | independent anchor 合同接口 | OSS Object Lock / 独立库；本地盘不算 | T-CP-001 | CP | TODO |
| T-CP-006 | key rotation 测试 | 旧 key 验证；未知 key ID；无效 sig | T-CP-002 | CP | TODO |
| T-CP-007 | 整链替换检测 | anchor/checkpoint 与本地 head 不一致 → 失败 | T-CP-005 T-CORE-025 | CP | TODO |
| T-CP-008 | startup verify 合同 | 启动对照 checkpoint 再开放写 | T-FILE-005 T-CP-004 | CP | TODO |
| T-CLI-001 | tools/evidence-cli scaffold | member | T-CORE-025 | CLI | TODO |
| T-CLI-002 | verify/inspect/head/export | I-10 默认行为全 AC | T-CLI-001 | CLI | TODO |
| T-CLI-003 | checkpoint verify | | T-CLI-001 T-CP-002 | CLI | TODO |
| T-CLI-004 | vectors verify | 复算 I-6 全部 | T-CLI-001 T-CORE-027 | CLI | TODO |
| T-CLI-005 | repair-tail | I-10 repair AC 全项 | T-CLI-001 T-FILE-005 | CLI | TODO |
| T-CLI-006 | 退出码 0/2/3/4/5/6/7 | I-10 语义表测 | T-CLI-002 | CLI | TODO |
| T-CLI-007 | vectors generate | 生成/比对 golden（非默认写链） | T-CLI-001 T-CORE-026 | CLI | TODO |
| T-OBS-001 | metrics 11 名注册清单 | **I-11 全名**；无敏感高基数 label | T-MEM-002 | Obs | TODO |
| T-PERF-001 | core seal/verify 基准骨架 | §29.1 | T-CORE-025 | Perf | TODO |
| T-PERF-002 | adapter 基准契约文档 | p50/p95/p99 等；不得降 durability | T-FILE-004 | Perf | TODO |
| T-BP-001 | 背压 fail-closed | I-22；不静默 Volatile | T-CORE-022 T-FILE-004 | Perf | TODO |
| T-SCH-001 | V1 冻结声明 | golden 后字段/tag/序不可变 | T-CORE-027 | Doc | TODO |
| T-SCH-002 | Reader 多版本兼容策略 | 保留期内可读旧版 | T-SCH-001 | Core | TODO |
| T-SCH-003 | 算法迁移须 V2/新 tag | 禁原地改 V1；双锚定 checkpoint | T-SCH-001 T-CP-001 | Core | TODO |

---

## W6 — 门禁 / CI / Cutover

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-ARCH-001 | EVIDENCE-PATH-001 | tools/ runtime package → fail（cutover 后强制；迁移期 warn 可记录） | T-CUT-002 | Arch | TODO |
| T-ARCH-002 | EVIDENCE-DEP-001/002 | I-12 白名单 | T-CORE-002 | Arch | TODO |
| T-ARCH-003 | EVIDENCE-ANYHOW-001 | 公开面无 anyhow | T-CORE-002 | Arch | TODO |
| T-ARCH-004 | DOMAIN/DEBUG/JSON/GENESIS/PUBLIC | 五门禁 | T-CORE-018 | Arch | TODO |
| T-ARCH-005 | EVIDENCE-MEMORY-PROD-001 | release 图无 memory | T-MEM-007 | Arch | TODO |
| T-ARCH-006 | POLICY/COVERAGE/CHECKPOINT | 系统门禁子集 | T-POL-002 T-CP-003 | Arch | TODO |
| T-ARCH-010 | EVIDENCE-CANONICAL-001 | 非 V1 hash → fail | T-CORE-014 | Arch | TODO |
| T-ARCH-011 | EVIDENCE-DURABILITY-001 | required 禁 Volatile/Process；机控与 T-BOOT-001 策略一致 | T-BOOT-001 | Arch | TODO |
| T-ARCH-012 | EVIDENCE-IDEMPOTENCY-001 | conformance 证据 | T-MEM-008 | Arch | TODO |
| T-ARCH-013 | EVIDENCE-CONCURRENCY-001 | concurrent suite 证据 | T-MEM-009 | Arch | TODO |
| T-ARCH-014 | EVIDENCE-RECOVERY-001 | crash recovery evidence | T-FILE-008 | Arch | TODO |
| T-ARCH-015 | EVIDENCE-FSYNC-001 | file fsync 合同 | T-FILE-004 | Arch | TODO |
| T-ARCH-016 | EVIDENCE-ATOMICITY-001 | Tier-A 原子性证明 | T-ATOM-001 T-ATOM-002 | Arch | TODO |
| T-ARCH-017 | EVIDENCE-ANCHOR-001 | 独立 anchor | T-CP-005 | Arch | TODO |
| T-ARCH-018 | EVIDENCE-SCHEMA-001 | schema 须新版本 | T-SCH-001 | Arch | TODO |
| T-ARCH-019 | EVIDENCE-VECTOR-001 | golden 漂移需 RFC | T-CORE-026 | Arch | TODO |
| T-BOOT-001 | bootstrap 强制生产 adapter | 禁 memory；required→Durable；依赖 T-MEM-007 文档标记 | T-MEM-007 | Boot | TODO |
| T-BOOT-002 | checkpoint hard deadline 阻写 | 与 T-CP-003 联动 | T-CP-003 | Boot | TODO |
| T-CI-001 | PR CI：fmt/clippy/test/lint-deps/archgate | I-20 日间命令 | T-CORE-027 T-MEM-008 | CI | TODO |
| T-CI-003 | coverage job line≥95% **且 branch≥90%** core | llvm-cov 双阈值；不足 FAIL | T-CORE-033 | CI | TODO |
| T-CI-004 | crate-standard --check | xtask | T-CORE-032 | CI | TODO |
| T-MUT-001 | mutants + I-7 kill list | score≥90；M-KILL-01..10 不存活 | T-CORE-027 | Quality | TODO |
| T-MIRI-001 | cargo miri test -p evidence（+memory） | 定期绿 | T-CORE-027 T-MEM-002 | Quality | TODO |
| T-FUZZ-001 | fuzz 五目标 I-8 + corpus 沉淀 | 发现即 regression | T-CORE-015 T-FILE-002 | Quality | TODO |
| T-CI-NIGHTLY-001 | Nightly：mutation/fuzz/miri/chaos/keyrot/schema/**branch 复测** | **正式任务**；未接线=BLOCKED≠PASS；branch≥90% 复验 | T-MUT-001 T-FUZZ-001 T-MIRI-001 T-CI-003 | CI | TODO |
| T-SUBJ-001 | Subject 规范化策略版本化文档 | 领域层规则索引；record 仅 subject_digest | T-CORE-010 T-POL-002 | Domain | TODO |
| T-MEM-PROD-SYS | systemd/release 清单禁 evidence_memory | 与 T-ARCH-005/T-BOOT-001 联检 | T-BOOT-001 | Boot | TODO |
| T-CUT-001 | 调用方全部迁离 legacy | 无 workspace 依赖旧包 | T-DOM-001 T-GATE-001 T-LEG-003 | Lead | TODO |
| T-CUT-002 | 删除 tools/evidence（或 legacy） | PATH-001 绿 | T-CUT-001 T-BOOT-001 | Lead | TODO |
| T-CUT-003 | 更新 docs/architecture/spec.md 路径 | crates/evidence（或人审选定路径） | T-CUT-002 T-DOC-005 | Doc | TODO |
| T-CUT-004 | 删除旧 EvidenceSink/hash_bytes/mock | 全树无引用 | T-CUT-002 | Lead | TODO |
| T-DOC-004 | ADR-010 修订提案 | 与 002 对齐 | T-CORE-025 | Doc | TODO |
| T-REG-001 | architecture registry 对齐草案 | 待人审 | T-CUT-003 | Doc | TODO |

---

## W7 — 十轮实现验收

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-V10-R01 … T-V10-R10 | 各跑完整 R-checklist + 机器命令 | 每轮独立 verdict | W1–W6 关键任务 | Verifier | TODO |
| T-V10-SUM | 汇总 fail_rounds=0 | evidence/system/…/verdict.md | T-V10-R01..R10 | Verifier | TODO |
| T-SKEP-001 | Skeptic 反证假 PASS | 无矛盾勾选 | T-V10-SUM | Skeptic | TODO |

---

## W8–W9 — 人审 / 版本 / §33

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-HUM-001 | Spec Status → Approved | 人签字 | approval-packet | ZoneCNH | **DONE**（2026-07-14 会话「授权审批」· APPR-EVIDENCE-002-v1） |
| T-HUM-002 | 旧 spec superseded 正式 | | T-HUM-001 | Human | TODO |
| T-VER-001 | version 0.1.1 bump | scripts/version.mjs | T-HUM-001 T-V10-SUM | Release | TODO |
| T-REG-002 | registry stable 决策 | 可 DEFER | T-HUM-001 | Human | TODO |
| T-33-001 | §33.1–33.6 证据勾选 | 全 PASS 或正式 DEFER | T-HUM-001 T-V10-SUM | Owner | TODO |
| T-EVID-SYS | 战役 Evidence 包归档 | §32 目录完整 | T-V10-SUM | Verifier | TODO |

---

## §33 勾选 → Task 映射（防遗漏）

### 33.1 规格闭合

| 项 | Task |
|----|------|
| SPEC Approved | T-HUM-001 |
| 旧 spec superseded | T-DOC-002 T-HUM-002 |
| ADR 冲突修订 | T-DOC-004 T-DOC-005 |
| 路径 package 对齐 | T-CUT-002 T-CUT-003 T-LEG-003 |
| architecture registry | T-REG-001 T-REG-002 |
| evidence-policy.toml | T-POL-001 T-POL-002 |
| 无未登记 Unknown | T-RES-001 T-SKEP-001 |

### 33.2 Core 闭合

| 项 | Task |
|----|------|
| crates/evidence | T-CORE-001 |
| canonical V1 冻结 | T-CORE-014..027 |
| 无字段拼接歧义 | T-CORE-029 |
| 无全零 genesis | T-CORE-017 |
| ChainId/sequence/EventId | T-CORE-005/006 T-MEM-002 |
| 时间分离 | T-CORE-011 |
| actor/subject | T-CORE-008 T-CORE-010 |
| typed outcome | T-CORE-009 |
| 无 hash_bytes | T-CORE-018 T-CUT-004 |
| 无 Debug/JSON hash | T-DOM-001 T-ARCH-004 |
| 无 anyhow | T-CORE-002 T-ARCH-003 |
| 字段私有 | T-CORE-012 T-CORE-035 |

### 33.3 Adapter 闭合

| 项 | Task |
|----|------|
| memory 仅测试 | T-MEM-004/007 T-ARCH-005 |
| file durable | T-FILE-004 |
| postgres/outbox | T-PG-003/004 |
| 并发 conformance | T-MEM-005 T-FILE-009 T-PG-007 |
| idempotency | T-MEM-002 T-PG-005 |
| crash recovery | T-FILE-005/008 T-PG-006 |
| disk/fsync 故障 | T-FILE-008 |
| 不降级 volatile | T-MEM-004 T-ARCH-005 |

### 33.4 Checkpoint 闭合

| 项 | Task |
|----|------|
| signed checkpoint | T-CP-002 |
| key rotation | T-CP-006 |
| independent anchor | T-CP-005 |
| tail truncation | T-CP-004 |
| full replacement 检测 | T-CP-007 |
| startup verify | T-CP-008 |

### 33.5 测试闭合

| 项 | Task |
|----|------|
| golden | T-CORE-026/027 T-CLI-004 |
| property | T-CORE-028/030 |
| fuzz | T-FUZZ-001 |
| line≥95% | T-CORE-033 T-CI-003 |
| branch≥90% | T-CI-003 T-CI-NIGHTLY-001（AC：branch coverage≥90% core） |
| mutants≥90% | T-MUT-001 |
| Miri | T-MIRI-001 |
| adapter chaos | T-FILE-008 T-PG-006 |
| historical schema | T-LEG-002 T-SCH-002 T-CI-NIGHTLY-001 |

### 33.6 系统闭合

| 项 | Task |
|----|------|
| required ops 登记 | T-POL-002 |
| fail-closed | T-DOM-005 |
| Tier-A 原子性 | T-ATOM-001 T-ATOM-002 T-ARCH-016 |
| external Attempted+terminal | T-ATOM-004（可 DEFER accepted） |
| source artifacts retention | T-PRIV-001 T-PRIV-002 |
| verifier/schema/keys 保留 | T-PRIV-002 T-CP-006 T-SCH-002 |
| CI Evidence 可追溯 | T-EVID-SYS T-CI-001 |

---

## 统计（v1.1 补全后）

| Wave | Task 数（约） | 当前 DONE |
|------|---------------|-----------|
| W0 | 16 | 6（plan+inventory） |
| W1 | 38 | 0 |
| W2 | 10 | 0 |
| W3 | 22（含 ATOM/PRIV/LEG） | 0 |
| W4 | 18 | 0 |
| W5 | 24 | 0 |
| W6 | 30+ | 0 |
| W7 | 12 | 0 |
| W8–W9 | 6 | 0 |
| **合计** | **~170+** | **6** |

> v1.1：消灭幽灵 `T-ATOM via design`；补 I-* 引用；全量 EVIDENCE-*；拆 CI 草案桶；双包隔离；ADR-012 对账。
