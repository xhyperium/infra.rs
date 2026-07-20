# Plan — GOAL-TYPES-CANONICAL-002 / SPEC-TYPES-CANONICAL-002 闭合执行包

| 字段 | 值 |
|------|-----|
| Plan ID | `PLAN-TYPES-CANONICAL-002-v1` |
| Source Goal | [GOAL-TYPES-CANONICAL-002](../20260717/xhyper-canonical-complete-goal.md) · Status **Draft** |
| Source Spec | [SPEC-TYPES-CANONICAL-002](../20260717/xhyper-canonical-complete-spec.md) · Status **Draft / Non-normative** |
| Active SSOT | [canonical-spec.md](../canonical-spec.md) |
| Package | `xhyper-canonical` / `canonical` @ `crates/types/canonical` **0.1.0** |
| Layer | Types / 跨层共享纯 DTO |
| Baseline | `main@4fe8e988`（战役开盘） |
| Work Todo | [../todo.md](../todo.md) |
| Gap Matrix | [gap-matrix.md](./gap-matrix.md) |
| Spec Inventory | [spec-inventory.md](./spec-inventory.md) |
| Tasks | [tasks.md](./tasks.md) |
| Residual | [residual-open.md](./residual-open.md) |
| Approval Packet | [approval-packet.md](./approval-packet.md) |
| 10x Verdict | [canonical-plan-10x-verdict.md](./canonical-plan-10x-verdict.md) |
| Alignment | [alignment-2026-07-17.md](./alignment-2026-07-17.md) · [evidence/types-canonical-002/](../../../../../evidence/types-canonical-002/) |
| Strategy | **诚实 inventory → agent-safe 事实闭合 → residual 显式 OPEN → 门禁绿 → 10x → 人审 APPROVE** |
| Campaign status | **agent-safe 闭合目标** · **≠ Spec Approved** · **≠ package stable** · **≠ Goal ACHIEVED / Production Ready** |

> 本 plan 的 DONE 仅表示 agent-safe 工作项与门禁证据闭合。  
> **禁止**把 10x PASS、PR APPROVE、todo DONE 写成 Spec Approved / stable / wire 跨版本承诺。

---

## 0. 深度分析结论（对照 20260717 goal + spec）

### 0.1 `canonical` 是什么

`canonical` 的批准终态是 **最小、稳定、无业务行为的跨层 DTO 词汇表**（ADR-001/007、架构 §4.2），不是：

```text
schema registry / CanonicalWriter-Reader / 通用 codec / hash-sign-evidence 框架
```

确定性 evidence 编码由 `xhyper-evidence` 的 versioned `canonical::v1` 模块负责；名称相似 **不** 构成迁移本 crate 职责的理由。

### 0.2 当前实现一句话（源码事实，`4fe8e988`）

| 面 | 事实 |
|----|------|
| 公开类型 | 4 ID（含 deprecated `OrderId`）+ `CancelOrderRequest` + 2 枚举 + 8 DTO；`Money` 重导出 `decimalx` |
| 依赖 | `xhyper-decimalx` + `serde`；dev：`serde_json` |
| 行为 | 无业务方法 / I/O / 全局状态 / 本地错误类型 |
| 数值 | 无 `f32`/`f64` 金融字段 |
| 测试 | 5 个单测：Order RT、空 book、OrderRef/cancel wire、legacy OrderAck wire、OKX cancel fixture 双向 |
| Fixture | `fixtures/market/order_cancel_okx.json` |
| 消费者 | contracts、binance、okx、domain_{core,market,exchange,macro}、bootstrap e2e、contract-testkit、taos |

### 0.3 主要漂移与缺口

| ID | 主题 | 严重度 | 处置 |
|----|------|--------|------|
| DRIFT-01 | active Candidate 链接指向已删除 `.agent/draft/...` | P0 文档 | agent-safe 修正 |
| DRIFT-02 | Goal/Spec 相对路径写成 `../specs/types/canonical/...`（错误） | P1 文档 | agent-safe 修正 |
| DRIFT-03 | active 已登记 ID/cancel 类型，但 README 未列 `OrderRef`/`CancelOrderRequest`/`VenueId`/`InstrumentId` | P1 | agent-safe |
| DRIFT-04 | 规范要求全 DTO/枚举 RT + 全 `OrderStatus`/`OrderRef` variants；当前仅 5 测 | P1 | agent-safe 补测 |
| DRIFT-05 | 无 `Money ≡ decimalx::Money` 类型同一性回归 | P1 | agent-safe 补测 |
| DRIFT-06 | 无正式 consumer inventory / 字段语义缺口表落盘 | P1 | plan 包 + todo |
| OPEN-ID | Venue/Instrument/Order 字符集·规范化·映射 | OPEN | residual / HUMAN |
| OPEN-TIME | `ts: i64` 单位/epoch/范围 | OPEN | residual / HUMAN |
| OPEN-WIRE | 未知字段策略、schema 版本、未覆盖 DTO 的 wire 承诺 | OPEN | residual / HUMAN |
| OPEN-VALID | 各 DTO validation owner 登记未冻结 | OPEN | residual / HUMAN |
| OPEN-MIG | legacy `OrderId`/`Order`/`OrderAck` 删除时机 | OPEN | residual；**禁删** |
| OPEN-LAYOUT | 新建 `types/core`/`types/protocol` 或移除 serde | OPEN | 须独立 RFC |
| REJECT-CODEC | Canonical Encoding Core | REJECTED | 持续门禁 |

### 0.4 对原草案方向的再确认（与 goal §3 一致）

| 候选 | 裁定 | 本 plan 动作 |
|------|------|--------------|
| Canonical Encoding Core | **REJECTED** | 禁止引入；10x 负向扫描 |
| 新建 types/core·protocol 大搬迁 | **OPEN** | residual；本战役不搬迁 |
| `canonical → ∅` | **REJECTED** | 保持 `→ decimalx` |
| 移除 serde | **OPEN** | residual；本战役不删 |
| 纯 DTO 零业务行为 | **APPROVED** | 源码+测试持续证明 |

### 0.5 完成语义（防假关）

```text
agent-safe DONE  ≠  Spec Draft→Approved
10x fail_rounds=0  ≠  实现完成宣称 Spec Approved
PR APPROVE         ≠  package stable / crates.io
M0 事实闭合        ≠  M1 语义批准 / M3 全量迁移
```

---

## 1. 里程碑映射（M0–M3）

| 里程碑 | Goal 定义 | 本战役 agent-safe 范围 | 非本战役 |
|--------|-----------|------------------------|----------|
| **M0** 事实闭合 | 同步 active API/fixture/consumers；字段语义缺口表 | active 链接/API 对齐；inventory；gap；测试补全；README | 无 |
| **M1** 身份与时间 | 批准 ID/时间/字符串语义；additive 新类型 | **仅**登记 OPEN + 文档不得谎称已批准；不臆造 newtype | 人审批准语义；引入 Timestamp newtype |
| **M2** Wire 治理 | 承诺类型的版本化 fixture；未承诺标不稳定 | 已有 cancel + legacy ack golden 保持；文档区分承诺/未承诺 | 全 DTO wire 稳定承诺 |
| **M3** 下游迁移 | contracts/adapters/domain/testkit 迁移 | consumer inventory；触及路径编译；**保留** legacy API | consumer=0 后删除；trait 大迁 |

---

## 2. 已批准不变量（必须持续证明）

| ID | 内容 | 证明方式 |
|----|------|----------|
| CAN-BND-001 | 纯 DTO，无业务状态机/I/O | 源码无 `impl` 业务方法；grep |
| CAN-NUM-001 | 数值单点 `decimalx`；禁金融 f32/f64 | Cargo + type identity test + grep |
| CAN-LAYER-001 | 不依赖 contracts/L1/domain/adapter/evidence | `cargo xtl lint-deps` + Cargo.toml |
| CAN-CODEC-001 | 禁通用 codec/hash-sign | residual + 10x 负向扫描 |

---

## 3. 路径互斥与 agent team 分片

| Lane | 允许路径 | 禁止 |
|------|----------|------|
| L-DOC | `.agent/SSOT/types/canonical/**`、`evidence/types-canonical-002/**` | 改 runtime 语义裁定 |
| L-DTO | `crates/types/canonical/**`、`fixtures/market/**`（仅既有 wire） | 删 OrderId；加 codec；上层依赖 |
| L-DOWN | 仅当编译失败时最小修 adapters/contracts/domain/testkit | 无 consumer 证据的 breaking |
| L-GATE | 门禁命令、10x 脚本、SCRATCH 日志 | 伪造 APPROVED / 手写 digest |

单任务单 writer；跨 lane 合并由 root 验证。

---

## 4. 禁止项（I-CAN-FORBID）

1. 假 Spec Approved / package stable / Goal Achieved / Production Ready  
2. 将 `canonical` 改造成 codec core / schema registry / hash-sign  
3. 无 consumer 清零证据删除 `OrderId`/`Order`/`OrderAck`  
4. 文档声称 `ts` 为 ms/ns 或 ID 字符集已冻结（在人审前）  
5. 引入 contracts/L1/domain/adapter/evidence 生产依赖  
6. 金融字段使用 `f32`/`f64` 或复制 `(mantissa, scale)`  
7. 10x 单轮 cherry-pick PASS / SKIP 计 PASS  
8. AI 独断把 OPEN 语义写成 APPROVED  
9. 伪造 `@liukongqiang5` APPROVE 或手写 APPROVED 无 API readback  
10. 双 SSOT：active 与 candidate 互相覆盖而不标注 Draft 边界  

---

## 5. 验收门禁（agent-safe）

```bash
cargo test -p xhyper-canonical
cargo check -p xhyper-canonical --all-targets
cargo clippy -p xhyper-canonical --all-targets -- -D warnings
cargo xtl lint-deps
cargo fmt -- --check
```

受影响时追加：`xhyper-binance` / `xhyper-okx` / contracts / domain / contract-testkit 相关 `-p` 测试。

十轮：固定命令集 × 10，每轮独立日志，汇总 `fail_rounds=0`；失败修复后 **整组重跑**。

---

## 6. 交付物清单

| 路径 | 作用 |
|------|------|
| `plan/plan.md` | 本文件 |
| `plan/spec-inventory.md` | 公开 API / 依赖 / fixture / consumer / 禁止项枚举 |
| `plan/gap-matrix.md` | goal/spec 条款 ↔ 现状 ↔ 任务 |
| `plan/tasks.md` | 原子任务 T-* |
| `plan/residual-open.md` | OPEN/HUMAN/DEFER 诚实台账 |
| `plan/approval-packet.md` | 人审闸门（不由 AI 独断 Approved） |
| `plan/alignment-2026-07-17.md` | 对齐指针 |
| `plan/canonical-plan-10x-verdict.md` | 10x 汇总 |
| `todo.md` | 工作台账（DONE/HUMAN_ONLY/DEFERRED/POLICY） |
| `evidence/types-canonical-002/` | 门禁与 10x 证据目录 |

---

## 7. 执行顺序

1. 落盘 plan 包 + todo（本步）  
2. 修 Goal/Spec/active 交叉链接与 API 登记对齐  
3. 加固单测（全 DTO RT、variants、Money 同一性、fixture 双向）  
4. README/CHANGELOG 记实；consumer inventory  
5. 聚焦门禁绿  
6. 10x fail_rounds=0  
7. 开 PR；`export LIUKONGQIANG5_APPROVE_TOKEN`；`_closeout_approve.py` 或等价 API；保存 readback  
8. residual 保持 OPEN；**不**宣称 Spec Approved  

---

## 8. 追溯

- Goal: `GOAL-TYPES-CANONICAL-002`  
- Spec: `SPEC-TYPES-CANONICAL-002`  
- ADR-001 / ADR-007  
- `crates/types/canonical/{Cargo.toml,src/lib.rs}`  
- `fixtures/market/order_cancel_okx.json`  
- 架构 `docs/architecture/spec.md` §4.2  
