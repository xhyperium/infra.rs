# decimalx 当前实现规范

> **状态**：Active current-state SSOT · L1 checked path 可生产使用
>
> **边界**：不是跨语言 wire 协议，也不声明 package stable / crates.io Production Ready
>
> **实现**：`crates/types/decimal`，package/lib `decimalx`，当前交付版本 `0.1.2`

## 1. 权威性与范围

当前事实按以下顺序判定：

1. 本仓 [`CONSTITUTION.md`](../../../../../CONSTITUTION.md) 及
   [`docs/constitution/`](../../../../../docs/constitution/)；
2. 本 active spec；
3. [`crates/types/decimal/src/lib.rs`](../../../../../crates/types/decimal/src/lib.rs) 与
   `Cargo.toml`、测试所证明的实现事实。

历史材料中的 “ADR-006 / ADR-007” 和 “XLib spec” 只记录本设计的来源。仓库中没有可解析的
ADR-006/007 权威原件，因此不得用这些编号覆盖本仓宪章、本文或当前源码。

`decimalx` 是 `/types/` 的纯十进制值类型模块，定义 `Decimal`、`Price`、`Qty`、`Ratio`、
`Currency` 与 `Money`。它不负责汇率、跨币种运算、定价、手续费、会计政策、交易所 tick/step、
数据库映射、网络协议、I/O 或运行时装配。金额、价格和数量运算禁止经过 `f32` / `f64`。

## 2. 表示与不变量

- `Decimal` 表示 `mantissa × 10^(-scale)`；`mantissa: i128`、`scale: u8` 均为私有字段，
  通过 `mantissa()` / `scale()` 读取。
- `MAX_SCALE = 18` 是当前生产构造、解析、反序列化与 checked 运算结果的强制上限；
  `TECH_MAX_POW10_EXP = 38` 仅说明 `i128` 的十次幂技术边界，不能替代 `MAX_SCALE`。
- `Currency([u8; 3])` 的字段私有。合法值恰为三个大写 ASCII 字母；通过 `try_new` / `FromStr`
  构造，通过 `as_str()` / `as_bytes()` 读取。
- `Money { amount: Decimal, currency: Currency }` 的字段私有。通过 `Money::try_new` 构造，
  通过 `amount()` / `currency()` 读取。
- `Price`、`Qty`、`Ratio` 是私有字段 newtype；它们只提供类型区分，不增加业务规则。
- `Decimal` 的 `Eq` / `Ord` / `Hash` 按数值语义处理；尾随零不同但数值相同的表示相等且同 hash。

非法 scale 和非法币种不能通过公开字段直接构造。serde 反序列化也必须经过相同校验边界。

## 3. 生产 API 合同

生产资金路径只能使用以下 fallible API：

- 构造与入口：`Decimal::try_new`、`FromStr` / `parse`、`validate`；
- 运算：`checked_add`、`checked_sub`、`checked_mul`、`checked_div`、`div`、
  `checked_rescale`；
- 币种与金额：`Currency::try_new` / `FromStr`、`Money::try_new`。

`Decimal::new` 在 `scale > MAX_SCALE` 时 panic。它是 const、测试和兼容便利，不是资金生产入口。
`Decimal::rescale` 会在 checked 缩放失败时 panic。它是测试和兼容便利，生产路径必须使用
`checked_rescale`。

`Add` / `Sub` / `Mul` 运算符仅在 feature `panicking-ops` 下提供，溢出时 panic；该 feature
默认关闭，生产资金路径不得启用或依赖这些运算符。`checked_*` 对所有可表示 `Decimal` 必须
返回 `Ok` / `Err`，不得因输入或算术边界 panic。

## 4. 算术与舍入

- 加减和比较将两侧无损对齐到较大 scale；若 `i128` 中间值不能表示，checked 运算返回错误。
- 乘法先计算 mantissa 乘积并合并 scale；结果可通过去除尾随零落回 `MAX_SCALE` 时允许返回，
  否则返回错误。
- 除法必须显式提供 `RoundingStrategy`，结果 scale 为 `max(lhs.scale, rhs.scale)`；`div` 是
  `checked_div` 的兼容别名，不是 panicking 路径。
- 缩位必须显式提供舍入策略；扩位必须可在 `i128` 内精确表示。
- 舍入策略固定为 `Floor`、`Ceiling`、`HalfUp`、`HalfDown`、`HalfEven`。
- `i128` 中间值溢出即返回 `DecimalError`，即使数学结果经额外约分可能可表示；禁止静默回绕。

## 5. 文本往返合同

`Display` 输出规范化十进制文本：去除无意义尾随小数零，整数不带小数点。`FromStr` 接受有限的
十进制文本，拒绝 NaN / Inf、非法字符、mantissa 溢出和 `scale > 18`。小数位数超过
`u8` 可表示范围时必须返回含真实位数的 `Parse` 诊断，禁止先窄化再把 256 等长度伪报为 0。

对每个可表示的 `Decimal`，包括 `mantissa = i128::MIN` 以及 `scale = 0`、`1`、`18` 的边界，
必须满足：

```text
value.to_string().parse::<Decimal>() == Ok(value)  // 按 Decimal 数值相等语义
```

该合同是 Rust 文本入口的数值往返合同，不把 `Display` 文本提升为数据库或跨语言持久化协议。

## 6. 错误合同

公开错误为 `DecimalError`，公开别名为 `DecimalResult<T> = Result<T, DecimalError>`。
`DecimalError::kind()` 返回稳定的程序化分类 `DecimalErrorKind`：

| `DecimalError` | `DecimalErrorKind` | 含义 |
|---|---|---|
| `ScaleOutOfRange { scale, max }` | `Scale` | scale 超过 `MAX_SCALE` |
| `MantissaOverflow` | `Mantissa` | mantissa 解析或运算溢出 |
| `DivisionByZero` | `DivisionByZero` | 除数为零 |
| `RoundingOverflow` | `Rounding` | 舍入步进溢出 |
| `RepresentationOverflow` | `Representation` | 对齐或中间表示范围不足 |
| `Parse(String)` | `Parse` | 文本解析失败 |
| `InvalidCurrency` | `Currency` | 币种不是三个大写 ASCII 字母 |

用户可见 `Display` 使用中文。`From<DecimalError> for kernel::XError` 必须映射为
`ErrorKind::Invalid`，并把原 `DecimalError` 保留为 `std::error::Error::source()`；只复制错误字符串、
丢失 source chain 不符合生产合同。

## 7. serde wire v1 的有限承诺

`WIRE_SCHEMA_VERSION = 1` 只标识本 crate 内部 Rust serde JSON shape：

| 类型 | v1 JSON shape | 反序列化 |
|---|---|---|
| `Decimal` | `{ "mantissa": i128, "scale": u8 }` | `try_new`；拒绝未知字段 |
| `Currency` | 三元素字节数组 | `Currency::try_new` |
| `Money` | `{ "amount": Decimal, "currency": Currency }` | `Money::try_new`；拒绝未知字段 |

这不是跨语言精确数值协议。尤其 JSON number 对 `i128` 的承载取决于消费端，JavaScript 等环境可能
不能精确表示大整数；跨语言表示、canonical encoding、迁移与兼容政策仍是 residual。不得仅凭
serde shape、schema 常量或往返测试宣称 wire stable / package stable。

## 8. 依赖、版本与交付边界

生产依赖仅为 `kernel` 与 workspace 集中管理的 `serde`；禁止 `decimalx → canonical` 反向依赖，
也不得引入运行时或平台依赖。版本以 `crates/types/decimal/Cargo.toml` 为准；本文当前交付版本为
`0.1.2`。相对 `0.1.1` 的行为变化已按
[`docs/governance/VERSIONING.md`](../../../../../docs/governance/VERSIONING.md) 执行一次 PATCH +1；
版本同步由实现/发布任务负责。

生产就绪声明严格限于 **L1 checked path**：合法构造、checked 算术、显式舍入、错误分类和内部
Rust serde 校验面。它不扩展为跨语言 wire、完整金融领域模型、package stable 或公开发布承诺。

## 9. 验收合同

- [x] `Decimal` / `Currency` / `Money` 字段私有，公开构造与读取 API 明确。
- [x] `MAX_SCALE = 18` 在生产 fallible 边界强制执行。
- [x] `DecimalError` / `DecimalErrorKind` 覆盖构造、解析、算术、舍入与币种错误。
- [x] 所有可表示 `Decimal` 的 `Display → FromStr` 数值往返通过边界与 property 测试。
- [x] `DecimalError → XError` 保留可访问的 source chain。
- [x] panicking 运算符由 default-off feature 隔离；生产路径只用 `checked_*`。
- [x] serde v1 限定为内部 Rust JSON shape；跨语言 `i128` 精确承载保持 residual。
- [x] 实现变更完成后，由执行/验证任务运行 [test](../test/test.md) 与 [gate](../gate/gate.md)
  所列门禁并保存证据。

以上验收项的实现已闭合；终门禁新增的 source 类型身份断言已 focused PASS。前一独立裁决因后续
代码/测试修复失效；最终内容候选 `f62859b` 已完成全量门禁，仍须独立复审。该状态不扩张为 Goal Achieved、
跨语言 wire stable、package stable 或公开发布承诺。
