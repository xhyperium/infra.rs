# decimalx 公开 API

**Package**：`decimalx` · **角色**：十进制 / Money  
**生产层级**：L1 Internal Ready

## 公开消费面

### 常量与边界

| 符号 | 说明 |
|------|------|
| `MAX_SCALE` | `18` |
| `TECH_MAX_POW10_EXP` | `38` |
| `DecimalLimits` | `MAX_SCALE` / `TECH_MAX_POW10_EXP` 关联常量 |

### Decimal

| 符号 | 说明 |
|------|------|
| `Decimal::new` / `try_new` / `ZERO` | 构造（`try_new` 生产推荐） |
| `mantissa` / `scale` / `is_within_limits` / `validate` | 查询 |
| `checked_add` / `checked_sub` / `checked_mul` / `checked_div` / `div` | 四则 |
| `rescale` / `checked_rescale` / `normalize` | 缩位与规范化 |
| `cmp_value` / `eq_value` | 数值比较 |
| `FromStr` / `Display` | 文本往返 |
| `+` / `-` / `*` | panicking 运算符（资金路径禁用） |
| serde `{mantissa, scale}` | 反序列化强制 `scale ≤ MAX_SCALE` |

### 舍入与错误

| 符号 | 说明 |
|------|------|
| `RoundingStrategy` | `Floor` · `Ceiling` · `HalfUp` · `HalfDown` · `HalfEven` |
| `DecimalError` | `ScaleOutOfRange` · `MantissaOverflow` · `DivisionByZero` · `RoundingOverflow` · `RepresentationOverflow` · `Parse` · `InvalidCurrency` |
| `DecimalError::kind` | → `DecimalErrorKind` |
| `DecimalResult<T>` | `Result<T, DecimalError>` |
| `From<DecimalError> for kernel::XError` | → `ErrorKind::Invalid` |

### Newtypes

| 类型 | 方法 |
|------|------|
| `Price` / `Qty` / `Ratio` | `new` · `as_decimal` · `into_inner` |
| `Currency` | `try_new` · `as_str` · `as_bytes` · `is_valid` · `validate` · `FromStr` |
| `Money` | `try_new` · `amount` · `currency` · `validate` |

## 最小用法

```rust
use decimalx::{Currency, Decimal, Money, RoundingStrategy};

let a = Decimal::new(10, 0);
let b = Decimal::new(3, 0);
let q = a.checked_div(b, RoundingStrategy::HalfEven).unwrap();
let ccy = Currency::try_new(*b"USD").unwrap();
let m = Money::try_new(a, ccy).unwrap();
assert_eq!(m.currency().as_str(), "USD");
let _ = q;
```

```bash
cargo run -p decimalx --example basic
```

## 覆盖

`tests/public_api_surface.rs` + `entry_checked_ops` / `oracle_diff` / `boundary_matrix` / `adversarial_serde` / `proptest_ops`。  
API 棘轮：`docs/api-baselines/decimalx.txt`。  
Wire 边界：见 [WIRE.md](./WIRE.md)。
