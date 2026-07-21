# `decimalx` 候选完整规范

```text
Spec ID:       SPEC-TYPES-DECIMALX-002
Status:        Draft / Non-normative candidate
Active SSOT:   .agents/ssot/types/decimal/decimalx-spec.md
Source Goal:   GOAL-TYPES-DECIMALX-002
Package:       decimalx 0.1.0
Lib / Path:    decimalx / crates/types/decimal
Layer:         Types / financial numeric primitives
Snapshot:      95102919 (2026-07-15)
Supersedes:    none while Draft
```

## 0. 文档定位

本 Draft 记录当前数值合同、已批准 ADR 与候选加固路径，不覆盖 [active spec](../decimalx-spec.md)。字段私有化、wire 改动、路径/依赖或公开签名变化均需兼容审查；涉及架构时先走 `docs/specs/`。

证据使用 `[KNOWN] <confidence>` / `[INFERRED] <confidence>`；需求状态使用 `APPROVED` / `PROPOSED` / `OPEN` / `REJECTED`。共同失败条件：snapshot 后实现、ADR 或 consumer inventory 改变。

## 1. 当前事实与漂移

| 主题 | 当前事实 | 风险/开放项 |
|---|---|---|
| 路径 | `crates/types/decimal` | 原 Draft 的 numeric 迁移未批准 |
| 依赖 | `xhyper-kernel`、`serde`；dev criterion/proptest/serde_json | 禁止反向依赖 canonical |
| 表示 | `pub mantissa: i128`, `pub scale: u8` | 任意 scale 可构造 |
| 算术 | checked + panicking operators/rescale | 生产调用策略待收敛 |
| 除法 | 结果 scale = `max(lhs.scale,rhs.scale)` | 是否需要显式 target scale 为 OPEN |
| 舍入 | Floor/Ceiling/HalfUp/HalfDown/HalfEven | 已由 ADR-006 批准 |
| 相等 | 数值 Eq/Ord/Hash，normalize trailing zero | 需扩大边界验证 |
| wire | serde 默认字段 shape | 跨版本承诺未批准 |
| Currency | parse 校验 3 位大写 ASCII；tuple 字段公开 | 非法直接构造仍可能 |

反例条件：字段可见性、panic 行为、serde shape、ADR-006/007 或定向 consumer inventory 任一变化，都会推翻对应 baseline/迁移成本结论。

## 2. 已批准基础合同

### DEC-LAYER-001 — 定位与依赖（`APPROVED`）

- `decimalx` 位于 `/types/`，提供纯数值和值对象，无业务逻辑。
- `canonical` 依赖 `decimalx`；`decimalx` 不依赖 `canonical`、contracts、domain、adapter、L1。
- `Money`/`Decimal`/`Currency` 族只有一个定义点。

### DEC-REP-001 — 数值表示（`APPROVED`）

```text
value = mantissa × 10^(-scale)
mantissa: i128
scale: u8 (表示类型事实；合法业务上限仍 OPEN)
```

没有 NaN/Infinity；负零按数值等于零并 normalize 为 `(0,0)`。

### DEC-ROUND-001 — 舍入（`APPROVED`）

- 加减先无损对齐到较大 scale；溢出返回错误。
- 乘法 mantissa 相乘、scale 相加，均 checked。
- 除法和缩位必须显式传 `RoundingStrategy`，无默认 fallback。
- Floor 向负无穷、Ceiling 向正无穷；HalfUp/Down/Even 的正负中点必须一致。

### DEC-FLOAT-001 — Float 禁止（`APPROVED`）

金额、价格、数量、比率的构造、运算、adapter 解析、存储和协议转换不得经过 `f32/f64`。源码字符串扫描只是辅助证据，必须结合 API 与 consumer 测试。

## 3. 当前公开 API 合同

### 3.1 `Decimal`

当前公开：`new`、`ZERO`、`cmp_value`、`eq_value`、`checked_add/sub/mul/div`、`div`、`checked_rescale`、`rescale`、`normalize`、`FromStr`、`Display`、`Add/Sub/Mul`、数值 `Eq/Ord/Hash`。

`rescale` 与 operators 溢出 panic 是兼容事实，不是推荐生产错误处理。`FromStr` 当前接受 trim、可选正负号、`.5`/`5.`，拒绝 NaN/Inf、多个点、非数字和 i128/u8 溢出。

### 3.2 强类型

- `Price(pub Decimal)`、`Qty(pub Decimal)`、`Ratio(pub Decimal)` 当前只提供类型区分，无正值/范围业务校验。
- `Currency(pub [u8;3])` 的 `FromStr` 校验三位大写 ASCII；`as_str` 对非法公开字段返回空串。
- `Money { pub amount, pub currency }` 不提供跨币种运算。

## 4. 候选加固要求

### DEC-LIMIT-001 — Limits（`PROPOSED`）

先从数据库精度、provider 精度、Display/parse 和现有数据统计推导合法 `MAX_SCALE`，不得凭惯例写死 38。批准后新增 fallible constructor 与 accessors；内部先迁移，公开字段按兼容计划收紧。

失败条件：仍可通过推荐 API 构造导致格式化、对齐或存储不可表示的值。

### DEC-ERR-001 — 错误分类（`PROPOSED`）

至少区分非法输入、除零、scale/数值 overflow 与不精确结果（若承诺 exact API）。是否继续映射为 kernel `Invalid` 或引入本地 `DecimalError` 必须评估下游和公开签名；本 Draft 不预设最终 enum。

### DEC-PANIC-001 — Panic 面（`PROPOSED`）

- checked API 是生产默认。
- 建立 `+/-/*/rescale` 调用点清单，迁移资金路径。
- panicking API 若保留，rustdoc 必须有 `# Panics` 与明确前置条件；若弃用，先 additive 替代并完成 consumer=0。
- 禁止在 release 通过饱和、wrap 或默认零值掩盖溢出。

### DEC-EQ-001 — 当前 Eq/Ord/Hash 语义（`[KNOWN] HIGH`）

当前实现要求值相等产生相同 hash 和 `Ordering::Equal`；这是代码/API baseline，不因存在即自动成为新的架构批准。

### DEC-EQ-002 — 全边界验证（`PROPOSED`）

对齐溢出时的比较算法需用 property/differential oracle 验证，不能仅用少数示例支持全 i128/u8 结论。

### DEC-DIV-001 — 除法目标精度（`OPEN`）

当前 `checked_div` 结果 scale 固定为两侧较大值。若真实消费者需要显式 target scale，应 additive 新增方法；不得修改旧签名或声称当前已有该能力。exact division / inexact status 也只在 consumer 证明需要时设计。

### DEC-WIRE-001 — Wire/Storage（`OPEN`）

- serde `{mantissa,scale}` 是当前 shape；是否稳定需单独批准。
- `Display` 是规范化人读文本，不自动等同持久化协议。
- JSON、Protobuf、SQL NUMERIC 与 evidence canonical encoding 各自拥有版本/范围/round-trip 合同。
- 移除 serde 前必须迁移 canonical DTO、fixtures、adapters 和持久化数据。

## 5. 兼容迁移

1. 以 `cargo metadata` + `rg` 固定 consumer/API/wire baseline。
2. 新增 fallible/checked/validated API，不立即破坏字段和 constructors。
3. 迁移 domain、ledger、binance、okx、taos、canonical、schema_codegen。
4. 建立历史 serde/storage reader 或证明没有持久化数据。
5. consumer=0、数据兼容和批准闭合后，才讨论字段私有化或 API 删除。

路径重命名、`Money<U>`、`decimalx → canonical` 不属于该迁移。

## 6. 测试与 Evidence

必须覆盖：

- 五种舍入在正负、精确、中点、非中点、奇偶分母边界；
- add/sub/mul/div/rescale 的 i128 与 scale overflow；
- Eq/Ord/Hash 的反身、对称、传递和 hash consistency；
- parse/Display 在合法 limits 全边界往返；
- `Currency` 非法直接构造迁移与脱困；
- 与独立高精度 oracle 的 differential tests（oracle 仅 dev）；
- 下游金融路径无 float、无 silent wrap、无未说明 panic。

聚焦命令：

```bash
cargo test -p xhyper-decimalx
cargo check -p decimalx --all-targets
cargo clippy -p decimalx --all-targets -- -D warnings
cargo xtl lint-deps
cargo fmt -- --check
```

涉及公开/wire 变更时追加所有 consumer 测试和 API diff。当前 property tests 通过不等于全 i128/u8 状态空间已证明。

## 7. 完成与晋级

- [ ] active spec 与当前 API、panic、serde、consumer 事实一致。
- [ ] DEC-LIMIT/ERR/PANIC/DIV/WIRE 的目标范围均获批准或显式延期。
- [ ] ADR-006/007 不变量由代码、property/differential 和依赖门禁证明。
- [ ] 破坏性收紧具备完整下游和历史数据迁移证据。
- [ ] 不存在路径/依赖反转或未经需求证明的大型类型体系。

批准本 Draft 不等于已完成迁移；stable 必须以全部开放风险闭合为证据。
