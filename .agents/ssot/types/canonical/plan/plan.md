# Plan — GOAL-TYPES-CANONICAL-002 / SPEC-TYPES-CANONICAL-002 闭合执行包

> **2026-07-21 权威对齐**：§0.2/§0.3 已按 Approved 生产路径改写（OrderId 类型已删；`ts`=Unix ns；S1 Approved）。
> 当前 1:1 矩阵见 [alignment-matrix-infra-2026-07-21.md](./alignment-matrix-infra-2026-07-21.md)；residual 见 [residual-open.md](./residual-open.md)。

| 字段 | 值 |
|------|-----|
| Plan ID | `PLAN-TYPES-CANONICAL-002-v1` |
| Source Goal | [GOAL-TYPES-CANONICAL-002](../20260717/xhyper-canonical-complete-goal.md) · **S1 path Approved**（≠ package stable / Goal 全 ACHIEVED） |
| Source Spec | [SPEC-TYPES-CANONICAL-002](../20260717/xhyper-canonical-complete-spec.md) · **Approved S1**（≠ package stable） |
| Active SSOT | [spec/spec.md](../spec/spec.md)（指针 [canonical-spec.md](../canonical-spec.md)） |
| Package | `xhyper-canonical` / `canonical` @ `crates/types/canonical` **0.1.0** |
| Layer | Types / 跨层共享纯 DTO |
| Baseline | `main@4fe8e988`（战役开盘）· infra 对齐 **2026-07-21** |
| Work Todo | [../todo.md](../todo.md) |
| Gap Matrix | [gap-matrix.md](./gap-matrix.md) |
| Spec Inventory | [spec-inventory.md](./spec-inventory.md) |
| Tasks | [tasks.md](./tasks.md) |
| Residual | [residual-open.md](./residual-open.md) |
| Approval Packet | [approval-packet.md](./approval-packet.md) |
| 10x Verdict | [canonical-plan-10x-verdict.md](./canonical-plan-10x-verdict.md) |
| Alignment | [alignment-2026-07-17.md](./alignment-2026-07-17.md) · [alignment-matrix-infra-2026-07-21.md](./alignment-matrix-infra-2026-07-21.md) |
| Strategy | **诚实 inventory → agent-safe 事实闭合 → residual 显式 OPEN → 门禁绿 → 人审**（10x 无 fresh 证据则 DEFER） |
| Campaign status | **agent-safe PASS** · Spec S1 **Approved** · **≠ package stable** · **≠ 全 wire Production Ready** |

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
| 公开类型 | `VenueId`/`InstrumentId` + `OrderRef` + `CancelOrderRequest` + 2 枚举 + 8 DTO；`Money` 重导出 `decimalx`；**无 `OrderId` 类型**（已删 2026-07-17） |
| 依赖 | `xhyper-decimalx` + `serde`；dev：`serde_json` |
| 行为 | 无业务方法 / I/O / 全局状态 / 本地错误类型 |
| 数值 | 无 `f32`/`f64` 金融字段 |
| 测试 | 22 单测：全 DTO RT、全 OrderStatus/OrderRef variants、Money 同一性、cancel/ack/v1 golden、shape/time helpers |
| Fixture | `fixtures/market/order_cancel_okx.json` + `order_ack_legacy.json` + `canonical/v1/*` |
| 消费者 | contracts、binance、okx、domain_{core,market,exchange,macro}、bootstrap e2e、contract-testkit、taos |

### 0.3 主要漂移与缺口（战役登记 → 终态）

> **2026-07-21**：DRIFT-01…06 均为 **历史缺口台账**；agent-safe 项已 **FIXED/CLOSED**。  
> 当前事实以 §0.2、[alignment-matrix-infra-2026-07-21.md](./alignment-matrix-infra-2026-07-21.md)、[residual-open.md](./residual-open.md) 为准。

| ID | 主题 | 终态 | 证据 |
|----|------|------|------|
| DRIFT-01 | active Candidate 链接指向已删除 `.agent/draft/...` | **FIXED** | active → `20260717/`；`spec/spec.md` Complete Spec 链接 |
| DRIFT-02 | Goal/Spec 相对路径错误 | **FIXED** | goal/spec 相对路径已校正 |
| DRIFT-03 | README 未列 OrderRef/Cancel/Venue/Instrument | **FIXED** | `crates/types/canonical/README.md` 公开类型清单 |
| DRIFT-04 | 全 DTO/枚举 RT + variants；曾仅 5 测 | **FIXED** | `cargo test -p xhyper-canonical` **22** tests（§0.2） |
| DRIFT-05 | 无 Money ≡ decimalx::Money 同一性 | **FIXED** | `money_is_decimalx_money_type_identity` |
| DRIFT-06 | 无 consumer inventory / 语义缺口表 | **FIXED**（文档） | `plan/spec-inventory.md` + gap/residual；产品树全量迁移仍 DEFER-M3 |
| OPEN-ID | Venue slug/shape 规则 **CLOSED**（CAN-ID）；OrderRef newtype 二期 **DEFER** | CLOSED/DEFER | residual OPEN-ID-001/002 |
| OPEN-TIME | `ts: i64` = Unix **ns** **CLOSED**（CAN-TIME-001 Approved） | CLOSED | residual OPEN-TIME-001 |
| OPEN-WIRE | 未知字段策略、schema 版本、未覆盖 DTO 的 wire 承诺 | OPEN | residual / HUMAN |
| OPEN-VALID | owner 表 v1 原则 **CLOSED**；consumer 可增补 | CLOSED 原则 | validation-owners.md |
| OPEN-MIG | `OrderId` **类型已删**；legacy `Order`/`OrderAck` DTO 字段形状保留至 consumer=0 | PARTIAL | residual OPEN-MIG-001 |
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
agent-safe DONE  ≠  package stable
Spec S1 Approved  ≠  package stable / crates.io / 全 wire Production Ready
10x fail_rounds=0  ≠  Goal ACHIEVED（无 fresh 10x 时 SAFE-15=DEFERRED）
PR APPROVE         ≠  package stable
M0/M1 语义闭合     ≠  M3 全量迁移 / DEFER-WIRE-FULL
```

---

## 1. 里程碑映射（M0–M3）

| 里程碑 | Goal 定义 | 本战役 agent-safe 范围 | 非本战役 |
|--------|-----------|------------------------|----------|
| **M0** 事实闭合 | 同步 active API/fixture/consumers；字段语义缺口表 | active 链接/API 对齐；inventory；gap；测试补全；README | 无 |
| **M1** 身份与时间 | 批准 ID/时间/字符串语义；additive 新类型 | **DONE** T1/T2：ts=ns；OrderRef/shape；OrderId 类型已删 | newtype 二期 DEFER；Timestamp 不进本 crate |
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
| L-DTO | `crates/types/canonical/**`、`fixtures/market/**`（仅既有 wire） | 删 legacy Order/OrderAck DTO 形状（无 consumer=0）；加 codec；上层依赖 |
| L-DOWN | 仅当编译失败时最小修 adapters/contracts/domain/testkit | 无 consumer 证据的 breaking |
| L-GATE | 门禁命令、10x 脚本、SCRATCH 日志 | 伪造 APPROVED / 手写 digest |

单任务单 writer；跨 lane 合并由 root 验证。

---

## 4. 禁止项（I-CAN-FORBID）

1. 假 Spec Approved / package stable / Goal Achieved / Production Ready  
2. 将 `canonical` 改造成 codec core / schema registry / hash-sign  
3. 无 consumer 清零证据删除 legacy `Order`/`OrderAck` **DTO 形状**（`OrderId` **类型**已于 2026-07-17 删除）  
4. 文档否认已批准事实：`ts`=Unix ns 与 CAN-ID shape **已 Approved**；禁止改回 OPEN 假叙事  
5. 引入 contracts/L1/domain/adapter/evidence 生产依赖  
6. 金融字段使用 `f32`/`f64` 或复制 `(mantissa, scale)`  
7. 10x 单轮 cherry-pick PASS / SKIP 计 PASS  
8. AI 独断把 OPEN 语义写成 APPROVED  
9. 伪造 `@liukongqiang5` APPROVE 或手写 APPROVED 无 API readback  
10. 双 SSOT：active 与 20260717 互相覆盖而不标注权威；当前 **S1 Approved** 与 active 应对齐  

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
