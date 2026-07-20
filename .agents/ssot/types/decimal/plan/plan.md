# Plan — GOAL-TYPES-DECIMALX-002 / SPEC-TYPES-DECIMALX-002 agent-safe 闭合

| 字段 | 值 |
|------|-----|
| Plan ID | `PLAN-TYPES-DECIMALX-002-agent-safe-v1` |
| Source Goal | [`20260717/xhyper-decimalx-complete-goal.md`](../20260717/xhyper-decimalx-complete-goal.md) · `GOAL-TYPES-DECIMALX-002` |
| Source Spec | [`20260717/xhyper-decimalx-complete-spec.md`](../20260717/xhyper-decimalx-complete-spec.md) · `SPEC-TYPES-DECIMALX-002` |
| Active SSOT | [`../decimalx-spec.md`](../decimalx-spec.md)（**仍为权威验收合同**） |
| Package | `xhyper-decimalx` @ `crates/types/decimal` · **0.1.0** |
| Gap Matrix | [`gap-matrix.md`](./gap-matrix.md) |
| Tasks | [`tasks.md`](./tasks.md) |
| Residual | [`residual-open.md`](./residual-open.md) |
| Work Todo | [`../todo.md`](../todo.md) |
| CURRENT-STATE | [`CURRENT-STATE.md`](./CURRENT-STATE.md) |
| Alignment | [`alignment-decimalx-2026-07-17.md`](./alignment-decimalx-2026-07-17.md) |
| 10x Verdict | [`decimalx-plan-10x-verdict.md`](./decimalx-plan-10x-verdict.md) |
| Strategy | **对账 → 台账 → M0 inventory/边界测试 → 非破坏文档加固 → 门禁 → 10x → @liukongqiang5 APPROVE** |
| Campaign status | **DONE (agent-safe)** · 10x PASS · **≠** Goal ACHIEVED · **≠** Spec Approved · **≠** package stable |
| Forbidden | Draft 升 Approved · 伪造 APPROVE · 字段私有化未批 · 改 wire · 假 ACHIEVED · 路径迁 `numeric` · `decimalx→canonical` 循环 |

---

## 0. 深度分析结论

### 0.1 事实源优先级

```text
cargo metadata + crates/types/decimal/src/** + ADR-006/007
  > active decimalx-spec.md
  > Draft SPEC-TYPES-DECIMALX-002 / GOAL-TYPES-DECIMALX-002（候选，非权威）
```

| 主题 | 当前事实 `[KNOWN] HIGH` | Draft 状态 |
|------|-------------------------|------------|
| 路径 | `crates/types/decimal` | 迁 `numeric` **REJECTED** |
| 依赖 | `xhyper-kernel` + `serde`；dev criterion/proptest/serde_json | `decimalx→canonical` **REJECTED** |
| 表示 | `pub mantissa: i128`, `pub scale: u8` | 字段私有化 / MAX_SCALE **PROPOSED** |
| 算术 | checked + panicking `+/-/*`/`rescale` | panic 面收敛 **PROPOSED** |
| 除法 | scale=`max(lhs,rhs)`；显式 `RoundingStrategy` | target scale **OPEN** |
| 舍入 | Floor/Ceiling/HalfUp/HalfDown/HalfEven | ADR-006 **APPROVED** |
| Eq/Hash | 数值语义 + trailing-zero normalize | 全边界验证 **PROPOSED** |
| wire | serde 结构字段 shape | 跨版本 stable **OPEN** |
| Currency | FromStr 3 大写 ASCII；字段公开 | 封闭 **PROPOSED** |

### 0.2 与 OBJECTIVE「全部 DONE」的裁定

OBJECTIVE 要求「目标全部完成 DONE」与 Goal/Spec 中多项 `PROPOSED`/`OPEN`/人审批准 **冲突**。

**本计划裁定**：

1. **全部 agent-safe 任务** → `DONE` + Evidence；
2. MAX_SCALE 取值、字段私有化、错误枚举升格、wire stable、Spec Approved、Goal Achieved → **HUMAN_ONLY**；
3. 全量下游 consumer 破坏性迁移 → **DEFERRED**（可分期）；
4. todo 零 bare OPEN agent-safe 行 = 战役完成，**≠** GOAL ACHIEVED / Spec Approved。

### 0.3 REJECTED 方向（不得回流）

| 候选 | 裁定 |
|------|------|
| 迁移到 `crates/types/numeric` | **REJECTED** |
| `decimalx → canonical` | **REJECTED**（环） |
| 默认 `Money<U>` 泛型单位 | **REJECTED** |
| BigInt 替换 i128 | 非本战役；无边界证据不得做 |
| 汇率/跨币种/tick/会计政策 | 非目标 |

### 0.4 实现基线（源码对账）

- `Decimal::new` / 公开字段可构造任意 `scale`/`mantissa`（含 u8 满量程）。
- checked：`checked_add/sub/mul/div`、`checked_rescale` → `XError::Invalid`。
- panicking：`Add`/`Sub`/`Mul`/`rescale` 走 checked + `expect`。
- 内联测试 ~35 + proptest 12（fuzz.rs）；ADR-006 核心路径已有覆盖，边界可再补强。
- consumers：`domain_{market,macro,exchange,ledger,core}`、`binance`/`okx`、`taos`、`canonical`、`schema_codegen`、`contract-testkit`。
- 证据：[`evidence/m0-consumer-inventory-2026-07-17.txt`](./evidence/m0-consumer-inventory-2026-07-17.txt)。

---

## 1. 执行策略

```text
1. 证据优先：PASS 绑定 cargo test / rg inventory / SCRATCH 或 plan/evidence
2. 外科手术：crates/types/decimal + 本 plan/todo + 对齐；不改 wire/字段可见性
3. 单 writer：docs/plan | tests | rustdoc/README | 10x | approve
4. residual 纪律：DONE / HUMAN_ONLY / DEFERRED / POLICY only
5. 禁止：假 Approved、伪造 APPROVE、把 Draft 写成 SSOT
6. 十轮：fail_rounds=0
7. 分支：fix/types-decimalx-agent-safe-20260717（非 main）
```

### 1.1 Agent team 路径分片

| Wave | 角色 | 路径 | 产出 |
|------|------|------|------|
| W0 | plan-writer | `.agent/SSOT/types/decimal/plan/**` · `todo.md` | 台账 + gap |
| W1 | inventory | `rg` / metadata / evidence/ | M0 baseline |
| W2 | tests | `crates/types/decimal/src/lib.rs` tests · `tests/fuzz.rs` | 边界/Eq/Hash/parse |
| W3 | docs | rustdoc `# Panics` · README · active spec 链接 · CHANGELOG | 非破坏对齐 |
| W4 | verify | cargo test/check/clippy/fmt · 10x · PR · approve | 收口 |

同文件禁止并行写；W2/W3 合并后统一跑门禁。

---

## 2. 里程碑映射（agent-safe 子集）

| Milestone | Goal 意图 | 本战役可做 | 不做 |
|-----------|-----------|------------|------|
| **M0** | inventory + 边界测试 | inventory 落盘；parse/Display/cmp/Hash/Currency 补强 | 全 i128 空间证明 |
| **M1** | Limits/安全构造 | 文档登记 PROPOSED；不引入未批 MAX_SCALE | 字段私有化、强制 MAX_SCALE |
| **M2** | Panic 面 | 调用点清单；`# Panics` rustdoc；生产 checked 指引 | 删除 panicking API / 全迁移 |
| **M3** | Wire | 现有 serde 测试 + OPEN 登记 | 宣称 wire stable / 改 shape |

---

## 3. 验证计划（战役门禁）

```bash
cargo test -p xhyper-decimalx
cargo check -p xhyper-decimalx --all-targets
cargo clippy -p xhyper-decimalx --all-targets -- -D warnings
cargo fmt -- --check
# 未改依赖图：可不跑 lint-deps；若改 Cargo.toml 则必须
.agent/SSOT/types/decimal/plan/scripts/run_10x_gate.sh
```

10x checklist 见 [`checklist-10x.md`](./checklist-10x.md)。

---

## 4. 完成定义（战役）

- [x] plan/ 完整包 + todo.md disposition
- [x] agent-safe 任务全 DONE（无 bare OPEN；T-VER-003 待 approve 读回后闭合）
- [x] 聚焦门禁绿 + SCRATCH 日志
- [x] 对齐文档一致且不宣称 stable/Approved/Achieved
- [x] 10x `fail_rounds=0`
- [x] approve readback：PR #507 · tip-bound SSOT=SCRATCH；plan/evidence 为 POINTER_NOT_TIP_BOUND（≠ Goal Achieved）

**明确不在战役完成内**：Spec Approved、Goal Achieved、package stable、consumer=0 破坏性迁移。
