# Review: evidence v0.1.1 — 2026-07-22

| 字段 | 值 |
|------|-----|
| 目标 crate | `evidence` → `xhyper-evidence` |
| 路径 | `crates/evidence` |
| 层级 | L1 |
| 审查日期 | 2026-07-22 |
| 审查者 | AI Agent（review-prompt v1.0 验证执行） |
| 前置依赖 | `kernel`（间接消费者）、`bootstrap`（注入消费者） |
| SSOT | `.agents/ssot/tools/evidence/` |
| 对齐文档 | `docs/ssot/evidence-ssot-alignment.md` |

## 1. 概览

evidence crate 提供一个简洁、专注的审计证据追加面，包含内存和文件两种后端实现、HMAC-SHA256 签名、远程传输抽象和查询 API。代码质量良好（`forbid(unsafe_code)`、`deny(missing_docs)`、零 test/clippy 告警、追加性能 ~469ns/op）。

主要缺口：(1) 错误类型仅 2 变体、使用英文 Display、无 `XError` 映射；(2) `expect()` 在 6 处锁调用中可能 panic；(3) SSOT 战役 PLAN-EVIDENCE-002 中的 W0–W8 任务仅 6/16 完成，spec/design/goal 等目录仍为占位；(4) 缺少 serde、fuzz、属性测试。整体属于「结构完成度高、生产补强中」状态。

## 2. 通用维度评估

| 维度 | 评分 | 说明 |
|------|------|------|
| D1. 公开 API 正确性 | 4/5 | API 设计清晰；`FileEvidenceAppender::open()` 中 `unwrap_or_default()` 静默吞 IO 错误 |
| D2. 类型与不变量 | 5/5 | `forbid(unsafe_code)`；`next_seq` 用 `saturating_add`；`[u8; 32]` 精确签名类型 |
| D3. 错误处理 | 2/5 | 仅 2 个变体；英文 Display（违���中文约定）；无 `XError`/`ErrorKind` 映射；无 `source()` 链 |
| D4. 并发安全 | 3/5 | `Send + Sync` 正确；但 6 处 `expect("lock")` 在 poison 时 panic；`inner_lock()` 返回 `Result` 但其他方法不统一 |
| D5. Trait 设计 | 4/5 | 三个 trait 均为对象安全；`EvidenceQuery` 仅为 `InMemoryEvidenceAppender` 实现（文件后端缺查询） |
| D6. 依赖与版本 | 5/5 | 仅 `sha2 = { workspace = true }`；零多余依赖；workspace deps 门禁通过 |
| D7. SSOT 对齐 | 2/5 | 对齐文档 PASS；但 SSOT 战役 PLAN 中仅 6/16 任务完成；goal/design/gate 等为占位；不可变规约为 Draft |
| D8. 测试覆盖 | 3/5 | 17 单元 + 3 集成 + 1 surface + 1 bench + 2 examples；缺少 serde 测试、fuzz、proptest 属性测试 |
| D9. 可观测性 | 2/5 | 零 tracing/log 调用；审计模块可以接受，但生产使用缺少操作信号 |
| **Σ** | **30/45** | |

## 3. 分层专项评估

| 检查项 | 状态 | 备注 |
|--------|------|------|
| 仅 `sha2` 生产依赖 | ✅ | 零多余依赖 |
| HMAC key 由调用方注入 | ✅ | `sign_evidence(key, seq, name)` — 正确 |
| 追加不可变 | ✅ | 两种后端均只追加不删除 |
| `forbid(unsafe_code)` | ✅ | `lib.rs` 顶部声明 |
| `deny(missing_docs)` | ✅ | 所有公开项均有文档 |
| 错误链到 `XError` | ❌ | 自持 `EvidenceError`，无 `From` 实现 |
| 用户可见错误中文 | ❌ | 英文：`"evidence durability failure"` / `"evidence backend unavailable"` |

## 4. 发现明细

### P0：阻塞性缺陷

| # | 文件:行号 | 问题描述 | 类别 | 修复建议 |
|---|----------|---------|------|---------|
| — | — | 未发现 P0 问题 | | |

### P1：重要问题

| # | 文件:行号 | 问题描述 | 类别 | 修复建议 |
|---|----------|---------|------|---------|
| 1 | `src/lib.rs:132,136,141,146`；`src/remote.rs:43,49` | 6 处 `expect("lock")` 在 Mutex poison 时直接 panic | 正确性 | 改用 `lock().map_err(\|_\| EvidenceError::Unavailable)?` 或统一使用 `inner_lock()` |
| 2 | `src/lib.rs` | `EvidenceError` 无 `XError`/`ErrorKind` 映射，消费者无法按 `kernel` 错误分类处理 | 架构 | 增加 `From<EvidenceError>` 转为 `XError`（`DurabilityFailure` → `Transient`，`Unavailable` → `Unavailable`） |
| 3 | `src/lib.rs:132,136,141,146`、`src/remote.rs:43,49` | `InMemoryEvidenceAppender` 的 `fail_next()`/`close()`/`names()`/`len()` 使用 `expect()` 而非返回 `Result` | API 健壮性 | 改为返回 `Result` 或至少用 `map_err` 防 poison panic |

### P2：建议改进

| # | 文件:行号 | 问题描述 | 类别 | 修复建议 |
|---|----------|---------|------|---------|
| 4 | `src/lib.rs` | `EvidenceError` Display 使用英文，违反治理文档中文约定 | 约定 | 改为中文：「持久化失败」「后端不可用」 |
| 5 | `src/lib.rs:81-82` | `FileEvidenceAppender::open()` 中 `read_to_string` 用 `unwrap_or_default()` — 文件存在但不可读时静默吞错误 | 正确性 | 区分「文件不存在」和「不可读」，前者用空内容继续，后者返回错误 |
| 6 | `src/remote.rs` | `RemoteEvidenceAppender` 传输失败时本地已占序号，导致本地与远程 seq 不一致 | 设计 | 文档化「at-most-once」语义，或改为先传输后分配序号 |
| 7 | SSOT 全局 | PLAN-EVIDENCE-002 中 W0–W8 仅 6/16 任务完成；goal/design/gate/prompt/release/review/retrospective 均为占位 | 治理 | 对齐 SSOT 占位与实际交付的关系，明确哪些战役不计划启动 |

### P3：代码风格/微优化

| # | 文件:行号 | 问题描述 | 类别 | 修复建议 |
|---|----------|---------|------|---------|
| 8 | `src/sign.rs:13` | `SignedEvidence` 的所有字段均为 `pub`，缺少封装 | 封装 | 考虑字段私有化并提供访问器，保持与 `AppendReceipt` 一致 |
| 9 | `src/lib.rs` | `inspect.rs` 的 `seq_is_monotonic` 和 `event_count` 在 `InMemory` 上已有等效功能，但对 `FileEvidenceAppender` 无直接使用路径 | API 一致性 | 为 `FileEvidenceAppender` 暴露 inspect 接口或标记为 `pub(crate)` |

## 5. SSOT 对齐状态

| 规格条目 | 实现状态 | 对齐结论 | Gap 说明 |
|---------|---------|---------|---------|
| `EvidenceAppender` trait | 已实现 | ✅ PASS | `append_named` 及其语义已完整 |
| `EvidenceError` 变体 | 已实现（2 个） | ✅ PASS | 但缺失 kernel 映射 |
| `AppendReceipt` 类型 | 已实现 | ✅ PASS | `name` + `seq` |
| `InMemoryEvidenceAppender` | 已实现 | ✅ PASS | fail_next/close/names/len |
| `FileEvidenceAppender` | 已实现 | ✅ PASS | open/append/read_entries/path |
| `EvidenceQuery` trait + impl | 已实现 | ✅ PASS | query_by_name/query_range/list_all |
| HMAC-SHA256 签名/校验 | 已实现 | ✅ PASS | sign_verify/verify_evidence |
| `EvidenceTransport` + Mock | 已实现 | ✅ PASS | FnTransport/MockTransport/RemoteAppender |
| Goal/Spec/Design 战役 | 占位 | ⚠️ DEFER | SSOT 中 goal/design/gate 等目录未填充战役内容 |
| 不可变规约 CI 接入 | Draft | ⚠️ DEFER | SPEC-EVIDENCE-IMMUTABILITY-DRAFT-001 未激活 |

## 6. 质量门禁结果

| 门禁项 | 状态 | 备注 |
|--------|------|------|
| `cargo build`（workspace 上下文） | ✅ | 编译通过 |
| `cargo test -p evidence --all-targets` | ✅ | 21 测试全部通过（17 单元 + 3 集成 + 1 surface） |
| `cargo fmt --check` | ✅ | 格式合规 |
| `cargo clippy -p evidence -D warnings` | ✅ | 零警告 |
| benchmark `hot_path` | ✅ | ~469ns/iter（100k iters） |
| `check-workspace-deps.mjs` | ✅ | evidence 无内联版本违规 |

## 7. 生产就绪判定

| 维度 | 判定 |
|------|------|
| L 层 | L1 Internal Ready（**有条件**）— 锁 poison 安全问题和错误中文需修复 |
| S 完整性 | 25/35（继承基线；SSOT 占位拖累 S1/S4/S6） |
| QT 场景 | QT-4（持久化与审计）：**Conditional** — 仅内存/文件追加签名路径，非合规审计 |
| 整体 Go/No-Go | **有条件 GO**（内部库语义；修复 P1#1-#3 后可升格） |
| 阻塞项 | P1：expect panic x6、错误中文、XError 映射 |
| 评估依据 | 本审查遵照 [review-prompt.md](./review-prompt.md) 逐项执行 |

## 8. 综合建议

1. **短期（P1 修复）**：统一 Mutex 锁的错误处理路径（改用 `map_err` 或 `inner_lock`）；`EvidenceError` Display 中文化；增加 `From<EvidenceError>` → `XError` 映射。
2. **中期（P2 改进）**：`FileEvidenceAppender` 的 IO 错误区分；`RemoteEvidenceAppender` 的传输语义文档化；SSOT 战���占位声明。
3. **长期（测试补强）**：增加 fuzz 入口、proptest 属性测试（签名/解析 roundtrip）、serde 测试（如为 `SignedEvidence` 增加 serde）。

## 9. 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-22 | 初版：review-prompt v1.0 可用性验证审查 |

> **声明**：本审查为 AI 辅助代码审查（遵循 review-prompt.md v1.0），不替代 Maintainer 人类签核与安全审计。审查结论仅代表代码基线在审查时刻的快照分析。
