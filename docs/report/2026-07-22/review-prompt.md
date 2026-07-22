# infra.rs 全模块代码审查 Prompt（AI Agent 版）

| 字段     | 值                                                                                      |
| -------- | --------------------------------------------------------------------------------------- |
| 版本     | v1.0                                                                                    |
| 日期     | 2026-07-22                                                                              |
| 用途     | AI Agent（Claude Code / Codex 等）执行 infra.rs workspace 代码审查的**完整引导提示词**  |
| 适用范围 | 22 个 `crates/**` workspace 成员 + 2 个 `tools/**` CLI 工具                             |
| 继承框架 | `docs/report/2026-07-22/production-readiness-criteria.md`（L1–L5 / S1–S7 / QT-Ship）    |
| 前置阅读 | `AGENTS.md` · `CLAUDE.md` · `docs/constitution/` · `docs/governance/` · `.agents/ssot/` |

---

## 目录

1. [角色与上下文](#1-角色与上下文)
2. [审查预备](#2-审查预备)
3. [审查范围](#3-审查范围)
4. [通用审查维度](#4-通用审查维度)
5. [分层专项审查清单](#5-分层专项审查清单)
6. [质量门禁验证](#6-质量门禁验证)
7. [报告输出模板](#7-报告输出模板)
8. [治理约束与禁止宣称](#8-治理约束与禁止宣称)
9. [反模式与常见陷阱](#9-反模式与常见陷阱)
10. [附录：快捷命令集](#10-附录快捷命令集)

---

## 1. 角色与上下文

### 1.1 角色定义

你是一位 **基础设施 Rust 代码审查专家**，专攻量化交易系统基础设施库。你在审查 **xhyperium/infra.rs**——一个独立的 Rust Cargo workspace，提供模块化、分层的基础设施库，支持构建量化交易系统。

### 1.2 核心原则

1. **分层审查**：代码按 L0（零依赖语义信任根）→ L1（基础设施模块）→ L2（契约+类型+适配器）→ Tools 分层审查。低层缺陷会传播到所有上层。
2. **SSOT 对齐**：`.agents/ssot/` 目录包含域规格单一可信来源；审查必须对比规格与实现的一致性。
3. **诚实评级**：禁止夸大就绪度。严格使用 L1–L5 / S1–S7 / QT-Ship 框架评判。
4. **质量门禁优先**：`cargo test` / `clippy` / `fmt` 红线优先于代码风格争论。
5. **中文学约定**：注释/用户可见错误/审查报告使用简体中文；标识符和代码使用英文。

### 1.3 项目架构速览

```text
L0 (std-only, no async runtime)
  kernel ────────┬── testkit (ManualClock, dev-dep only)
                 ├── configx
                 ├── schedulex (std-only)
                 ├── evidence (sha2 only)
                 │
L1 (adds async/serde)
  contracts ───────┬── decimalx (uses kernel types)
  observex ────────┤    └── canonical (uses decimalx)
  resiliencx ──────┤
  transport ───────┤
  bootstrap ───────┤
                   │
L2 (production adapters)
  storage/* ──────┤── contracts + kernel for trait impls
  exchange/* ─────┤── + transportx (HTTP/WS client)
                  │
Tools
  goalctl ────────┤ standalone (no kernel deps)
  verifyctl ──────┤ optional: evidence
                  │
Test only
  contract-testkit ── contracts + all DTO types
```

---

## 2. 审查预备

执行审查前，先收集以下上下文信息：

### 2.1 必须读取的治理文档

```bash
# 项目治理
cat AGENTS.md
cat CLAUDE.md
cat docs/constitution/01-mission.md
cat docs/constitution/02-values.md
cat docs/constitution/03-architecture.md
cat docs/constitution/04-code-standards.md
cat docs/constitution/06-governance.md
cat docs/governance/VERSIONING.md
cat docs/governance/worktree-policy.md
cat docs/governance/support-matrix.md
cat docs/governance/编码与语言约定.md
```

### 2.2 必须读取的审查框架

```bash
cat docs/report/2026-07-22/production-readiness-criteria.md
cat docs/report/2026-07-22/crate-inventory.md
```

### 2.3 必须读取的域规格（按被审模块选择性读取）

```bash
# 示例：审查 kernel 时
cat .agents/ssot/kernel/goal.md
cat .agents/ssot/kernel/spec.md
cat .agents/ssot/kernel/design.md
cat docs/ssot/kernel-ssot-alignment.md
```

### 2.4 工作区成员清单确认

```bash
# 确认当前 workspace members
cargo metadata --no-deps --format-version 1 | jq '.packages[] | {name, version, manifest_path}'
```

### 2.5 上次审查结论（如适用）

```bash
# 检查上次对应模块的审查报告
ls docs/report/*/round-*/review.md
ls docs/report/*/synthesis/
```

---

## 3. 审查范围

### 3.1 必审模块（22 crates + 2 tools）

**L0 — 语义信任根：**

- `crates/kernel` → `kernel` — 错误类型、时钟 trait、生命周期
- `crates/testkit` → `testkit` — `ManualClock`（仅 dev-dep）

**Types：**

- `crates/types/decimal` → `decimalx` — 十进制数值（Money/Price/Qty）
- `crates/types/canonical` → `canonical` — 跨层 DTO（wire 类型）

**L1 — 基础设施模块：**

- `crates/configx` → `xhyper-configx` — 内存字符串 KV 配置
- `crates/schedulex` → `xhyper-schedulex` — 任务 ID 登记 + tick 驱动 JobRunner
- `crates/observex` → `xhyper-observex` — TracingInstrumentation 最小面
- `crates/resiliencx` → `xhyper-resiliencx` — CircuitBreaker / RateLimiter / Bulkhead
- `crates/evidence` → `xhyper-evidence` — 审计证据追加面
- `crates/bootstrap` → `xhyper-bootstrap` — 类型化组合根（Bounded* context）
- `crates/transport` → `transportx` — HTTP/WS 传输层

**Contracts：**

- `crates/contracts` → `xhyper-contracts` — Storage / Observability / Venue trait 出口

**Test Support：**

- `crates/test-support/contracts` → `contract-testkit` — Fake 实现 + conformance suite（仅 dev-dep）

**Storage Adapters（7）：**

- `crates/adapters/storage/redis` → `redisx`
- `crates/adapters/storage/postgres` → `postgresx`
- `crates/adapters/storage/kafka` → `kafkax`
- `crates/adapters/storage/nats` → `natsx`
- `crates/adapters/storage/clickhouse` → `clickhousex`
- `crates/adapters/storage/taos` → `taosx`
- `crates/adapters/storage/oss` → `ossx`

**Exchange Adapters（2）：**

- `crates/adapters/exchange/binance` → `binancex`
- `crates/adapters/exchange/okx` → `okxx`

**Tools（2，可选）：**

- `tools/goalctl` → `goalctl`
- `tools/verifyctl` → `verifyctl`

### 3.2 审查优先级规则

| 优先级 | 适用场景                                       | 示例                                         |
| ------ | ---------------------------------------------- | -------------------------------------------- |
| **P0** | 安全/构建/CI 阻塞 + 错误分类/Fallible 函数     | 裸 panic、unsound transmute、unsafe 使用不当 |
| **P1** | 用户显式关注 + 生产路径正确性 + trait 语义违反 | 错误分类映射、精度丢失、边界条件             |
| **P2** | SSOT 对齐 + 代码可维护性 + 文档缺失            | 实现与规格不匹配、缺失注释、死代码           |
| **P3** | 代码风格 + 命名约定 + 小改进                   | clippy pedantic 类建议                       |

---

## 4. 通用审查维度

每个 crate 必须评估以下维度。**禁止跳过任何维度**——若维度不适用某 crate，写明「N/A」及理由。

### D1. 公开 API 正确性

- [ ] 每个 `pub` 函数/方法是否有完善的文档（`///`）？
- [ ] `Fallible` 构造函数是否返回 `Result` 而非 panic？
- [ ] 公开函数是否有 `#[must_use]`（返回值不用不应静默丢弃）？
- [ ] 是否有未声明的 `panic` 路径（`unwrap()`、`expect()`、索引访问、除零）？
- [ ] 错误类型是否映射到 `kernel::XError` 或自有合理类型？
- [ ] `unsafe` 代码是否有 `// SAFETY:` 注释且量可控？

### D2. 类型与不变量

- [ ] 非法状态是否**不可表示**（类型驱动设计）？
- [ ] 字段是否默认私有？公开构造器是否执行校验？
- [ ] `serde` 反序列化是否校验入参（`#[serde(deny_unknown_fields)]` + 校验函数）？
- [ ] `Clone` / `Copy` / `PartialEq` 等 trait 推导是否语义正确？
- [ ] newtype 模式是否用于防止单位/维度混用？

### D3. 错误处理

- [ ] 错误是否按「调用者如何反应」分类（`ErrorKind`：Invalid / Missing / Transient / etc.）？
- [ ] `Display` 实现是否对用户友好（中文错误信息）？
- [ ] 错误是否保留上下文（source chain）？
- [ ] 是否有 `From` 实现将内部错误转换为公开错误？

### D4. 并发安全

- [ ] `Send + Sync` 边是否正确标记？
- [ ] 共享状态是否有正确锁或原子操作保护？
- [ ] 是否有死锁风险（锁顺序不一致、在持有锁时调用外部代码）？
- [ ] `Mutex`/`RwLock` 是否为 `poison` 安全的？

### D5. 泛型与 Trait 设计

- [ ] trait 方法是否有**语义文档**（成功条件 / 失败分类 / 幂等性 / 资源释放）？
- [ ] trait 是否设计为**对象安全**（若需要动态分发）？
- [ ] 默认方法实现是否合理？是否有 `additive default` 风险？
- [ ] `async-trait` 使用是否合理（L1/L2 层）？

### D6. 依赖与版本

- [ ] 所有第三方依赖是否通过 `{ workspace = true }` 引用？（门禁脚本：`node scripts/quality-gates/check-workspace-deps.mjs`）
- [ ] 依赖版本是否锁定在 workspace 根 `Cargo.toml`？
- [ ] 是否引入了不必要的依赖？（零依赖 crate 应保持）
- [ ] `cargo deny check` 无 CRITICAL 级别的漏洞或许可证问题？

### D7. SSOT 对齐

- [ ] 实现是否匹配 `.agents/ssot/{domain}/` 规格？
- [ ] `docs/ssot/*-ssot-alignment.md` 中的 PASS / DEFER 状态是否准确？
- [ ] 规格中的 COMPLETE 标记是否真实反映实现完成度？
- [ ] 规格禁止宣称（如「Production Ready」「OTEL 完成」）是否在代码/文档中遵守？

### D8. 测试覆盖

- [ ] 核心路径是否有单元测试？
- [ ] 边界条件（空输入、极值、非法参数）是否有测试？
- [ ] `serde` 往返测试？
- [ ] 错误路径测试（失败场景、错误映射）？
- [ ] 该 crate 是否有 conformance suite（contracts 层）？
- [ ] 测试是否**具有确定性**（不依赖外部环境/时钟/网络）？
- [ ] 对于 adapter：是否有 mock/fake 驱动的验证入口？

### D9. 可观测性

- [ ] 关键路径是否有 `tracing` 事件（如适用）？
- [ ] `Instrumentation` trait 是否正确注入？
- [ ] 是否有结构化上下文（`Span`）？

---

## 5. 分层专项审查清单

### 5.1 L0 层（kernel / testkit）专项

| 检查项              | 说明                                                           |
| ------------------- | -------------------------------------------------------------- |
| `#![no_std]` 兼容性 | kernel 应能可选地在 `no_std` 下编译（当前 std-only，确认意图） |
| loom 兼容性         | 并发路径是否可在 loom 下运行（`#[cfg(loom)]`）                 |
| 零依赖纪律          | 是否仅依赖 `thiserror` 等最小依赖                              |
| ClockDomain 安全    | 单调时间是否防止跨域比较                                       |
| ShutdownSignal      | 是否一次触发、多方观察、无数据竞争                             |
| ComponentState      | 状态转换是否完整、非法转换是否被阻止                           |

### 5.2 Types 层（decimalx / canonical）专项

| 检查项        | 说明                                                                         |
| ------------- | ---------------------------------------------------------------------------- |
| decimal 精度  | `checked_*` 是否覆盖所有算术操作；资金路径是否禁用 panicking op              |
| serde 安全性  | 反序列化是否拒绝非法 scale/currency                                          |
| wire 版本策略 | committed 类型是否满足：`deny_unknown_fields` + golden + N-1 兼容 + 拒绝样例 |
| shape 校验    | `is_plausible_*` 函数是否在构造点调用                                        |
| ID 类型化     | `VenueId`/`InstrumentId` 是否使用 newtype 防止混用                           |

### 5.3 L1 模块专项

| 模块         | 专项检查                                                                         |
| ------------ | -------------------------------------------------------------------------------- |
| `configx`    | 配置读取是否线程安全？快照一致性？                                               |
| `schedulex`  | 任务 ID 唯一性保证？tick 驱动时钟是否无漂移？                                    |
| `observex`   | Instrumentation 实现是否无开销（零成本抽象）？                                   |
| `resiliencx` | CircuitBreaker 是否有半开路径？RateLimiter 是否公平？Bulkhead 是否防止资源泄漏？ |
| `evidence`   | HMAC key 管理？追加操作是否不可变？                                              |
| `bootstrap`  | Bounded* context 是否仅暴露必需方法？                                            |
| `transport`  | 连接池管理？TLS 配置？超时/重试策略？代理支持？                                  |

### 5.4 Contracts 层专项

- [ ] 每个 trait 是否有完备语义文档（输入/输出/错误分类/幂等/一致性）？
- [ ] 是否有 conformance suite（`contract-testkit`）？
- [ ] 是否有至少一个非 scaffold 验证入口？
- [ ] 新增方法是否无 additive default 陷阱？
- [ ] `VenueAdapter` 拆分为能力 trait 的进展？

### 5.5 Adapters 层专项

- [ ] 实现是否完全覆盖对应 trait 的所有方法？
- [ ] 错误是否准确映射到 `XError` kind？
- [ ] 是否有 mock/fake 实现支持测试？
- [ ] 是否使用 trait 而非具体类型依赖？
- [ ] 连接/资源管理是否遵循 RAII？
- [ ] 重试/熔断策略是否合理（resiliencx 集成）？
- [ ] 真实后端 live 测试是否存在（哪怕 `#[ignore]`）？

**Exchange 适配器额外检查：**

- [ ] API 签名/认证是否正确（HMAC、时间戳、nonce）？
- [ ] WebSocket 重连机制？
- [ ] 速率限制遵循交易所规定？
- [ ] 订单字段映射是否正确（side、time-in-force、order type）？
- [ ] 错误码映射完整？

**Storage 适配器额外检查：**

- [ ] 连接池配置是否可用户自定义？
- [ ] 事务支持（postgresx `TxRunner`）？
- [ ] 发布/订阅语义（kafka/nats/redis）？

### 5.6 Tools 专项

| 检查项     | 说明                                                       |
| ---------- | ---------------------------------------------------------- |
| CLI UX     | `--help` 是否完整？错误信息是否可理解？                    |
| 子命令设计 | `goalctl` doctor/validate/compile/lint 是否正交？          |
| 证据完整性 | `verifyctl` plan/execute/report/dry-run 是否满足审计需求？ |
| 配置管理   | 是否使用结构化配置而非裸 CLI args？                        |

---

## 6. 质量门禁验证

审查完成后，**必须**执行以下门禁（或说明为何不能执行）：

### 6.1 强制门禁（通过 CI 或本地）

```bash
# 1. 编译
cargo build --workspace

# 2. 测试
cargo test --workspace

# 3. 代码风格
cargo fmt --all --check

# 4. 静态分析
cargo clippy --workspace --all-features --all-targets -- -D warnings

# 5. 依赖检查
cargo deny check

# 6. workspace 依赖门禁
node scripts/quality-gates/check-workspace-deps.mjs

# 7. 全面质量门禁
node scripts/quality-gates/check.mjs

# 8. 版本检查
node scripts/quality-gates/check-crate-versions.mjs
```

### 6.2 目标门禁（涉 crate 必执行）

```bash
# kernel loom
RUSTFLAGS='--cfg loom' cargo test -p kernel --test lifecycle_concurrency_loom --release

# canonical 对齐
node scripts/quality-gates/check-canonical-align.mjs

# 覆盖率门禁（核心 crate）
node scripts/quality-gates/cov-gate-100.mjs kernel
node scripts/quality-gates/cov-gate-100.mjs decimalx
node scripts/quality-gates/cov-gate-100.mjs canonical
```

### 6.3 门禁结果记录格式

```text
| 门禁项                   | 状态（✅/❌/N/A） | 备注                  |
| ------------------------ | --------------- | --------------------- |
| cargo build              | ✅               |                       |
| cargo test               | ✅               | 3 个 warning 见下     |
| cargo fmt --check        | ✅               |                       |
| cargo clippy -D warnings | ❌               | 4 个 clippy 警告：... |
| ...                      |                 |                       |
```

---

## 7. 报告输出模板

每份审查报告必须遵循以下结构。报告保存至 `docs/report/{yyyy-mm-dd}/review-{module-name}.md`。

### 7.1 单 crate 审查报告模板

```markdown
# Review: {crate-name} v{version} — {审查日期}

| 字段       | 值                                |
| ---------- | --------------------------------- |
| 目标 crate | `{crate-name}` → `{package-name}` |
| 路径       | `{path}`                          |
| 层级       | {L0 / L1 / L2 / Types / Tools}    |
| 审查日期   | {yyyy-mm-dd}                      |
| 审查者     | {AI Agent / 人工}                 |
| 前置依赖   | {审查所需的其他 crate/文档清单}   |
| SSOT       | `{ssot-path}`                     |
| 对齐文档   | `{alignment-doc-path}`            |

## 1. 概览

<!-- 300 字以内的定性汇总，含主要发现和总体评估 -->

## 2. 通用维度评估

| 维度                  | 评分 (0-5) | 说明 |
| --------------------- | ---------- | ---- |
| D1. 公开 API 正确性   | {0-5}      |      |
| D2. 类型与不变量      | {0-5}      |      |
| D3. 错误处理          | {0-5}      |      |
| D4. 并发安全          | {0-5}      |      |
| D5. 泛型与 Trait 设计 | {0-5}      |      |
| D6. 依赖与版本        | {0-5}      |      |
| D7. SSOT 对齐         | {0-5}      |      |
| D8. 测试覆盖          | {0-5}      |      |
| D9. 可观测性          | {0-5}      |      |
| **Σ**                 | **/{45}**  |      |

## 3. 分层专项评估

<!-- 引用 §5 中对应层级的专项检查清单，逐项检查并记录 -->

## 4. 发现明细

### P0：阻塞性缺陷

| #   | 文件:行号       | 问题描述 | 类别        | 修复建议 |
| --- | --------------- | -------- | ----------- | -------- |
| 1   | `src/lib.rs:42` | ...      | 安全/正确性 | ...      |

### P1：重要问题

| #   | 文件:行号 | 问题描述 | 类别 | 修复建议 |
| --- | --------- | -------- | ---- | -------- |

### P2：建议改进

| #   | 文件:行号 | 问题描述 | 类别 | 修复建议 |
| --- | --------- | -------- | ---- | -------- |

### P3：代码风格/微优化

| #   | 文件:行号 | 问题描述 | 类别 | 修复建议 |
| --- | --------- | -------- | ---- | -------- |

## 5. SSOT 对齐状态

| 规格条目      | 实现状态                | 对齐结论          | Gap 说明 |
| ------------- | ----------------------- | ----------------- | -------- |
| {spec item 1} | {fully/partial/missing} | {PASS/DEFER/OPEN} |          |
| {spec item 2} | {fully/partial/missing} | {PASS/DEFER/OPEN} |          |

## 6. 质量门禁结果

| 门禁项                   | 状态    | 备注 |
| ------------------------ | ------- | ---- |
| cargo build              | ✅/❌/N/A |      |
| cargo test               | ✅/❌/N/A |      |
| cargo fmt --check        | ✅/❌/N/A |      |
| cargo clippy -D warnings | ✅/❌/N/A |      |
| cargo deny check         | ✅/❌/N/A |      |
| check-workspace-deps     | ✅/❌/N/A |      |
| cov-gate-100             | ✅/❌/N/A |      |
| 其余专项门禁             | ✅/❌/N/A |      |

## 7. 生产就绪判定

| 维度          | 判定                                      |
| ------------- | ----------------------------------------- |
| L 层          | L{1-5} / N/A                              |
| S 完整性      | Σ/35                                      |
| QT 场景       | QT-{1-7}：Ready / Conditional / Gap / N/A |
| 整体 Go/No-Go | GO / 有条件 GO / NO-GO / N/A              |
| 阻塞项        | {none / list blocking items}              |

## 8. 综合建议

<!-- 对模块负责人的具体行动建议 -->

## 9. 变更记录

| 日期         | 说明 |
| ------------ | ---- |
| {yyyy-mm-dd} | 初版 |
```

### 7.2 跨 crate / 集成审查报告模板

```markdown
# 集成审查：{主题} — {审查日期}

| 字段     | 值                       |
| -------- | ------------------------ |
| 审查范围 | {涉及的 crate 清单}      |
| 视角     | {集成 / 兼容性 / 端到端} |
| 审查日期 | {yyyy-mm-dd}             |

## 1. 集成图

<!-- ASCII 或描述性依赖/调用关系图 -->

## 2. 跨 crate 接口分析

| 接口         | 生产者  | 消费者  | 兼容性 | 备注 |
| ------------ | ------- | ------- | ------ | ---- |
| {trait/type} | {crate} | {crate} | ✅/⚠️/❌  |      |

## 3. 集成风险

| #   | 风险描述 | 影响  | 发生概率 | 缓解措施 |
| --- | -------- | ----- | -------- | -------- |
| 1   |          | H/M/L | H/M/L    |          |

## 4. 端到端链路验证

<!-- 如适用：从 bootstrap → adapter → 外部系统的完整链路验证 -->

## 5. 综合结论

```

### 7.3 目录结构约定

```text
docs/report/{yyyy-mm-dd}/
├── README.md                    # 报告索引
├── review-prompt.md             # 本审查 prompt（审查指南）
├── crate-inventory.md           # 包清单与 SSOT 映射（可选）
├── production-readiness-criteria.md  # 生产就绪判据（可选）
├── review-{crate1}.md           # 单 crate 审查
├── review-{crate2}.md
├── integration-{topic}.md       # 集成/跨 crate 审查
└── synthesis.md                 # 综合裁定（多轮/多次审查后）
```

---

## 8. 治理约束与禁止宣称

### 8.1 禁止宣称清单

AI Agent **严禁**在审查报告中宣称以下事项：

1. ❌ **「workspace 整体 Production Ready」** — 只有 Maintainer 人类可签核 L5
2. ❌ **「Agent 已完成 L5 / 代签 prod-signoff」** — 签核不可由 AI 完成
3. ❌ **「STATUS 99% 即等于可生产发布」** — 完成度 ≠ 生产就绪
4. ❌ **「exchange 可交易 / 全功能就绪」** — 除非有最新 live 证据更新
5. ❌ **「SSOT COMPLETE 镜像即等于本仓已交付」** — SSOT 是规格，实现以 `crates/**/src` 为准
6. ❌ **「本审查已覆盖所有安全风险」** — 审查不替代安全审计
7. ❌ **使用 emoji 输出** — 除非用户明确要求

### 8.2 必须声明的局限

每份审查报告必须包含以下声明之一：

> **「本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计。审查结论仅代表代码基线在审查时刻的快照分析。」**

### 8.3 工作区约束

- 所有活跃开发在 `.worktrees/<branch-name>` 内进行，**禁止**直接在 `main` 上编辑代码
- CI 门禁不可绕过（禁止 `--no-verify`）
- 所有第三方依赖必须 `{ workspace = true }`，禁止内联版本

### 8.4 语言约束

- 审查报告、注释、用户可见错误信息：**简体中文**
- 标识符、代码、技术术语：**英文**
- 许可证文件：**保留英文原文**
- 非默认使用英文技术正文；需要书面豁免

---

## 9. 反模式与常见陷阱

### 9.1 审查姿势反模式

| 反模式                  | 正确做法                                              |
| ----------------------- | ----------------------------------------------------- |
| 只看 diff 不看全文件    | 对核心逻辑模块，完整阅读 `src/` 下的全部源文件        |
| 忽略测试文件            | 测试是规格的一部分；检查测试是否覆盖错误路径          |
| 跳过文档                | `rustdoc`、README、`docs/` 下的对齐文档都应抽查       |
| 只挑软问题（格式/命名） | P0/P1 硬问题优先：安全、正确性、不变量破坏            |
| 过聚焦非生产路径      | 按优先级聚焦生产路径（L0/L1 核心 + adapter 业务逻辑） |
| 单文件孤立审查          | 跨 crate 接口变化需要检查所有消费者                   |

### 9.2 代码中常见陷阱

| 陷阱                  | 示例                                     | 如何发现                                                     |
| --------------------- | ---------------------------------------- | ------------------------------------------------------------ |
| 静默精度丢失          | `f64` 金额运算                           | 检索 `f64` 在金额/价格上下文的使用                           |
| serde 无校验          | 反序列化未检查字段合法性                 | 搜索 `#[derive(Deserialize)]` → 确认有 `deny_unknown_fields` |
| 未声明 panic          | `.unwrap()` 在 fallible 公开函数中       | `rg '\.unwrap\(\)' crates/{name}/src/`                       |
| 错误分类错误          | 将 `Invalid` 映射为 `Transient`          | 检查错误 `From`/`Into` 实现                                  |
| 幽灵依赖              | crate 引用间接依赖                       | `cargo tree -p {name}` 检查                                  |
| additive default 漏洞 | trait 新方法有默认返回 Error             | 检查 `contracts` 新增方法是否有默认实现                      |
| 跨域时钟比较          | `MonotonicInstant` 跨 `ClockDomain` 比较 | > 编译期已阻止（类型安全）；检查 `unsafe` 绕过               |
| 资源泄漏              | 未在 drop 时释放连接                     | 检查 `Drop` 实现                                             |

### 9.3 报告写作反模式

| 反模式                   | 正确做法                                     |
| ------------------------ | -------------------------------------------- |
| 笼统评价「代码质量良好」 | 给出具体维度的评分和证据                     |
| 发现问题不给行号         | 每个问题附 `文件:行号` 和代码引用            |
| 只现问题不给建议      | P0/P1 问题必须附带修复建议                   |
| 夸大严重程度             | 诚实评级：P0=阻塞、P1=重要、P2=改进、P3=风格 |
| 忽略已存在的 DEFER       | 审查前阅读上次审查的 DEFER 列表              |

---

## 10. 附录：快捷命令集

### 10.1 信息收集

```bash
# 工作区信息
cargo metadata --no-deps --format-version 1
cargo tree -p {name}
cargo doc -p {name} --no-deps --open

# 代码统计
cloc crates/{name}/src/
rg -c 'fn ' crates/{name}/src/

# 依赖图
cargo depgraph | dot -Tpng -o deps.png
```

### 10.2 安全与正确性检查

```bash
# 检索潜在 panic
rg 'unwrap\(\)|expect\(|\.unwrap\(\)' crates/{name}/src/
rg '\[.*\]' crates/{name}/src/  # 索引访问
rg 'unsafe' crates/{name}/src/

# 检索 serde 反序列化
rg 'derive.*Deserialize' crates/{name}/src/
rg 'deny_unknown_fields' crates/{name}/src/

# 检索错误映射
rg 'ErrorKind' crates/{name}/src/
rg 'XError' crates/{name}/src/

# 检索 f64 使用（金额上下文）
rg 'f64' crates/{name}/src/
```

### 10.3 测试分析

```bash
# 测试覆盖
cargo tarpaulin -p {name} --out Html

# 运行指定测试
cargo test -p {name} --all-targets
cargo test -p {name} --doc

# loom 测试
RUSTFLAGS='--cfg loom' cargo test -p {name} --release
```

### 10.4 质量门禁

```bash
# 全量门禁
cargo clippy --workspace --all-features --all-targets -- -D warnings
cargo fmt --all --check
cargo deny check
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps

# workspace 专项
node scripts/quality-gates/check.mjs
node scripts/quality-gates/check-crate-versions.mjs
node scripts/quality-gates/check-workspace-deps.mjs
node scripts/quality-gates/check-canonical-align.mjs
```

---

## 变更记录

| 日期       | 说明                                                                            |
| ---------- | ------------------------------------------------------------------------------- |
| 2026-07-22 | 初版：基于 `production-readiness-criteria` L1–L5/S1–S7/QT 框架的综合审查 Prompt |

---

> **附录：** 本 Prompt 设计为自包含的审查引导文档。执行审查前，AI Agent 应完整阅读本 Prompt 的全部内容，确保理解每个维度的评估标准和输出格式要求。
