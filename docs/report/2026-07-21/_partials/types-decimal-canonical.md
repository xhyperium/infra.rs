# types：decimalx + canonical 生产就绪度 partial

| 字段 | 值 |
|------|-----|
| 审计日期 | 2026-07-21 |
| 审计 HEAD | `9174840`（`fix(hooks): address Codex P2 on nice/timeout gate (#155)`） |
| 范围 | `crates/types/decimal`（package **`decimalx`** / lib `decimalx`）、`crates/types/canonical`（package **`canonical`** / lib `canonical`） |
| 角色 | production readiness 审计员（只读源码；本文件为 partial） |
| 对照 | `docs/report/2026-07-21/core-crates-production-readiness.md` §1 / §4 / §11 / §12；`docs/ssot/types-ssot-alignment.md` |
| 验证 | `cargo test -p decimalx -p canonical --all-targets` → **exit 0**（见 §5） |

> **命名澄清**：`Cargo.toml` / `cargo metadata` 的 package 名为 `decimalx`、`canonical`。部分 SSOT/历史文档写 `xhyper-decimalx` / `xhyper-canonical` 为组织命名叙事；**本仓 `-p` 目标以 Cargo 名为准**。

---

## 1. 结论表

| 模块 | 分层（计划口径 L1–L5） | 生产判定 | 可接受使用 | 仍不可宣称 |
|------|------------------------|----------|------------|------------|
| **decimalx** | **L1 Internal Ready** | **有条件就绪（内部）** | 受控入口 `try_new` / parse / 校验 serde；资金与持久化路径**仅** `checked_*` | package stable、跨版本 wire 协议、整体 Production Ready、crates.io |
| **canonical** | **L2 Wire Ready（committed 子集）** | **部分就绪** | committed 清单类型（v1 + v1.1–v1.3）作跨层 DTO / JSON wire；adapter 入口配合 `shape::*` | 全 DTO 包级 PR、wire envelope、跨语言协议、业务校验、package stable |

| 维度 | decimalx | canonical |
|------|----------|-----------|
| **正确性** | 字段私有；非法 scale 不可表示；`checked_*` 对可达状态 `Ok/Err`；中间值溢出显式 `Err` | 纯 DTO，无业务语义；金额类型透传 `decimalx` 校验 |
| **契约** | ADR-006/007 数值合同清晰；除法必显式舍入 | wire 策略写死在 `src/wire.rs`；CAN-TIME-001 `ts`=Unix ns |
| **兼容** | serde shape = **当前事实**（`docs/WIRE.md` 明确非 stable） | committed 有 golden/N-1/拒绝样例；**无** envelope 版本字段 |
| **运维** | `DecimalError` 可分类 + 中文 Display → `XError` | 形状失败不 panic；业务错误由上层映射 |
| **安全** | `forbid(unsafe_code)`；无浮点金额 API；serde 拒绝非法 scale/币种 | `forbid(unsafe_code)`；committed `deny_unknown_fields`；非法 Decimal scale 反序列化失败 |
| **可验证** | unit + entry + proptest + oracle + boundary + adversarial；scheduled miri/mutants | unit wire 矩阵 + fixture + `public_api` + align 脚本 |
| **治理** | `[lints] workspace = true`；`publish = false`；中文错误 | 同上；`missing_docs` 仍 follow-up（未 `deny`） |

**相对 §1 初版快照（「未就绪」）**：在 HEAD `9174840` 上 **§1 已过时**。§11（PR #98）与 §12（W1–W2 + L5 GO-with-Accepts）的 **L1 / L2 分层判定仍适用**，且源码证据仍在。

---

## 2. 语义与实现要点

### 2.1 decimalx — 金额不变量

| 不变量 | 实现证据 | 状态 |
|--------|----------|------|
| `scale ≤ MAX_SCALE(18)` | 字段私有；`try_new` / `new`(panic) / `FromStr` / 校验 `Deserialize` | **闭合** |
| 非法 scale 不可在 crate 外表示 | `Decimal { mantissa, scale }` 私有 | **闭合**（§4.1 P0 已修） |
| 币种 3 大写 ASCII | `Currency::try_new` + 校验 serde | **闭合** |
| `Money` 双字段校验 | `try_new` + `validate` + 校验反序列化 | **闭合** |
| 生产运算路径 | `checked_add/sub/mul/div/rescale`；运算符 `+/-/*` / `rescale` 文档标 `# Panics` | **有条件**：纪律 + 门禁，非类型消灭 panicking API |
| 中间值溢出 | `RepresentationOverflow` / `MantissaOverflow`；即使约分后可表示仍 `Err` | **正式合同** |
| 禁止 f32/f64 金额 | 无浮点公开 API；AGENTS/README 约束 | **闭合** |
| `validate` 入口 | `Decimal::validate` / `Currency::validate` / `Money::validate` | **有**（对已构造值多为恒 Ok） |

**Price / Qty / Ratio**：透明 newtype，仅能包裹已校验 `Decimal`（不再公开 tuple 字段塞入非法值）。

### 2.2 decimalx — 非法状态可表示性（§4.1 复核）

| 历史 P0（§4.1） | HEAD `9174840` |
|-----------------|----------------|
| 字段公开 + derive Deserialize 绕过 | **已修**：字段私有 + `Deserialize` → `try_new` |
| `Decimal::new` 接受任意 u8 scale | **已修**：`scale > MAX_SCALE` **panic**（const）；生产用 `try_new` |
| `checked_rescale` 对 `new(1,255)` panic | **路径已不可达**（无法构造非法 scale 值） |
| 统一 `XError::Invalid` 英文 | **已修**：`DecimalError` 分类 + 中文 Display，再 `Into<XError>` |

### 2.3 decimalx — serde / wire

- 当前 shape：`{"mantissa":i128,"scale":u8}`（结构字段，**非**十进制字符串）。
- 反序列化强制 `try_new`；非法 scale → 失败。
- **内部** `DecimalWire` / `MoneyWire` **未** `deny_unknown_fields`：额外字段行为未硬化（`adversarial_serde` 明确「不强制」）。
- `docs/WIRE.md`：**不**等于跨版本 stable；SQL NUMERIC 不在本 crate 合同。

### 2.4 canonical — wire 版本与未知字段

| 版本常量 | 类型 |
|----------|------|
| `COMMITTED_WIRE_V1` | `CancelOrderRequest`, `OrderRef`, `OrderAck`, `OrderStatus`, `Side` |
| `COMMITTED_WIRE_V1_1` | `Order` |
| `COMMITTED_WIRE_V1_2` | `Tick`, `Trade` |
| `COMMITTED_WIRE_V1_3` | `Position`, `OrderBookSnapshot`, `PriceLevel`, `SymbolMeta` |

冻结策略（`src/wire.rs`）：

- committed 类型全部 `#[serde(deny_unknown_fields)]`
- 未知 variant → 反序列化失败（有拒绝样例）
- 缺字段 → 失败；无默认值
- 嵌套 `Price`/`Qty`/`Decimal` 非法 scale → 失败（透传 decimalx）
- **无 wire envelope / schema 版本字段**；N-1 靠 fixture + 测试
- 枚举 wire = Rust variant 名（`"Open"` / `{"Exchange":"..."}`）；**加 variant = 破坏性**（读者旧代码拒识；策略已文档化，非 open-enum）

**Uncommitted（相对本 crate wire 清单）**：`Money`、`VenueId`/`InstrumentId` alias 等——`Money` wire SSOT 在 decimalx；公开市场 DTO **均已晋升** committed（§11.2 DEFER-3 原文「Order/Tick/Trade 仍 Uncommitted」**已过时**，以 §12 / `wire.rs` 为准）。

### 2.5 canonical — validate 入口

| 层级 | 能力 | 边界 |
|------|------|------|
| serde | committed 拒绝未知字段/variant/缺字段；Decimal 校验 | **不是**业务校验 |
| `shape::*` | venue slug / instrument / `OrderRef` 非空 / `cancel_request_shape_ok` | adapter **可选**形状防御；**Deserialize 不自动调用** |
| `proposed_time::*` / `ns_from_unix_millis` | ms→ns，溢出 `None` | `ts: i64` **允许负值**（形状层不拦） |
| domain | 正 qty、状态机、symbol 存在性等 | **明确不在**本 crate |

### 2.6 依赖方向

```text
canonical → decimalx → kernel
```

- Money 类型恒等 re-export（`Money` ≡ `decimalx::Money`，有测）
- 禁止 types 依赖 testkit 作为 normal dep；canonical 不依赖 kernel（时间与 kernel 同刻度但分层）

---

## 3. 阻断项 / 改进项

### 3.1 阻断（生产资金 / 持久化路径）

| ID | 项 | 说明 |
|----|-----|------|
| B-D1 | 跨版本 decimal wire 协议未闭合 | 字段 shape 可变；不可把当前 serde 当长期存储/跨服务契约而不另定 schema |
| B-D2 | panicking 运算符仍公开 | `+`/`-`/`*`/`rescale`/`new` 可 panic；资金路径必须靠门禁 `check-decimal-no-panicking-ops.mjs` + 评审，**类型系统不消灭** |
| B-C1 | 无 wire envelope | 多版本并存时无法从载荷自描述 schema 版本；依赖部署协调 + fixture |
| B-C2 | 业务/入口 validate 非强制 | 仅 shape 辅助；脏 `venue`/`ts` 仍可反序列化成功 |

> **非阻断**：§4.1「非法状态可表示」与 §4.2「仅五类型 committed」在 HEAD 上**已不成立**为原 P0 形态——分别由字段私有/校验 serde 与 v1.1–v1.3 晋升闭合。

### 3.2 改进（P1/P2）

| ID | 优先级 | 项 |
|----|--------|-----|
| I-D1 | P1 | `Decimal`/`Money` 反序列化加 `deny_unknown_fields`（或文档冻结「忽略 extra」） |
| I-D2 | P1 | 完整 `cargo-fuzz` 靶（现有 proptest adversarial 为轻量替代） |
| I-D3 | P2 | benches 从 `.gitkeep` 升为可回归数值热路径 |
| I-D4 | P2 | SSOT/文档 package 名与 Cargo `decimalx` 对齐（消除 `xhyper-decimalx` 漂移） |
| I-D5 | P2 | 跨币种 `Money` 运算显式拒绝 API（当前无运算方法，靠「不提供」） |
| I-C1 | P1 | 可选 wire envelope（`schema_version`）或对外兼容矩阵文档化升级路径 |
| I-C2 | P1 | Deserialize 钩子或文档强制 adapter 调用 `shape::*` / `dto_ts_from_unix_millis` |
| I-C3 | P2 | `#![deny(missing_docs)]`（crate 自承 follow-up） |
| I-C4 | P2 | 恶意/超大 JSON 资源上限（当前无字节预算） |
| I-G1 | P2 | 文档中 `-p xhyper-decimalx` 命令改为 `-p decimalx`（与 metadata 一致） |

---

## 4. 与 core-crates 报告对照（HEAD 9174840）

| 报告位置 | 当时结论 | 本审计（9174840） |
|----------|----------|-------------------|
| §1 结论表 | decimalx/canonical **未就绪** | **过时**；以 §12 为准 |
| §4.1 P0 不变量可绕过 | 字段公开 / 非法 scale | **已修**（#98）；源码复核通过 |
| §4.2 P0 wire 未闭合 | 无 committed / deny_unknown | **v1 基线已修**；**v1.1–v1.3 已晋升**（#124 / W2） |
| §11.1 A/B | 不变量 + committed v1 闭合 | **仍成立** |
| §11.2 DEFER-3 | Order/Tick/Trade Uncommitted | **过时**；§12 Close 分批；源码清单已含 |
| §11.2 DEFER-4 | fuzz/oracle/mutants/Miri 未宣称 | **部分闭合**（oracle/boundary/proptest + scheduled CI）；无完整 cargo-fuzz |
| §11.3 | 有条件 / 部分就绪 | **仍诚实** |
| §12.3 | decimalx **L1**；canonical **L2** | **仍适用**，与源码一致 |
| §12.4 / 签核 | L5 GO-with-Accepts；整体 PR **否** | **仍适用** |

**结论**：在 HEAD `9174840` 上，**§12 附录判定继续有效**；引用本报告时勿再使用 §1「未就绪」作现状摘要。

---

## 5. 测试证据（本轮实测）

命令：

```bash
cargo test -p decimalx -p canonical --all-targets
```

| 目标 | suite | 结果 |
|------|-------|------|
| `canonical` unit（含 `wire`/`shape`/`proposed_time`） | 37 | ok |
| `canonical` `tests/public_api.rs` | 4 | ok |
| `decimalx` unit | 63 | ok |
| `decimalx` `adversarial_serde` | 4 | ok |
| `decimalx` `boundary_matrix` | 12 | ok |
| `decimalx` `entry_checked_ops` | 7 | ok |
| `decimalx` `oracle_diff` | 6 | ok |
| `decimalx` `proptest_ops` | 11 | ok |
| **合计** | **144** | **0 failed** |

证据类型映射：

| 关切 | 证据位置 |
|------|----------|
| 非法 scale / serde | unit `try_new_and_serde_reject_*`；`adversarial_serde`；wire 测非法 price scale |
| checked 无 panic | unit 溢出/MIN÷-1；`boundary_matrix`；`proptest` `no_panic_*` |
| 数值正确性 | `oracle_diff`（BigDecimal，仅 Ok 路径） |
| crate 外入口 | `entry_checked_ops`；`public_api` |
| committed wire 双向 + 拒绝 | `wire.rs` 测 + `fixtures/market/canonical/v1{,.1,.2,.3}/` |
| panicking 生产路径门禁 | `scripts/quality-gates/check-decimal-no-panicking-ops.mjs`（CI `validation.yml`） |
| align | `scripts/quality-gates/check-canonical-align.mjs` |
| scheduled | `.github/workflows/decimal-miri.yml`、`decimal-mutants.yml`、`decimal-coverage.yml`、`canonical-coverage.yml` |

**覆盖率历史**：§3.2 曾报 decimalx/canonical 行覆盖 100%（审计快照）；本 partial **未**重跑 llvm-cov。

---

## 6. STATUS 98% 落差

| 来源 | 数字 | 含义 |
|------|------|------|
| `STATUS.md` / `CRATES_STATUS.local.md` | decimalx / canonical **98%** | **结构进度**：布局八项×50% + 有测试×25% + LOC/内容×25% |
| 生产就绪（本审计） | L1 / L2 子集 | 不变量、wire 承诺、Accept 残留、禁止整体 PR |

官方口径（`docs/status/README.md`）：

> 完成度是**结构/可观测进度**，**不是** Production Ready 签字，也不是 SSOT 镜像 COMPLETE。

**落差解释**：

1. 98% ≈ 「crate 标准骨架 + 测试 + 实质源码」齐备，**不**评估跨版本 wire、fuzz 完整度、资金路径纪律自动化是否 100%。
2. `examples/` / `benches/` 多为 `.gitkeep` 仍可拿高 content 分（bootstrap 有可运行 example 才到 100%）。
3. 若把 98% 当成「可上资金生产」，属于**指标误用**；应对齐 §12：decimalx **L1**、canonical **L2 committed 子集**、**整体 Production Ready = 否**。

---

## 7. 签字 checklist（分层）

### 7.1 decimalx — L1 Internal Ready

| # | 项 | 状态 |
|---|-----|------|
| 1 | 非法 scale 不可表示（字段私有 + try_new/serde） | ✅ |
| 2 | `checked_*` 对可达状态无未声明 panic | ✅ |
| 3 | `DecimalError` 可分类 + 中文 Display | ✅ |
| 4 | 中间值溢出正式合同 | ✅ |
| 5 | 禁止 f32/f64 金额 API | ✅ |
| 6 | 资金路径仅 checked（门禁 + 文档） | ✅ 有条件（panicking API 仍存在） |
| 7 | oracle / 边界 / proptest 证据 | ✅ |
| 8 | package stable / 跨版本 wire | ❌ 明确未宣称 |
| 9 | 完整 cargo-fuzz / 持续 mutants·miri 必过 PR | ❌ scheduled / 轻量替代 |
| 10 | 整体 Production Ready | ❌ |

**建议签核用语**：`decimalx` **L1 Internal Ready（GO-with-Accepts：资金路径强制 checked；wire 非 stable）**。

### 7.2 canonical — L2 Wire Ready（子集）

| # | 项 | 状态 |
|---|-----|------|
| 1 | committed 清单显式（v1–v1.3） | ✅ 12 类型名 |
| 2 | 全部 committed `deny_unknown_fields` | ✅ |
| 3 | 双向 golden + N-1 fixture + 拒绝样例 | ✅ |
| 4 | 非法 Decimal scale 反序列化失败 | ✅ |
| 5 | `ts` = Unix ns 文档 + 转换入口 | ✅ |
| 6 | Money 复用 decimalx（类型恒等） | ✅ |
| 7 | 纯 DTO：无 I/O / 无业务方法 | ✅ |
| 8 | 业务 validate 强制 / shape 自动绑定 | ❌ 可选 shape |
| 9 | wire envelope / 跨语言协议 | ❌ |
| 10 | package Production Ready / stable | ❌ |

**建议签核用语**：`canonical` **L2 Wire Ready（仅 `COMMITTED_WIRE_V1{,_1,_2,_3}`；GO-with-Accepts：adapter 须自行 shape/时间换算）**。

### 7.3 禁止表述

- ❌ 「types 整体 Production Ready」
- ❌ 「STATUS 98% = 可上生产资金」
- ❌ 「decimal serde shape 跨版本 stable」
- ❌ 「canonical 全部 DTO 含 Money 已 wire stable」
- ❌ 使用过时 §1「decimalx/canonical 未就绪」描述 **post-#98 / post-W2** 现状（应写 L1/L2）

---

## 8. 源码与文档索引

| 路径 | 用途 |
|------|------|
| `/home/workspace/infra.rs/crates/types/decimal/src/lib.rs` | Decimal/Money/错误/serde/checked |
| `/home/workspace/infra.rs/crates/types/decimal/docs/WIRE.md` | wire 政策（非 stable） |
| `/home/workspace/infra.rs/crates/types/decimal/tests/*` | entry/oracle/boundary/adversarial/proptest |
| `/home/workspace/infra.rs/crates/types/canonical/src/{lib,wire,shape,proposed_time}.rs` | DTO + wire + 形状 + 时间 |
| `/home/workspace/infra.rs/fixtures/market/canonical/v1{,.1,.2,.3}/` | golden |
| `/home/workspace/infra.rs/docs/ssot/types-ssot-alignment.md` | 本仓对齐 SSOT |
| `/home/workspace/infra.rs/docs/report/2026-07-21/core-crates-production-readiness.md` | 主审计 + §11/§12 |
| `/home/workspace/infra.rs/docs/plans/releases/0.3.0-signoff.md` | L5 GO-with-Accepts |

---

## 9. partial 维护

| 日期 | HEAD | 说明 |
|------|------|------|
| 2026-07-21 | `9174840` | 初版：核实 §4 P0 已修、§12 L1/L2 仍适用；STATUS 98% 落差；测试 144 绿 |
