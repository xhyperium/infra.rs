# decimalx 当前设计

> **状态**：Active current-state design
>
> **范围**：L1 checked path；不声明跨语言 wire 或 package stable

## 设计目标

`decimalx` 在 `/types/` 层提供无浮点、无 I/O、无业务政策的十进制值类型。模块把可恢复失败留在
`DecimalError` 中，把非法 scale 与非法币种挡在构造边界，并让生产资金路径只依赖 checked API。

当前权威事实来自本仓宪章、[active spec](../spec/spec.md) 和
`crates/types/decimal/src/lib.rs`。历史 “ADR-006 / ADR-007” 仅是来源记录；本仓没有可解析的 ADR
原件，不将其作为当前裁决依据。

## 表示与封装

```text
Decimal { mantissa: i128, scale: u8 }  --私有字段--> mantissa() / scale()
Currency([u8; 3])                      --私有字段--> as_str() / as_bytes()
Money { amount, currency }             --私有字段--> amount() / currency()
Price(Decimal) / Qty(Decimal) / Ratio(Decimal)
```

`Decimal::try_new` 建立 `scale ≤ MAX_SCALE(18)` 不变量；`Currency::try_new` 建立三个大写 ASCII
字母不变量；`Money::try_new` 组合已校验的数值与币种。自定义 Deserialize 复用这些构造器，防止
serde 绕过封装。

`Decimal::new` 保留为 const、测试与兼容便利，越界会 panic；`rescale` 保留为测试与兼容便利，
checked 缩放失败会 panic。二者都不属于资金生产路径。

## 运算路径

```text
输入/parse
    │
    ▼
try_new / FromStr ──错误──> DecimalError
    │
    ▼
checked_add / checked_sub / checked_mul
checked_div(strategy) / checked_rescale(strategy)
    │                         │
    ├── 可表示 ──────────────> Decimal
    └── 越界/除零/舍入失败 ─> DecimalError
```

运算符 `+` / `-` / `*` 位于 default-off `panicking-ops` feature 后。该 feature 是兼容面，不是
生产能力。`div` 返回 `DecimalResult`，只是 `checked_div` 的已有名称别名。

所有中间算术使用 checked 操作。中间 `i128` 无法表示时即失败，不承诺自动切换 BigInt 或先约分
再重试。除法和缩位必须显式传入五种舍入策略之一。

## 文本设计

`Display` 使用 `unsigned_abs()` 处理负数绝对值，因此设计合同覆盖 `i128::MIN`。输出先规范化
尾随零；`FromStr` 必须能重新构造数值相等的 `Decimal`。负极值解析不能先把绝对值解析为 `i128`
再取负，因为 `|i128::MIN|` 超出正 `i128`；实现应按带符号字符串直接解析或采用等价的无溢出方案。

Display/FromStr 是内部 Rust 文本入口，不是跨语言或持久化格式。

## 错误设计

`DecimalError` 承载精确错误，`DecimalErrorKind` 提供不依赖文案的分类。转换为 `kernel::XError`
时，外层 kind 为 `Invalid`，原错误必须通过 `with_source` 或等价机制保留在 source chain 中。
这样既保持统一上层错误类型，也满足宪章“错误链不可断裂”的要求。

## serde 边界

wire v1 仅冻结内部 Rust serde JSON shape：Decimal 对象、Currency 三字节数组、Money 对象。
反序列化拒绝未知字段并走校验构造器。它不解决 JSON consumer 对 `i128` 的精度限制，因此不能
直接作为 JavaScript 或其他跨语言消费者的精确金额协议。

如需跨语言协议，必须单独裁定字符串/分段整数/二进制编码、schema 版本、canonical 规则与迁移
策略；不得通过改写当前 serde shape 暗中完成。

## 依赖与演进

- 生产依赖只允许 `kernel` 与 `serde`；禁止依赖 `canonical`，保持
  `canonical → decimalx → kernel` 的方向。
- `MAX_SCALE = 18` 是当前生产合同，不把技术常量 38 当作业务 scale。
- 字段已经私有；未来变更访问器或 serde shape 仍需兼容审查。
- 源码/API/行为交付按独立 crate 版本规则执行一次 PATCH +1。
