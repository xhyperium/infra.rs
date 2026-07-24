<!-- ssot:clause=domain_macro.clause.001 -->
<!-- ssot:trace=domain_macro.spec.001 -->
<!-- ssot:spec-profile=kernel.domain.v1 -->
<!-- ssot:spec-contract=identity|temporal|units|missing|determinism|errors|security|evidence|promotion|algorithm|compatibility -->
<!-- ssot:req=domain_macro.req.identity.001,domain_macro.req.temporal.001,domain_macro.req.values.001,domain_macro.req.failures.001,domain_macro.req.compatibility.001,domain_macro.req.evidence.001 -->
<!-- ssot:ac=domain_macro.ac.identity.001,domain_macro.ac.temporal.001,domain_macro.ac.values.001,domain_macro.ac.failures.001,domain_macro.ac.compatibility.001,domain_macro.ac.evidence.001 -->
# domain_macro 核心规格 v0.2.0-draft

> 状态：draft。该规格是跨来源数据契约，尚未宣称 `macrox` 已实现或可发布。
> 当前代码事实：`crates/macrox` **尚未落地**（计划 L0 宏观经济数据模型核心 crate）；实现状态以 `.agents/ssot/macro_data/manifest.json`（`domain_macro`: `not_started`）为准。

## 1. 目标与非目标

### 1.1 目标

- 为不同数据源提供稳定的宏观观测值对象、来源身份、业务期间、单位缩放、缺失值、发布和修订语义。
- 让构造、转换和反序列化共享同一验证边界；任意外部输入失败时返回可分类错误，不允许 panic。
- 将 Rust API 兼容与 JSON wire 兼容分开治理，支持黄金 fixture、N-1 读取和可回滚迁移。

### 1.2 非目标

- 不在 L0 实现 HTTP、认证、缓存、重试、代理、抓取或供应商反爬逻辑。
- 不把国家/货币的语法校验冒充为完整 ISO 注册表校验。
- 不把发布日历的预计日期、数据观测期间、发布时间和 vintage 混为同一时间。

## 2. 术语与身份

| 术语 | 规范定义 |
|---|---|
| `SourceSeriesId` | 来源、数据集和原始系列键的稳定不透明标识；保留原文，禁止当作 `IndicatorId` |
| `IndicatorId` | 本仓规范指标标识，恰好两个非空段：`CATEGORY.NAME`；供应商多段 ID 必须放在 `SourceSeriesId` |
| `Period` | 观测所属业务期间，不是发布时间；必须表示为 Date/Month/Quarter/Year 之一 |
| `Vintage` | 数据在某个 `as_of` 时点可见的版本；没有版本语义时使用明确的 `None` |
| `ObservationIdentity` | `source + source_series + indicator + subject + period + vintage` 的完整唯一键 |
| `PublicationInstant` | 带 UTC instant 的发布时间；可选保留发布方 IANA 时区和原始本地时间 |

`IndicatorId` 的语法为：`[A-Z][A-Z0-9_]*\.[A-Z][A-Z0-9_]*`，长度上限 128 个 ASCII 字节。语法有效不代表该指标已在注册表中分配；注册表检查是独立的 `IndicatorRegistry` 责任。

`IndicatorCategory` 是指标目录分类的非穷尽枚举，当前保留 `NationalIncome`、`PriceLevel`、`Employment`、`Monetary`、`Trade`、`Fiscal` 六个稳定标签；新增分类必须走 wire schema 兼容评估，不得把分类标签当作供应商原始系列 ID。

国家和货币值对象同样分两层：`CountryCode`/`CurrencyCode` 只验证 ASCII 大写长度（alpha-2/alpha-3），ISO 分配、保留和历史代码必须由带版本的注册表验证。地区聚合（如欧元区或英国子地区）不得塞进国家代码。

## 3. 时间与期间

```rust
pub enum Period {
    Date { date: Date },
    Month { year: i32, month: u8 },
    Quarter { year: i32, quarter: u8 },
    Year { year: i32 },
}

pub struct PublicationInstant {
    pub at_utc: DateTime<Utc>,
    pub source_timezone: Option<IanaTimezone>,
    pub source_local: Option<LocalDateTime>,
}
```

约束：月份为 1–12、季度为 1–4、年份在业务允许范围内；`source_local` 与 `source_timezone` 必须成对出现。DST 不存在/重复的本地时间必须由适配器转换时显式处理，L0 不猜测。

日历条目使用 `AllDay { date }` 或 `At { instant, timezone }` 互斥形态。`upcoming(now_utc)` 必须显式接收参考时间，先按 instant 排序，再按稳定的指标/来源键排序；已取消和已发布条目不进入 upcoming。

## 4. 单位、精度与缺失值

```rust
pub struct ScaledUnit {
    pub dimension: UnitDimension,
    pub currency: Option<CurrencyCode>,
    pub scale10: i32,
    pub basis: Option<IndexBasis>,
    pub change_kind: Option<ChangeKind>,
}

pub enum MissingReason { NotAvailable, NotApplicable, Suppressed, NotPublished }
pub enum NumericValue { Decimal(Decimal), Missing(MissingReason) }

pub enum IngestError {
    InvalidWire { code: ErrorCode, field_path: String },
    SchemaMismatch { expected: u16, actual: u16 },
    FixtureHashMismatch,
}
```

- `Percent` 的 wire 口径必须明确是百分点（5.2）还是比例（0.052）；两者不能共享同一单位标签。
- 金额必须携带币种和 `scale10`，例如“百万美元”不得静默当作“美元”。
- 指数必须携带基期（必要时基期值和季调口径）；变化率必须携带周期。
- 非有限数、负零、超出精度或越过实现范围的数值拒绝；不得以 `f64::NAN` 表示缺失。
- 精度和舍入策略由 wire schema 版本固定。若实现暂时使用 `f64`，必须在边界转换为有限值并记录允许误差；生产实现优先使用十进制定点。
- 置信区间必须同时提供置信水平，且 `lower <= upper`、三者均为有限值。

## 5. 观测与修订

```rust
pub struct Observation {
    pub identity: ObservationIdentity,
    pub value: NumericValue,
    pub unit: ScaledUnit,
    pub publication: Option<PublicationInstant>,
    pub status: PointStatus,
    pub source_observation_id: Option<String>,
}

pub struct RevisionChain {
    pub revisions: Vec<Revision>,
    pub current: NumericValue,
}

pub struct Revision {
    pub revision_number: u32,
    pub published_at: PublicationInstant,
    pub source_vintage: Option<Vintage>,
    pub previous: NumericValue,
    pub next: NumericValue,
}
```

`ObservationIdentity.period` 是期间的唯一事实源，`Observation` 不再重复保存 `period`；wire 若同时出现扁平 `period`，反序列化必须验证与 identity 相等，否则返回 `identity_period_mismatch`。`Vintage` 不是 `publication`：它描述来源在某个 as-of 时点可见的版本，`ingested_at` 只描述本仓接收时间，不能替代来源 vintage。

修订只能通过 `RevisionChain::append` 追加。追加时必须验证：编号从 1 连续递增、revision 时间严格单调、前值等于链上当前值、当前值等于末次新值、来源 vintage 不倒退、状态编号与链长度一致。失败操作保持聚合不变。单独构造 `Revision` 不得声称已验证“递增”。修订失败是摄取错误，不得编码为合法 `Missing` observation。

初始值、修订值和状态都属于同一个 `ObservationIdentity`；来源、单位、期间或 vintage 改变时创建新身份，不覆盖旧观测。

## 6. 快照、重复和差异

```rust
pub struct MacroState {
    pub schema_version: u16,
    pub as_of: DateTime<Utc>,
    pub observations: Vec<Observation>,
    pub batch_id: BatchId,
}

pub enum MacroDiff {
    Added { new: Observation },
    Removed { old: Observation },
    Changed { old: Observation, new: Observation, value_delta: Option<Decimal> },
}
```

快照构造必须拒绝重复 `ObservationIdentity`、晚于 `as_of` 的 publication/revision、键与值身份不一致和非确定性排序。wire 层使用数组，不直接序列化 `HashMap<struct, value>`；消费者按 identity 建索引。

`change_pct` 不作为必有字段：旧值为零、缺失或单位不同均返回 `None` 并保留变化类型，不生成无穷或伪造百分比。派生算法必须声明输入 identity 集合、算法版本、舍入模式和失败条件；任何插值/聚合失败都返回 `IngestError`，不生成 Missing。

## 7. 错误契约

错误必须包含稳定 `code`、有限长度的字段路径和脱敏上下文；不得包含 API key、Cookie、完整认证 URL、原始响应或无界用户文本。建议最小错误码：

`invalid_country_code`、`invalid_currency_code`、`invalid_indicator_id`、`invalid_period`、`invalid_timezone`、`identity_period_mismatch`、`invalid_unit`、`non_finite_value`、`invalid_confidence_interval`、`duplicate_observation`、`revision_chain_violation`、`as_of_violation`、`wire_schema_unsupported`、`wire_decode_failed`、`fixture_hash_mismatch`。

所有公开入口（构造器、`TryFrom`、反序列化、快照插入、修订追加和派生算法）必须返回 `Result`；禁止 `unwrap`、`expect`、`panic!` 和 `todo!` 作为外部输入路径。`MissingReason` 只表示来源业务缺失，解析失败、schema drift、身份冲突和 hash mismatch 必须走 `IngestError`。

## 8. Wire schema 与兼容

规范 JSON 使用 envelope：

```json
{
  "schema_version": 1,
  "as_of": "2026-01-01T00:00:00Z",
  "observations": [],
  "batch_id": "opaque"
}
```

日期、时间、枚举、Decimal、MissingReason、Identity 和 Unit 的 JSON 形态必须在 schema 中固定；未知字段默认可忽略但必须保留兼容测试，未知枚举值默认拒绝并返回稳定错误。新增字段只有在有默认语义且不改变必需性时才可向后兼容。Rust 结构体字段字面量兼容和 wire 兼容分开评估。

每次 schema 变更必须提供：当前版本 roundtrip、N-1 读取、未知字段、未知枚举、损坏输入、迁移失败回滚和 golden JSON fixture。未提供 fixture 不得将 `spec_status` 标为 `verified`。

## 9. 验证矩阵

| ID | 规则 | 失败证据 |
|---|---|---|
| DM-V01 | 受约束值对象只能通过验证入口构造 | 非法 ASCII/Unicode/越界输入返回稳定错误 |
| DM-V02 | Period、publication、vintage 语义分离 | DST、跨日和晚于 as_of 输入被拒绝 |
| DM-V03 | 单位含缩放、基期和变化口径 | 百分点/比例、百万金额和非有限值边界测试 |
| DM-V04 | RevisionChain 连续、单调、原子追加 | 断号、逆序、前值不匹配测试 |
| DM-V05 | ObservationIdentity 防止跨源覆盖 | 重复、来源变化、零值 diff 测试 |
| DM-V06 | JSON envelope 可 roundtrip 且 N-1 可读 | golden、迁移、未知字段/枚举 fixture |
| DM-V07 | 错误不泄露秘密且外部输入不 panic | Debug/JSON/error/tracing 脱敏测试与 fuzz |

当前所有条款均为设计输入；是否落地只由 manifest、真实代码、测试和 evidence 联合证明。
